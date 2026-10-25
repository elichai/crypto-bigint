#!/usr/bin/env python3
"""Run every discovered library proof except explicitly recorded pending obligations."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import signal
import subprocess
import sys
import threading


ROOT = Path(__file__).resolve().parent.parent
VERSION = "0.68.0"


def inventory_names(inventory):
    if inventory["kani-version"] != VERSION:
        raise ValueError("unexpected Kani inventory version")
    if inventory["contract-harnesses"] or inventory["contracts"]:
        raise ValueError("contract proofs need an explicit CI policy")
    names = [name for group in inventory["standard-harnesses"].values() for name in group]
    if not names or len(names) != len(set(names)):
        raise ValueError("empty or duplicate harness inventory")
    if inventory["totals"]["standard-harnesses"] != len(names):
        raise ValueError("inventory count mismatch")
    return sorted(names)


def select_harnesses(names, pending, word, mode, shard, shards):
    if shards < 1 or not 0 <= shard < shards:
        raise ValueError("invalid shard")
    excluded = {}
    for item in pending:
        if not item["reason"] or not item["words"] or set(item["words"]) - {32, 64}:
            raise ValueError("invalid pending obligation")
        if word in item["words"]:
            name = item["harness"]
            if name in excluded or name not in names:
                raise ValueError(f"duplicate or stale pending obligation: {name}")
            excluded[name] = item["reason"]
    selected = [name for name in names if (name in excluded) == (mode == "pending")]
    return [name for name in selected
            if int.from_bytes(hashlib.sha256(name.encode()).digest()[:8], "big") % shards == shard], excluded


def validate_results(payload, selected, returncode):
    verification = payload["verification_results"]
    results = verification["results"]
    names = [result["harness_id"] for result in results]
    if len(names) != len(set(names)) or set(names) != set(selected):
        raise ValueError("missing, duplicate, or unexpected verification results")
    summary = verification["summary"]
    if summary["status"] != "completed" or summary["executed"] != len(selected):
        raise ValueError("incomplete verification run")
    errors = {item["harness_id"]: item for item in payload["error_details"]}
    if len(errors) != len(payload["error_details"]) or set(errors) != set(selected):
        raise ValueError("missing, duplicate, or unexpected error details")
    failures = [result["harness_id"] for result in results
                if result["status"] != "Success" or errors[result["harness_id"]]["has_errors"]]
    if returncode or summary["failed"] or summary["successful"] != len(selected):
        if not failures:
            raise ValueError(f"Kani exited {returncode} or reported an inconsistent summary")
    return failures


def stream_progress(log, stopped):
    with log.open(errors="replace") as source:
        pending = ""
        while True:
            line = source.readline()
            if line:
                pending += line
                if not line.endswith("\n"):
                    continue
            elif not stopped.is_set():
                stopped.wait(0.2)
                continue
            if pending.lstrip().startswith(("Checking harness", "VERIFICATION:",
                                           "CBMC failed", "Verification Time")):
                print(pending.rstrip(), flush=True)
            pending = ""
            if not line:
                return


def run(command, env, log, timeout):
    print(shlex.join(command), flush=True)
    # Kani can leave solver children alive after its own per-harness timeout.
    with log.open("w") as output:
        process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=output,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        stopped = threading.Event()
        progress = threading.Thread(target=stream_progress, args=(log, stopped), daemon=True)
        try:
            progress.start()
            return process.wait(timeout=timeout)
        finally:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
            stopped.set()
            if progress.ident is not None:
                progress.join(timeout=2)


def print_resources():
    try:
        ram = os.sysconf("SC_PHYS_PAGES") * os.sysconf("SC_PAGE_SIZE")
    except (AttributeError, OSError, ValueError):
        ram = "unavailable"
    try:
        memory_limit = Path("/sys/fs/cgroup/memory.max").read_text().strip()
    except OSError:
        memory_limit = "unavailable"
    print(f"Host resources: cpus={os.cpu_count()} ram_bytes={ram} "
          f"disk_free_bytes={shutil.disk_usage(ROOT).free} cgroup_memory_max={memory_limit}",
          flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--word", type=int, choices=[32, 64], required=True)
    parser.add_argument("--shard", type=int, default=0)
    parser.add_argument("--shards", type=int, default=1)
    parser.add_argument("--mode", choices=["gate", "pending"], default="gate")
    parser.add_argument("--timeout", type=int, default=300, help="per-harness seconds")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.timeout < 2:
        parser.error("timeout must be at least two seconds")
    print_resources()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    env["RUSTFLAGS"] = "--cfg miri" + (' --cfg cpubits="32"' if args.word == 32 else "")
    env["CARGO_TARGET_DIR"] = str(ROOT / "target" / f"kani-ci-{args.word}")
    env["CARGO_INCREMENTAL"] = "0"
    real_z3 = shutil.which("z3")
    if not real_z3:
        raise ValueError("Z3 is required by per-harness solver attributes")
    solver_dir = output / "solver-bin"
    solver_dir.mkdir()
    wrapper = solver_dir / "z3"
    wrapper.write_text(f"#!/bin/sh\nexec {shlex.quote(real_z3)} -T:{args.timeout - 1} -memory:4096 \"$@\"\n")
    wrapper.chmod(0o755)
    env["PATH"] = str(solver_dir) + os.pathsep + env["PATH"]
    common = ["cargo", "kani", "--lib", "--features", "alloc"]
    # Kani list writes its inventory into the working directory.
    inventory_path = ROOT / "kani-list.json"
    if inventory_path.exists():
        raise ValueError("kani-list.json already exists; move it before running CI")
    if run(common + ["list", "--format", "json"], env, output / "inventory.log", 600):
        raise ValueError("Kani inventory failed; see inventory.log")
    inventory_path.rename(output / "inventory.json")
    names = inventory_names(json.loads((output / "inventory.json").read_text()))
    pending = json.loads((ROOT / "scripts/kani-ci-pending.json").read_text())
    selected, excluded = select_harnesses(names, pending, args.word, args.mode,
                                         args.shard, args.shards)
    plan = {"word_bits": args.word, "pointer_bits": 64, "mode": args.mode,
            "shard": args.shard, "shards": args.shards, "inventory_count": len(names),
            "selected": selected, "pending_unproved": excluded, "rustflags": env["RUSTFLAGS"]}
    (output / "selection.json").write_text(json.dumps(plan, indent=2) + "\n")
    summary = (f"Kani Word{args.word} {args.mode}, shard {args.shard}/{args.shards}: "
               f"{len(selected)} selected; {len(excluded)} pending and unproved.\n")
    print(summary, flush=True)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as report:
            report.write(summary + "\n" + "\n".join(
                f"- Pending: `{name}` — {reason}" for name, reason in excluded.items()) + "\n")
    if not selected:
        raise ValueError("empty shard; reduce shard count")
    command = common + ["--exact", "-j1", "-Z", "unstable-options",
                        "--harness-timeout", f"{args.timeout}s", "--export-json",
                        str(output / "results.json")]
    for name in selected:
        command += ["--harness", name]
    code = run(command, env, output / "verification.log", len(selected) * (args.timeout + 10) + 600)
    failures = validate_results(json.loads((output / "results.json").read_text()), selected, code)
    (output / "status.json").write_text(json.dumps({"exit_code": code, "failures": failures}, indent=2) + "\n")
    if failures:
        print("Unproved or failed harnesses:\n" + "\n".join(failures), file=sys.stderr)
        return 1
    print(f"Verified {len(selected)} selected library harnesses; {len(excluded)} remain pending.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, KeyError, OSError, subprocess.TimeoutExpired) as error:
        print(f"Kani CI failed: {error}", file=sys.stderr)
        sys.exit(1)
