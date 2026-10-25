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
The workflow installs Z3 explicitly. A local PATH wrapper bounds each Z3 process too, and
the runner kills the verifier process group after a run to clean up remaining solver children.
No global external SMT argument is passed to CaDiCaL. Default memory, overflow, assertion,
and unwinding checks remain enabled. Exit status, exact harness coverage, result statuses,
error details, and summary counts must all indicate success. Missing JSON is a failure.

The separate `tests/kani_regressions.rs` contains unresolved bug counterexamples. `--lib`
keeps those out of this gate. They must not be reported as passing library proofs.
To retry pending library obligations, manually dispatch with `pending: true`. Those jobs
use a 600-second budget and remain failed if any obligation is unproved. Configure branch
protection to require the eight `Kani Word… shard …` checks and `Kani runner tests`;
the diagnostic pending checks should not be required. No `continue-on-error` masks results.

The workflow caches the versioned Kani bundle, its nightly toolchain, the Cargo launcher,
and Cargo registry/git downloads. Keys include OS, architecture, Kani version, and the
manifest/lockfile digest. These downloads contain no word-specific build outputs.
Build directories are separated by word width and are not cached. Proof results are always
recomputed; they are uploaded as evidence, never restored as proof authority. Solver and
build binaries are not committed. Keep the committed lockfile and update cache version `v1`
when changing the installation layout.

From the repository root, with Kani 0.68.0 and Z3 installed:

```sh
python3 scripts/kani-ci-test.py -v
python3 scripts/kani-ci.py --word 64 --shard 0 --shards 4 --output /tmp/kani-word64-shard0
python3 scripts/kani-ci.py --word 32 --mode pending --timeout 600 --output /tmp/kani-pending32
```

Use a fresh output directory. Each run retains inventory, exact selection, JSON results,
status, and logs. Kani also generates its usual ignored `target/` artifacts.
The official [Kani action](https://github.com/model-checking/kani-github-action) is pinned
to an inspected commit; its version input installs the specified Kani release.
The [Kani documentation](https://model-checking.github.io/kani/) describes the verifier's
scope and limitations. A passing gate proves the selected harness properties within their
domains and unwind bounds; it does not prove the whole library or constant-time execution.
