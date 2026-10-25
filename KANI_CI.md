# Kani CI

The `Kani` workflow verifies library harnesses with Kani **0.68.0** on Ubuntu x86_64.
It runs on pull requests, pushes to `master` and `elichai/kani-proofs-ci`, and manual dispatch.
Four deterministic shards run sequential proofs (`-j1`) for each arithmetic word width.
The 32-bit configuration uses `--cfg cpubits="32"` on a host with 64-bit pointers;
it is not a 32-bit target proof. Both use `--cfg miri` to select portable `cmov`.

`scripts/kani-ci.py` discovers the current `--lib --features alloc` inventory on every run.
Every discovered harness is selected exactly once across the shards, except the explicit
pending obligations in `scripts/kani-ci-pending.json`. New proofs join the gate automatically.
Stale or duplicate pending names fail the runner. Each pending entry states its word widths
and reason. Pending obligations are **unproved**, excluded from verified counts, and listed
in each job summary and `selection.json`. They are not expected successes.

The gate gives each harness 300 seconds and respects its `#[kani::solver(...)]` attribute.
The workflow installs the official **Z3 5.3.1** x86_64/glibc-2.39 release, matching the
local verification version. `scripts/kani-install-z3.sh` verifies its pinned SHA-256 digest
on both downloads and cache hits, checks its version, and adds its binary directory to PATH.
The installer checks the Ubuntu 24.04 runner's Linux architecture and glibc version.
The old Ubuntu package (Z3 4.8.12) timed out on `boxed_square_wide_64` with forced32 in
the first hosted run. With Z3 5.3.1, that proof passes in 25.421 seconds and its shard
passes all 85 selected harnesses in run `40184650379`.
A local PATH wrapper bounds each Z3 process's time and virtual memory (`-memory:4096`, MB), and
the runner kills the verifier process group after a run to clean up remaining solver children.
The memory cap leaves headroom for CBMC and the runner; it does not cap their own memory.
Resource exhaustion is an unproved failure, never a reason to skip a proof.
The runner streams harness names, verification outcomes, CBMC failures, and timings to the
job console while preserving its full log. Streamed names remain in the Actions log if a
runner shutdown prevents artifact upload. See `KANI_PLAN.md` for the validation history.
Each run also prints one host snapshot: CPU count, physical RAM and free disk bytes, and the
Linux cgroup `memory.max` value when available. These readings are diagnostic context.
No global external SMT argument is passed to CaDiCaL. Default memory, overflow, assertion,
and unwinding checks remain enabled. Exit status, exact harness coverage, result statuses,
error details, and summary counts must all indicate success. Missing JSON is a failure.

The separate `tests/kani_regressions.rs` contains unresolved bug counterexamples. `--lib`
keeps those out of this gate. They must not be reported as passing library proofs.
To retry pending library obligations, manually dispatch with `pending: true`. Those jobs
use a 600-second budget and remain failed if any obligation is unproved. Configure branch
protection to require the eight `Kani Word… shard …` checks and `Kani runner tests`;
the diagnostic pending checks should not be required. No `continue-on-error` masks results.

The workflow caches Kani's nightly toolchain, the Cargo launcher, and Cargo registry/git
downloads. Keys include OS, architecture, Kani version, and the
manifest/lockfile digest. These downloads contain no word-specific build outputs.
The official Kani action always downloads its bundle during setup, so that bundle is not
cached. Setup recreates the toolchain link. A separate Z3 archive cache is keyed by OS,
architecture, version, and archive digest; the digest is verified before each extraction.
Build directories are separated by word width and are not cached. Proof results are always
recomputed; they are uploaded as evidence, never restored as proof authority. Solver and
build binaries are not committed. Keep the committed lockfile and update cache version `v2`
when changing the installation layout.

From the repository root, with Kani 0.68.0 and Z3 5.3.1 installed:

```sh
python3 scripts/kani-ci-test.py -v
python3 scripts/kani-ci.py --word 64 --shard 0 --shards 4 --output /tmp/kani-word64-shard0
python3 scripts/kani-ci.py --word 32 --mode pending --timeout 600 --output /tmp/kani-pending32
```

Use a fresh output directory. Each run retains inventory, exact selection, JSON results,
status, and logs. Kani also generates its usual ignored `target/` artifacts.
The official [Kani action](https://github.com/model-checking/kani-github-action) is pinned
to an inspected commit; its version input installs the specified Kani release.
The pinned [Z3 release](https://github.com/Z3Prover/z3/releases/tag/z3-5.3.1) archive is
`z3-5.3.1-x64-glibc-2.39.zip`, with SHA-256
`acca9625e8cc8edeb9647b653a1bd696544ddeb5d1c2970a4576734a0c175b40`,
as published in its GitHub release asset metadata.
The [Kani documentation](https://model-checking.github.io/kani/) describes the verifier's
scope and limitations. A passing gate proves the selected harness properties within their
domains and unwind bounds; it does not prove the whole library or constant-time execution.
