# Kani verification plan and status

## Clean branch, proof review and CI: 2026-10-25

The published proof branch starts directly at upstream `f9e15c8` (v0.7.8, upstream's
`master` branch). The accumulated proof delta was replayed onto that commit without merge
commits. The previous history and uncommitted work were preserved on the local
`elichai/kani-proofs-backup-20261025` branch and in scratch patches before replay.
Older merge hashes below describe historical checks, not the new branch's history.
All Rust source changes relative to current upstream are proof modules; Cargo's extra cfg lint
registration allows `cfg(kani)`. No local runtime fix is included.

### Additional proofs and restored regressions

Thirteen library harnesses were added per word width, including two promotions from the
unresolved regression suite. All thirteen have passing targeted Kani evidence on both widths.

- Four limb proofs check owned/borrowed AND, OR and XOR operators and assignments, plus NOT,
  against native Word results over every input. The existing inherent-method proof remains;
  duplicate inherent-method assertions were removed from the new harnesses.
- Two split proofs specify exact word positions, truncation, zero extension and checked
  dimension rejection for every three-word input. These strengthen coverage beyond a
  concat/split round trip. Array equality needs byte-comparison unwind bounds; comments
  explain the otherwise non-obvious bounds.
- Seven special-modulus proofs restore the exact three-word maximum-carry regression and
  boxed one-word zero-parameter regression; cover every boxed Word pair at `c=0`; every
  fixed two-word pair at `c=0`; the boxed counterpart of the exact carry boundary; and
  three independent maximum-`c` result boundaries for both storage types.
  The latter use `d=B+k`, `k in {0,1,MAX}`, with exact expected word arrays. An attempted
  arbitrary-Word `k` family timed out or hit solver errors and was discarded as a new
  experiment; these finite cases do not claim that larger domain.
- The existing five boxed modular/MAC harnesses were rerun successfully after upstream's
  special-modulus implementation change. Their assertions and input domains are retained.

Upstream PR #1368 fixes finding 1.5 and the one-limb `c=0` problem in section 2 of
`FINDINGS.md`. Two integration regressions moved into the library suite; **39 unresolved
regressions remain**. Direct `Uint<1>::mul_mod_special` still triggers a Kani compiler internal
error in an unreachable branch. No such harness is active: upstream native tests cover its
zero/max-parameter behavior on both word widths. The boxed one-word path is formally checked.

The new full two-word zero-parameter proof retains its complete domain. CaDiCaL handles
forced32; Z3 proves Word64 in about 28 seconds, so a conditional solver attribute records that
choice. New boxed one-word `c=0` paths need a larger unwind bound to cover the nonzero
placeholder-divisor computation; no unwinding checks are disabled.

### Local validation and cumulative evidence

| Word width | Library proofs with passing evidence | Unproved library obligations | Native tests passed |
|---|---:|---:|---:|
| 64 bits | 325 / 335 | 10 | 908 |
| 32 bits | 326 / 336 | 10 | 904 |

These counts combine prior per-harness results with the new targeted checks; they are not a
fresh full-library run or a function-coverage percentage. The new GitHub workflow independently
discovers and reruns the active inventory on the pushed commit. Its live results and uploaded
JSON are the authority for that CI run; historical passes are not substituted for a failed run.
The ten earlier unproved obligations per width remain explicit in
`scripts/kani-ci-pending.json`. The unchanged forced32 Montgomery inner proof was retried with
CaDiCaL for 300 seconds after the upstream const-evaluation change and still timed out.

Fresh all-feature native tests, including doctests, pass with warnings denied on both word
configurations. In particular, upstream fixed-width and boxed zero/max-`c` tests pass.
Formatting, whitespace, workflow lint and the CI runner's unit tests pass.
Evidence: `/tmp/kani-ci-round-native{64,32}.log`,
`/tmp/kani-ci-round-gaps{64,32}.json`,
`/tmp/kani-ci-round-special-*.json` and `/tmp/kani-ci-round-monty32.json`.
Mixed initial batches include failed experiments; only successful final-domain entries count.

### CI policy and remaining work

See [KANI_CI.md](KANI_CI.md) for commands, scope, caching, diagnostics and evidence.
Kani 0.68.0 is pinned. Eight jobs shard the active library inventory across 64/32-bit arithmetic
words, one verifier at a time per runner, with a 300-second per-harness budget. Newly discovered
harnesses join automatically. The gate requires exact result coverage and successful statuses,
exit code and error details. No disabled checks or blanket `continue-on-error` hide failures.

The explicit pending manifest is a backlog of unproved obligations, not successful proofs.
Manual diagnostics can retry it with larger budgets and remain visibly failed when unproved.
Known bug counterexamples are separate test harnesses (27 integration-file and 12 inline).
Branch protection is not changed; the documentation lists the checks a maintainer can require.

CI pins Z3 5.3.1 with a checked release-archive SHA-256, matching local verification. The first
hosted run used Ubuntu's Z3 4.8.12 and timed out on `boxed_square_wide_64` with forced32;
84 other harnesses in that shard passed. No counterexample was produced. With pinned Z3,
that proof passes in 25.421 seconds, and all 85 harnesses in its shard pass in run
`40184650379`. Six proof shards passed in that run. Forced32 shard 2 passed 73/74,
with `uint_mul_u64_correct` timing out at 300 seconds without a counterexample.
The unchanged multiplication property passes locally with CaDiCaL in 24.935 seconds, so
forced32 now selects CaDiCaL while Word64 retains Z3. Its domain and both exact-product
assertions are unchanged; the updated solver choice still needs its own hosted check.
Word64 shard 3 lost its hosted runner with exit 143 on two runs before preserving
results. The cause is unconfirmed; those runs do not establish a verification result.
The runner now streams active harnesses and outcomes, prints a host resource baseline,
and bounds each Z3 process to 4096 MB. Resource failures remain failures. CBMC's own memory
is not bounded by that solver option. The updated runner needs its own hosted check.

CI caches the exact Kani nightly/toolchain, Cargo downloads and the checked Z3 archive.
Kani's installer downloads its bundle again even on cache hits, so that bundle is not cached.
CI does not cache successful proof results or commit generated solver/build binaries.
Every selected property is checked again. Cache hits
reduce setup; they do not eliminate the dominant solver work. The historical local timings
are not a benchmark of GitHub-hosted performance.

The review found useful additions; it cannot establish that no more proofs can be written.
Priority gaps remain general division/reciprocals, inverse/GCD/Bezout correctness, Montgomery
canonical bounds and parameter construction, larger multiplication and mixed signed products,
and optional encoding/array integrations. CT/VT equivalence is agreement rather than an
independent mathematical specification. Concrete inverses and fixed-modulus u8/u16 proofs
retain their stated domains. Whole-library correctness and constant-time execution remain
unproved.

## Upstream refresh: 2026-10-24

Fetched upstream `f8d9dfc` (v0.7.6) and merged it as `9b74396`. All existing uncommitted
proof work was restored; before/after binary patches differ only in the expected base-file
hash for the upstream-added inverse tests. The stash and scratch backups are retained.
Local changes in this refresh are proofs and documentation; production changes come from upstream.

### Fixed inverse regression restored to the library suite

PR #1367 removes the final truncation of boxed odd-modulus inverses (finding 1.2).
The returned storage precision is now the maximum of input and modulus precision, because the
internal modulus is widened before normalization. Equal-precision callers retain their width.

The original `regression_boxed_inverse_wider_modulus` has moved from the unresolved integration
regressions into `boxed_invert_odd_mod_ct_wider_modulus`, with its exact domain and assertions
preserved. A second harness checks the same property through `invert_odd_mod_vartime`.
Both require successful inversion, the full expected value and modulus precision. Extra assertions
check the constructor precisions. The four existing `invert_mod2k` harnesses are unchanged.

For W=Word::BITS, the concrete input is 3 in one-word storage, the modulus is
`2^(2W-1)-1` in two-word storage, and the expected inverse is `WideWord::MAX / 3`.
Indeed, three times that value is `2m+1`. This is input32/modulus64 on forced32 and
input64/modulus128 on 64-bit words. These are concrete regression proofs, not arbitrary-input
or arbitrary-modulus inversion proofs. The native matrix below also checks input64/modulus128
with forced32; that larger forced32 case is not claimed as formally proved.

### Final targeted Kani results

| Word width | CT inverse | VT inverse | Checks per harness |
|---|---:|---:|---:|
| 64 bits | PASS, 156.50 s | PASS, 141.98 s | 3817 |
| 32 bits | PASS, 105.15 s | PASS, 105.80 s | 3827 |

CaDiCaL verifies both harnesses on both widths. The original unwind bound of 300 was reduced
to 70 without changing inputs or assertions; all unwinding and other default checks remain
enabled. The 64-bit proofs exceed the earlier 120-second budget but pass with a 300-second
per-harness budget. Forced32 used 180 seconds. No stubs, unproved contracts, safety-only mode
or disabled checks are used.

A reproducible 64-bit command is:
```sh
RUSTFLAGS='--cfg miri' cargo kani --lib --features alloc --harness boxed_invert_odd_mod_ --harness-timeout 300s -j2 -Z unstable-options
```
Add `--cfg cpubits="32"` to RUSTFLAGS for forced32. This selects 32-bit arithmetic words
on a 64-bit pointer host. Portable `cmov` remains selected by `--cfg miri`.

### Current cumulative status

| Word width | Library proofs passed | Unproved timeouts | Native tests passed |
|---|---:|---:|---:|
| 64 bits | 312 / 322 | 10 | 903 |
| 32 bits | 313 / 323 | 10 | 899 |

The previous verified results and ten old timeout obligations per width carry forward from
the seventh-round ledger. Only these two added library harnesses were verified in this refresh;
this is **not a fresh full-library Kani run or a percentage of functions proved**.
Fresh inventories retain every previous library harness and add the CT/VT pair. The unresolved
integration suite now has **41 harnesses**, after promoting the fixed regression. It was not
rerun in full. The unchanged limb bit-restriction control was rerun on both widths and still
fails exactly its intended assertion. No new runtime finding or library counterexample was found
in the targeted checks. Older tables below are historical.

### Native before/after validation and evidence

Archived production snapshots `946a433` and `9b74396` were checked with one unchanged native
harness in debug/release on both word widths. All 16 old wide-modulus cases (inputs 2 and 3,
CT/VT) reach and fail the exact inverse assertion; all 16 patched counterparts pass.
Forty baseline controls and their forty patched counterparts also pass: reverse mixed precision,
equal precision, zero/non-coprime rejection and boxed Montgomery inversion through both APIs.
Thus all 112 checks have usable assertion-level evidence. This does not prove general inversion
correctness or constant-time execution.

Fresh all-feature native suites pass, including doctests: 903/899 tests. Formatting and
whitespace checks pass. No extra runtime fix was added locally.

- Final Kani reports: `/tmp/kani-upstream-20261024-inverse-original64-cadical300.json` and
  `/tmp/kani-upstream-20261024-inverse-original32-cadical.json`, with matching logs.
- Inventories: `/tmp/kani-upstream-20261024-inventory{64,32}.json`.
- Delta ledger and carried-forward counts: `/tmp/kani-upstream-20261024-ledger.json`;
  earlier per-harness evidence remains in `/tmp/kani-round7-ledger{64,32}.json`.
- Native matrix: `/private/tmp/kani-upstream-20261024-native-validation/run-1/results.json`
  and sibling logs; unchanged harness SHA-256:
  `661a742e7b0d5cec65036dc18dd56ab7eb3d65d2c39d5ebfff4800a3a3a27f98`.
- Native suites: `/tmp/kani-upstream-20261024-native-all{64,32}.log`.
- Unresolved regression controls:
  `/tmp/kani-upstream-20261024-unresolved-control{64,32}.json`.

Earlier 120-second 64-bit timeout reports are retained as
`inverse-original64-cadical.json`. Discarded fixed64/128 experiments used a larger forced32
domain; their bounded Z3 reports (`inverse{64,32}-z3-bounded.json`) contain backend exit code 6
with no property results, not counterexamples or successful proofs. The first misconfigured
Z3-wrapper invocation (`inverse{64,32}-z3.json`) is an invocation error. Interrupted initial
runs are likewise not counted as results. All these names have the same
`/tmp/kani-upstream-20261024-` prefix.

## Seventh expansion: 2026-10-23

The upstream baseline remains `946a433`. Three subagents worked on boxed division, boxed
square roots and the checked-division timeouts. Root added wrapper, inversion and modular
arithmetic coverage. This round adds **24 library harnesses per word configuration**:
**23 pass on both widths**, and one new variable-time inverse obligation times out on both.
The existing `checked_eq_select_native` proof also has stronger assertions. Only proof code
and documentation change.

### Current cumulative results

| Word width | Library proofs passed | Unproved timeouts | Library counterexamples | Native tests passed |
|---|---:|---:|---:|---:|
| 64 bits | 310 / 320 | 10 | 0 | 901 |
| 32 bits | 311 / 321 | 10 | 0 | 897 |

These combine the sixth-round ledger with final targeted checks of every new/changed harness.
They are **not a fresh full-library Kani run or a percentage of functions proved**.
Inventories `/tmp/kani-round7-inventory{64,32}.json` retain every previous harness name and add
24 each. No old full-domain proof was narrowed. The separate regression suite contains
**42 known-failing harnesses**: the previous 41 plus the boxed special-modulus regression below.
Older tables are historical.

The nine old timeouts per width remain. Eight are common:
`checked_div_absent_propagation`, `checked_div_option_composition`,
`classic_bingcd_ct_matches_vartime`, `u32_rem_correct_for_bit_precisions`,
`boxed_gcd_vartime_zero_narrow_rhs`, `boxed_gcd_vartime_zero_wide_rhs`,
`uint_div_rem_limb_high_bit_u128_correct` and `uint_div_rem_limb_one_u128_correct`.
The ninth is `sqrt_rem_word_correct` for 64-bit words, or
`montgomery_multiply_inner_matches_textbook` for 32-bit words. The new tenth obligation on
both widths is `boxed_invert_mod2k_vartime_out_of_range`.

### New coverage and stronger assertions

- **Nine boxed division proofs:** reciprocal limb division for every u64 dividend and divisor
  `2^(Word::BITS-1)`; CT/VT quotient/remainder for every u128 dividend modulo `2^127`;
  mixed 64/128-bit division modulo `2^64`; remainder-only APIs modulo `2^127`; exact division
  of every u128 dividend by 64-bit powers of two (all shifts 0..63), and every u64 dividend by
  128-bit powers of two (all shifts 0..127); every u16 dividend divided by a 128-bit-stored 3;
  checked division by zero or `2^63`; and division/remainder operators and assignments modulo
  `2^63`. Exactness/presence flags, successful values and quotient/remainder precision are
  checked independently. Domains are identical across word widths.
- **Three boxed square-root proofs:** every u16 input in one-word storage through CT floor,
  wrapping/deprecated, checked and nonzero-wrapper APIs. Native `u16::isqrt` checks the root;
  checked presence is equivalent to being a perfect square. Result precision and successful
  nonzero wrappers are checked. Variable-time square roots are outside these new proofs.
- **Three wrapper/identity proofs:** every u128 through borrowed/owned `NonZero` and `Odd`
  conversions, default-to-one helpers, `one_like`, `set_one` and `set_zero`. Presence, values,
  precision and wrapper invariants are checked, including valid raw payloads of absent
  `CtOption` results. The absent odd payload is not assumed to equal one.
- **Four inverse obligations:** every u64 input modulo 256 and modulo 1 through CT/VT APIs;
  and rejection of every u32 exponent greater than 64 through each API separately. Modulo 256,
  existence iff odd, canonical range and the native modular-product identity specify the unique
  inverse. Three harnesses pass; the full-domain VT rejection harness remains unproved.
- **Five modular/multiply-accumulate proofs:** every 64-bit addend/multiplicand with unrestricted
  Word multiplier and incoming carry, checking low 64 bits and complete outgoing carry;
  every reduced u64 pair modulo `2^64-1`; arbitrary Word operands in two-word storage at
  special parameter zero; every u8 pair modulo 251 in 64-bit storage; and every u16 square
  modulo 65521 through CT/VT square APIs. Every proof also checks result precision.
- **Stronger existing checked-selection proof:** selection now checks the raw payload even when
  absent. The `CtOption -> Checked -> CtOption` round trip checks raw payload and validity.
  All 126 checks pass on both widths.

### Timeout work and proof limits

The full u64 Mersenne multiplication proof uses `2^64 == 1 (mod 2^64-1)` to replace a costly
native remainder oracle with an exact fold and at most one subtraction. Reduced inputs ensure
the folded value is less than twice the modulus. On forced32, an exact native radix-2^32 product
reference also avoids the large multiplication circuit. The same reference makes the full
multiply-accumulate proof tractable. No operand bits or carry bits are dropped. Intermediate
bounds are the same as the sixth-round product reference: for B=2^32,
`p01 <= B²-B-1`, `p10 <= B²-B` and `p11 <= B²-2`, all within u64.
Final Mersenne checks pass in 24.16 seconds (64-bit Z3) and 23.63 seconds (forced32 CaDiCaL).
The final multiply-accumulate checks pass in 2.62/17.10 seconds respectively.

The new CT and VT out-of-range inverse obligations were split after their conjunction timed out.
CT passes for every `k > 64`. VT still reaches 120 seconds with an arbitrary u64 scalar
constructor and an assumption identical to the callee's early-return guard; the asserted
precision preserves exactly the original domain. Symbolic heap processing of an infeasible
dynamic-length branch is a hypothesis, not a confirmed diagnosis. No counterexample is reported.
The new VT obligation is retained, rather than restricting the exponent or disabling checks.

Old checked-division experiments using Z3, arithmetic refinement and simpler divisor cases still
timed out. An equivalent Word-sized square-root reconstruction also timed out with both solvers
on 64-bit words. The original checked-division and full-word square-root obligations are restored.
No old timeout was resolved this round. No stubs, unproved contracts or disabled checks were added.

Larger new boxed modular-multiplication domains and combined VT square-root experiments timed
out; they were discarded before selecting the explicit u8/u16 domains above. New grouped
full-u128 boxed division cases also exceeded the budget. None replaces an old broader proof.
Arbitrary moduli, arbitrary heap lengths and arbitrary limb-count arithmetic remain outside this
new coverage. Two forced32 exact-division cases take about 104 and 117 seconds under a
120-second budget, so their runtimes remain sensitive to the solver and host.

### New finding and evidence

Section 2 of `FINDINGS.md` records the missing nonzero-`c` precondition in one-limb
special-modulus multiplication. Fixed and boxed one-limb `3 * 7` with `c = 0` panic in native
debug/release on both word widths; two-limb controls return 21. This is an API/documentation
precondition mismatch. If zero is unsupported, documenting `c != 0` is sufficient; no
application exploit or memory-safety failure is established.

The new boxed desired-property regression fails exactly the `NonZero<Limb>::new_unwrap`
assertion on both widths. A fixed one-limb Kani attempt caused a compiler internal error in
`Uint<1>::from_wide_word` and was removed; its native reproducer remains. Production code is
unchanged, so the new regression is intentionally failing.

Raw evidence is retained, including failed attempts; final per-harness ledgers identify the
successful source export rather than treating a mixed-result batch as fully successful:

- Cumulative ledgers: `/tmp/kani-round7-ledger{64,32}.json`; inventories:
  `/tmp/kani-round7-inventory{64,32}.json`.
- Checked selection: `/tmp/kani-round7-checked{64,32}-final-payload.json`.
- Division: `/tmp/kani-round7-boxdiv64-group.json` (seven retained successes),
  `/tmp/kani-round7-boxdiv64-final-extra.json` (two), and
  `/tmp/kani-round7-boxdiv32-final.json` (all nine).
- Square roots: `/tmp/kani-round7-sqrt{64,32}-boxed-ct-u16.json`.
- Wrappers/inverses: successful entries in
  `/tmp/kani-round7-invert-wrappers{64,32}-cadical.json` and
  `/tmp/kani-round7-invert-split64.json`; final VT timeouts:
  `/tmp/kani-round7-invert-vt-scalar{64,32}.json`.
- Modular arithmetic: `/tmp/kani-round7-mersenne-final{64,32}.json`,
  successful MAC entries in `/tmp/kani-round7-product-helper{64,32}.json`,
  `/tmp/kani-round7-mulmod8-64.json`, successful square entry in
  `/tmp/kani-round7-mulmod64-cadical.json`, and
  `/tmp/kani-round7-modular32-cadical.json`. The two-word zero-parameter controls are in
  the wrapper/inverse batches.
- New regressions: `/tmp/kani-round7-reg{64,32}.json`; native controls:
  `/tmp/kani-round7-special-zero{64,32}-{debug,release}.log`.
- Fresh all-feature native suites: `/tmp/kani-round7-native-all{64,32}.log`; 901/897 tests,
  including doctests, pass. Formatting and whitespace checks also pass.

Forced32 selects 32-bit arithmetic words on this 64-bit pointer host. Kani uses portable
`cmov` under `--cfg miri`. A timeout or backend failure, even with zero failed checks,
is not a successful verification. Earlier tool limitations still apply.

## Sixth expansion: 2026-10-23

The upstream baseline remains `946a433`. Three subagents worked on boxed multiplication,
Montgomery addition/subtraction and the signed linear-combination timeout. Root added boxed
concatenating/conditional addition and resize coverage. This round adds **23 library harnesses
per word configuration**, all verified on both. Only proof code and documentation change.

### Current cumulative results

| Word width | Library proofs passed | Unproved timeouts | Library counterexamples | Native tests passed |
|---|---:|---:|---:|---:|
| 64 bits | 287 / 296 | 9 | 0 | 901 |
| 32 bits | 288 / 297 | 9 | 0 | 897 |

These combine the fifth-round ledger with final targeted checks of every new/changed harness
and the existing boxed resize proof. They are **not a fresh full-library run or a percentage of
functions proved**. Inventories `/tmp/kani-round6-inventory{64,32}.json` retain all previous names
and add 23 each. The separate regression suite now contains **41 known-failing harnesses**:
37 retained, plus four new regressions verified to fail only their intended assertions on both
widths. Older tables below are historical.

Eight library obligations remain unproved on both widths: `checked_div_absent_propagation`,
`checked_div_option_composition`, `classic_bingcd_ct_matches_vartime`,
`u32_rem_correct_for_bit_precisions`, `boxed_gcd_vartime_zero_narrow_rhs`,
`boxed_gcd_vartime_zero_wide_rhs`, `uint_div_rem_limb_high_bit_u128_correct` and
`uint_div_rem_limb_one_u128_correct`. The ninth is `sqrt_rem_word_correct` for 64-bit words,
or `montgomery_multiply_inner_matches_textbook` for 32-bit words. The latter was retried unchanged
with CaDiCaL this round and still timed out at 120 seconds. Other old timeout results carry forward.

### Timeout work without weakening the old proof

`safegcd_signed_lincomb_u64_correct` now passes on forced32 with CaDiCaL in 35.34 seconds.
Every original u64 magnitude, i64 coefficient and sign input remains arbitrary. Absolute-value,
full concatenated magnitude and nonzero-result sign assertions are retained; zero may have either
sign. No assumptions, stubs, unproved contracts or disabled checks are added.

The native product oracle uses exact radix-2^32 partial products with a carry after each step,
instead of one large multiplication circuit. For B=2^32, the intermediate upper bounds are
`p01 <= B²-B-1`, `p10 <= B²-B` and `p11 <= B²-2`, so each fits u64. The assembled value is exactly
the full u64 product. Sharing the implementation's carry order helps the solver while keeping
the reference in native arithmetic. The 64-bit-word proof also passes again, with Z3 in 103.66
seconds; CaDiCaL exceeded 120 seconds there. This is close to the configured Z3 limit and may
remain sensitive to solver/host changes. Final source attributes select the measured backends.

### New arithmetic coverage

- **Ten boxed multiplication/square proofs:** every u64 pair or u64 square, wide/concatenated
  products, wrapped results, overflow flags, checked presence and successful values, saturation,
  operators under the native fit condition, and a full u128 RHS with 64-bit retained output.
  A separate arbitrary-Word product checks the complete next-word carry, including nonzero
  carry. Every result also checks precision. Native full-product oracles retain all bits;
  forced32 uses exact partial products rather than narrowing operands. Larger limb counts,
  Karatsuba and panicking operator paths are outside this new coverage.
- **Six boxed addition proofs:** concatenating addition at 64/128 bits and both mixed directions,
  including the extra carry limb; conditional addition at 64/128 bits and 128-bit LHS with a
  64-bit RHS, for both choices. Numeric results, carry and precision are checked independently.
  The 128-bit oracle preserves the 129th sum bit rather than truncating it into u128.
- **Three Montgomery proofs:** add, double and subtract for all reduced u64 operands and every
  odd u64 modulus, including modulus 1. Native u128/piecewise oracles check canonical output,
  exact result and the input sum carry or subtraction borrow. These do not prove arbitrary
  limb-count Montgomery multiplication.
- **Four boxed precision proofs:** arbitrary u128 values through checked and owned/in-place
  unchecked resize at requests 0, 63, 64, 65, 127 and 128; arbitrary u32 bit-limit queries; and
  zero/one/max constructor boundaries 0, 1, W-1, W, W+1, 64, 127 and 128. The existing
  owned/borrowed 128-to-64 resize proof was also rechecked. Checked resize uses the exact
  requested bit limit; unchecked truncation follows allocated word rounding. The owned
  zero-request behavior is explicitly documented as current behavior, with separate regressions
  for the ownership and wrapper problems below.

The new constructor experiment over every request 0..128 timed out with symbolic lengths and
with an exhaustive concrete loop. The retained boundary proof passes in 6.54/7.84 seconds
(64/32-bit words); it does **not** prove all 129 requests. No existing harness was replaced by
this restricted new coverage. Heap-heavy conjunctions were separated where needed, and measured
CaDiCaL/Z3 choices keep the final ten multiplication harnesses passing on both widths.

### New findings and evidence

`FINDINGS.md` now records zero-request resize ownership disagreement and `Odd` invariant loss
(1.20), positive-width truncation returning zero inside `NonZero` (1.21), and the `max(0)`
documentation mismatch (section 2). `from_words_with_precision(_, 0)` is another confirmed
empty-storage constructor under existing 1.9. Native debug/release probes confirm all cases on
both word widths. Four new desired-property regressions fail exactly five intended assertions
per width: one maximum assertion, one ownership equality, one oddness assertion and two
independently reachable nonzero assertions. There are no accompanying memory/unwind/backend
failures. No application-level exploit or memory-safety failure is established; no production
fix is included.

Raw evidence is retained without rewriting failed attempts:

- Cumulative per-harness status and source export: `/tmp/kani-round6-ledger{64,32}.json`.
- Root: `/tmp/kani-round6-add64.json`, `/tmp/kani-round6-resize64.json`,
  `/tmp/kani-round6-root32.json`, `/tmp/kani-round6-constructor-boundaries{64,32}.json`.
  The old `boxed_precision_constructors_native` timeout in `root32` is a discarded new
  experiment, not a successful final harness.
- Multiplication: `/tmp/kani-round6-boxmul64{,-cadical,-carry}.json` and
  `/tmp/kani-round6-boxmul32{,-z3,-carry}.json`. Successful retries supersede the initial
  incomplete cases individually; neither initial nine-harness export alone proves all nine.
- Modular and signed: `/tmp/kani-round6-modular{64,32}.json`,
  `/tmp/kani-round6-safegcd{64,32}.json`; remaining Montgomery timeout:
  `/tmp/kani-round6-modular-old32.json`.
- New regressions: `/tmp/kani-round6-max-reg{64,32}.json`, `/tmp/kani-round6-reg{64,32}.json`.
  Native resize probes: `/tmp/kani-round6-resize-native{64,32}-{debug,release}.log`.
- Fresh all-feature native suites: `/tmp/kani-round6-native-all{64,32}.log`; 901/897 tests,
  including doctests, pass. Formatting and whitespace checks also pass.

Forced32 selects 32-bit arithmetic words on this 64-bit host; it is not a 32-bit pointer target.
Kani continues to use portable `cmov` under `--cfg miri`. Tool limitations and older results
described below still apply.

## Fifth expansion: 2026-10-23

The upstream baseline remains `946a433`. Three subagents worked on signed multiplication,
limb division and boxed addition/subtraction. Root added comparison and conversion proofs.
This round adds **41 library harnesses per word configuration**, all verified on both.
Only proof code and documentation change; the 37 known-failing regressions are unchanged.

### Current cumulative results

| Word width | Library proofs passed | Unproved timeouts | Library counterexamples | Native tests passed |
|---|---:|---:|---:|---:|
| 64 bits | 264 / 273 | 9 | 0 | 901 |
| 32 bits | 264 / 274 | 10 | 0 | 897 |

These combine earlier library results with final targeted checks of every new/changed harness.
They are not a fresh full-library run or a percentage of functions proved. Fresh inventories
`/tmp/kani-round5-inventory{64,32}.json` contain all previous names plus 41 additions each.
Known-failing regressions are separate from the library totals; none was removed or rerun here.

Both word widths retain eight unproved obligations: `checked_div_absent_propagation`,
`checked_div_option_composition`, `classic_bingcd_ct_matches_vartime`,
`u32_rem_correct_for_bit_precisions`, `boxed_gcd_vartime_zero_narrow_rhs`,
`boxed_gcd_vartime_zero_wide_rhs`, `uint_div_rem_limb_high_bit_u128_correct` and
`uint_div_rem_limb_one_u128_correct`. The 64-bit-word build also retains
`sqrt_rem_word_correct`; 32-bit words retain `montgomery_multiply_inner_matches_textbook` and
`safegcd_signed_lincomb_u64_correct`. Unchanged results outside this round's targeted checks
are carried forward. All older tables below are historical.

### Timeout work and assertion preservation

The original full-domain `int_mul_i64_correct` now passes with forced 32-bit words using
CaDiCaL (47.49 seconds in the final check). Its native magnitude assertions, concatenated and
split product, sign, checked result and saturation assertions are unchanged. The existing exact
native 32×32 partial-product reference is retained. The solver attribute selects CaDiCaL for
32-bit words and Z3 otherwise; the full 64-bit-word proof also passes again.

The two limb-division conjunctions are split without dropping their properties:

| Fixed divisor | Quotient and remainder from both div/rem APIs | Both remainder-only APIs |
|---|---|---|
| 1 | `uint_div_rem_limb_one_u128_correct` | `uint_rem_limb_one_u128_correct` |
| `1 << (Word::BITS - 1)` | `uint_div_rem_limb_high_bit_u128_correct` | `uint_rem_limb_high_bit_u128_correct` |

Every harness still quantifies every u128 dividend. Both remainder-only proofs now pass on both
word widths using Z3. The quotient/remainder halves remain separate obligations; passing the
remainder APIs does not prove the quotient APIs. Their 64-bit-word Z3 rechecks still reach the
110-second solver limit with zero failed checks, which is inconclusive rather than a counterexample.
Fresh forced32 Z3 rechecks reach the same limit for both quotient/remainder halves.
Both halves also time out at 120 seconds with their default CaDiCaL backend on both word widths.

The original `u32_rem_correct_for_bit_precisions` also remains unproved. An unchanged Z3 run,
then an equivalent Euclidean witness formulation with Z3 and CaDiCaL, all exceeded their budgets.
The original native `%` assertion, every dividend and all 128 divisors are restored exactly.
No unproved contracts, stubs or disabled checks are introduced.

### New arithmetic coverage

Thirty boxed addition/subtraction proofs cover every 64/128-bit operand pair, including both
mixed-precision directions. Allocating carry/borrow operations use the maximum precision;
wrapping/checked/overflowing operations preserve the left operand's precision. Assertions check
numeric values and precision separately. Incoming addition carry is an unrestricted Word;
incoming subtraction borrow contributes its MSB and outgoing borrow is a full-word mask.
The 128-bit native oracle accounts for both possible overflow events, including carry-out 2.
In-place carrying/borrowing calls are checked whenever the right operand is no wider than the left.

Other assertions cover overflow/underflow flags, checked presence and successful values,
wrapping methods and assignments, and owned/borrowed left-operand operators with borrowed RHS.
Panicking operators are evaluated under independently computed native success conditions.
Separate API groups avoid too many live heap objects. Checking `CtOption::is_some` and guarded
`unwrap` preserves the successful-result obligations while avoiding costly heap/enum selection
from conversion to `Option<BoxedUint>`. All 30 pass with default CaDiCaL on both word widths.
Concatenating/conditional addition and primitive RHS overloads remain outside these new proofs.

One new mixed signed/unsigned multiplication proof checks both operand-order APIs, concatenated
products, widened magnitudes/signs and checked results. Its 64-bit-word domain is every
`i64 × u64`; its forced-32-bit-word domain is every `i32 × u32`, stored in I64/U64. The latter
always fits I64, so it does not prove mixed checked-overflow behavior at that configuration.
The full forced-32 mixed domains timed out and were removed; this limit is explicit in source.
This restricted new proof does not replace the unrestricted signed proof resolved above.

The new direct `div3by2_two_word_high_bit_divisor_correct` checks every two-word dividend at
divisor B²/2, with the third dividend word fixed to zero. It covers quotient 0/1, correction,
remainder and adjusted reciprocal. It does not cover arbitrary three-word dividends or divisors.

### Comparison, wrappers and conversions

Two new `UintRef` proofs cover every pair of active lengths 0..2 over arbitrary two-word storage:
constant/variable-time ordering, trait comparisons, zero/nonzero predicates and borrowed NonZero
wrappers. Oddness, borrowed Odd wrappers and `lowest_u64` are checked on nonempty views because
they access the first limb. Numeric oracles fold native words and preserve every active bit.

Five new boxed conversion proofs check native scalar constructors, U128 owned/borrowed/view
conversions and every odd u128 through owned/borrowed Odd conversions. Owned limb/boxed-limb
containers cover lengths 0..4 and empty normalization. The unsafe Vec<Word> ownership conversion
covers all lengths 0..4 crossed with requested spare capacities 0..2. Each output limb, length
and precision is asserted, including 256-bit values on 64-bit words without a truncating oracle.
Borrowed slice/UintRef conversions here cover nonempty U128-shaped inputs only.

Symbolic container length timed out; making length concrete resolved owned-limb conversion.
Word vectors also required concrete capacity: separate calls preserve all 15 size combinations
and every word value, passing in 38.66/28.28 seconds with 64/32-bit words. These are observed
verification times, not benchmarks. Source comments explain the representation choice.

Independent read-only review checked new oracles, overflow bounds, input domains and the division
assertion split. The findings document extends the existing borrow-word documentation trap with
a native BoxedUint reproducer; no new independent production-bug root cause was found.
Review also replaced the saturated-division oracle's shared `word::join` call with native
shifts/OR. The updated proof passes its default CaDiCaL backend on both word widths.

### Verification evidence

Signed exports: `/tmp/kani-round5-signed64.json`, `/tmp/kani-round5-signed32.json` (original
full signed proof, CaDiCaL), and `/tmp/kani-round5-signed32-z3.json` (three other signed proofs).
Boxed arithmetic: `/tmp/kani-round5-boxedarith{64,32}.json`, 30 successes per configuration.
Division: `/tmp/kani-round5-div{64,32}.json`, seven successes per configuration, and
`/tmp/kani-round5-div-all64.json` plus `/tmp/kani-round5-div-old32.json` for the two original
quotient/remainder Z3 timeouts. Their fresh CaDiCaL timeouts are in
`/tmp/kani-round5-div-cadical{64,32}.json`. Default-backend checks of the saturated-oracle cleanup are in
`/tmp/kani-round5-div-saturated{64,32}.json`. The passing
division subsets also rerun four existing proofs. Fresh native all-feature logs are
`/tmp/kani-round5-native{64,32}.log`: all 901/897 tests pass.

Comparison: `/tmp/kani-round5-ref64.json`; four successful conversions are in
`/tmp/kani-round5-conversions64.json`. Its failed symbolic-capacity WordVec result is superseded
by `/tmp/kani-round5-wordvec64.json` for the final all-concrete source. Forced32 comparison and
four conversions are in `/tmp/kani-round5-root32.json`, with the final WordVec check in
`/tmp/kani-round5-wordvec32.json`. Borrow documentation evidence is
`/tmp/kani-round5-boxed-borrow.rs` and `.log`, using the default-word native dev build.
The previous solver restrictions still apply: portable cmov, host 64-bit pointers even with
32-bit words, homogeneous Z3 wrapper calls, default safety/unwinding checks, no Bitwuzla.

## Fourth expansion: 2026-10-23

The baseline remains upstream `946a433`. The work is split across independent proof areas:
shared `UintRef` bit/shift operations, square-root decomposition, division/GCD, Montgomery
reduction and boxed bitwise operators. Only proof code and findings documentation change.

### Current cumulative results

This round adds **29 library harnesses per word configuration**, all verified on both.

| Word width | Library proofs passed | Unproved timeouts | Library counterexamples | Native tests passed |
|---|---:|---:|---:|---:|
| 64 bits | 223 / 232 | 9 | 0 | 901 |
| 32 bits | 222 / 233 | 11 | 0 | 897 |

These combine previous library results with final targeted checks of every new or changed
harness. They are not a fresh full-library run or a percentage of functions proved. The known
failing regression suite is counted separately. Fresh library inventories are
`/tmp/kani-round4-inventory{64,32}.json`; no earlier harness was removed.

No original timeout was resolved this round. Both configurations retain these eight:
`checked_div_absent_propagation`, `checked_div_option_composition`,
`classic_bingcd_ct_matches_vartime`, `u32_rem_correct_for_bit_precisions`,
`boxed_gcd_vartime_zero_narrow_rhs`, `boxed_gcd_vartime_zero_wide_rhs`,
`uint_div_rem_limb_high_bit_u128_correct` and `uint_div_rem_limb_one_u128_correct`.
64-bit words also retain `sqrt_rem_word_correct`; 32-bit words retain `int_mul_i64_correct`,
`montgomery_multiply_inner_matches_textbook` and `safegcd_signed_lincomb_u64_correct`.
Except for the two boxed GCD rechecks described below, unchanged timeout results are carried
forward from earlier rounds. The older tables below are historical.

### Shared bit and shift coverage

Twelve new harnesses pass with both word widths. They use independent native `WideWord`
oracles over every bit of two-word backing arrays, and active prefix lengths 0..2. Assertions
check numeric results, carry alignment and storage outside the borrowed prefix. The scopes are:

- Bit length/counts, bit queries, setters and restriction, including arbitrary u32 indices.
  `bits_vartime` is checked on nonempty views; the empty failure is retained separately.
- Equality/ordering across independent mixed lengths and conditional assignment with zero
  extension whenever the destination is wide enough.
- Constant-time shifts for nonempty views and every u32 shift, including the overflow flag and
  unchanged-overflow behavior. The empty-view contract failures are separate regressions.
- Variable-time in-place/out-of-place shifts, conditional/unconditional whole-limb shifts and
  direct single-bit shifts, including empty views. Out-of-place buffers are explicitly zeroed.
- Limb shifts for every shift 0..Word::BITS−1, arbitrary incoming carry words and both choices.
  The native oracle records OR insertion into the appropriate word, not carry addition. Shift
  zero ignores incoming carry; left/right outgoing bits have different alignments.

Five isolated new `regression_uint_ref_empty_*` harnesses fail the expected checks on both word
widths. `FINDINGS` 1.19 records the exact API guarantees and the reachable parsed-zero consumer
from 1.9. Native debug/release probes cover both word widths. The shl1 documentation's LSB/MSB
mismatch is also confirmed. These extend the empty-representation findings; they do not introduce
a second parser root cause. The regression suite now has 37 harnesses.

### Square-root decomposition

Two additional full-Word harnesses pass on both word widths with CaDiCaL: root/remainder bounds,
and an explicit zero base plus the complete radix-four recurrence. There are no input assumptions.
For v>0, p=v>>2<v and d=v&3. If the prefix result is (s,r), the verified recurrence gives

```
c = 4r + d,  b = (c >= 4s + 1)
q = 2s + b,  r' = c - b(4s + 1)
```

A mathematical induction composes the verified base and recurrence:
q²+r' = 4(s²+r)+d = v. With 0≤r≤2s, the two branches give 0≤r'≤2q.
Thus these machine-verified properties support a complete human induction argument for the
desired square-root specification. The induction composition itself is not a new Kani theorem;
the original `sqrt_rem_word_correct` harness is unchanged and still times out with 64-bit words.
No original obligation is recorded as passing merely because its simpler bounds now pass.

All recurrence arithmetic is widened: even unrestricted Word prefix outputs give
4r+d ≤ 2^(W+2)−1 < 2^(2W) for W=32/64, so this argument does not conceal modular wraparound.
An isolated recursive-contract experiment also timed out. No contract attributes, unproved
stubs or new global compiler flags entered the library.

### Division/GCD and Montgomery boundaries

Four new division/GCD harnesses pass with both word widths using Z3. Two check every Word
dividend and both input-validity flags at the fixed divisors `1 << (Word::BITS - 1)` and
`Word::MAX`. They cover all four owned/borrowed `Checked` division forms against native
`Option<Word>` results. Two check `Odd<BoxedUint>::gcd_vartime(x, 0)` over every odd 64/128-bit
input with the opposite zero precision. Both the numeric result and the 128-bit result precision
are asserted. Odd inputs are built by setting bit zero in native words before construction.

Three new single-limb Montgomery harnesses pass with both word widths using Z3 at the fixed
modulus `m = Word::MAX = R - 1`. The raw reduction proof checks every pair of lower/upper words,
including the complete carry. The normalized proof checks every wide `T < mR` and canonical
output. Retrieval checks the inner function and wrapper for every `x < m`, with the inner output
explicitly initialized to zero. Independent u128 oracles use `R ≡ 1 (mod m)` and `-m^-1 = 1`.
The 32-bit configuration covers a 32-bit modulus here; it does not represent a two-limb U64 proof.

No original timeout is discharged by these restricted boundary proofs. The two original boxed
GCD zero-right-operand proofs still time out on a fresh 64-bit-word Z3 run. The original
arbitrary-divisor Checked and full U64 Montgomery multiplication assertions, references and input
domains remain intact. The unchanged multiplication proof passes again with 64-bit words; its
32-bit-word timeout remains. Failed proof restructurings were removed.

### Boxed bitwise coverage and proof simplification

Eight new harnesses pass with both word widths using CaDiCaL. AND/OR/XOR cover every
`(u64, u128)` operand pair in both orders, against native numeric oracles. Assertions check the
128-bit result precision separately from its value. They cover inherent, checked and wrapping
methods; all four operator ownership forms; owned/borrowed assignments; and representative
`Wrapping` binary/assignment forms. NOT covers every 64/128-bit input with a precision mask,
including owned and both `Wrapping` forms. The limb-AND proof checks an arbitrary Word mask
repeated over every limb at both precisions.

An initial proof loop over differently sized heap values timed out at 120 seconds. Separate
calls with concrete lengths preserve the complete domains and assertions; the same limb-AND
property then passed in about 1.5 seconds with 64-bit words. These observed times are not
benchmarks. Comments explain the representation choice. The OR assignment proofs exercise the
upstream widening fix, including nonzero high RHS limbs, and retain the upstream native test.

Independent read-only review checked the new bit/shift, boxed bitwise, reduction and square-root
oracles, bounds, domains and precision assertions. No blocking proof issue remained. Review
also corrected the exact empty-view API guarantees in `FINDINGS` 1.19.

### Verification evidence

Shared proofs: `/tmp/kani-round4-root64.json`, `/tmp/kani-round4-root-extra64.json`,
`/tmp/kani-round4-root32.json`, and the final extended carry checks
`/tmp/kani-round4-root-carry{64,32}.json`. The extra64/32 files include the five isolated
regressions; exclude those from passing library totals. Square-root exports are
`/tmp/kani-round4-sqrt64.json`, `/tmp/kani-round4-sqrt-base64.json` (final base+recurrence), and
`/tmp/kani-round4-sqrt32.json`. Division/GCD exports are
`/tmp/kani-round4-division-final64.json` and `/tmp/kani-round4-division-final32-clean.json`;
Montgomery exports are `/tmp/kani-round4-reduction{64,32}-final.json`; boxed bitwise exports are
`/tmp/kani-round4-boxedbits{64,32}.json`.
Native all-feature logs are
`/tmp/kani-round4-native-all{64,32}.log`; all 901/897 tests pass. Native empty-view reproducers and
logs are `/private/tmp/kani-round4-native` and `/tmp/kani-round4-native{64,32}-{debug,release}.log`.
Per-harness solver/process restrictions from the third expansion continue to apply.

## Third expansion: 2026-10-23

The upstream baseline remains `946a433`. This round adds direct `UintRef` arithmetic,
signed encoding, partial-byte boxed decoding and Montgomery structural/arithmetic coverage.
It also turns eight earlier code-reading observations into concrete findings: two distinct
`UintRef` failures, `PrimeParams` field assignment, signed decoder input shapes, `NonZero`
modular products, and the three API/documentation traps detailed in `FINDINGS.md` section 2.
No production fixes are included.

### Current cumulative results

This round adds **21 library harnesses per word configuration**, all verified on both.
The complete original unsigned multiplication property also now passes with 32-bit words.

| Word width | Library proofs passed | Unproved timeouts | Library counterexamples | Native tests passed |
|---|---:|---:|---:|---:|
| 64 bits | 194 / 203 | 9 | 0 | 901 |
| 32 bits | 193 / 204 | 11 | 0 | 897 |

These are cumulative results from the previous library runs and final targeted checks of all
new/changed harnesses, not a fresh full-library run or a percentage of functions proved.
Unchanged division/GCD timeouts were last rerun in the second expansion. Both configurations
still have its eight common timeouts; 64-bit words additionally have `sqrt_rem_word_correct`.
The extra 32-bit-word timeouts are `int_mul_i64_correct`,
`montgomery_multiply_inner_matches_textbook`, and `safegcd_signed_lincomb_u64_correct`.
The older tables below are historical.

### Unsigned multiplication timeout resolved

The original full-domain `uint_mul_u64_correct` conjunction is split without losing assertions:

| Property | Current harness | Input domain |
|---|---|---|
| Widened and concatenated product | `uint_mul_u64_correct` | Every `(u64, u64)` |
| Wrapping product | `uint_wrapping_mul_u64_correct` | Every `(u64, u64)` |
| Checked and saturating products | `uint_checked_saturating_mul_u64_correct` | Every `(u64, u64)` |
| Limb product, carry and wrapping helper | `uint_mul_limb_u64_correct` | Every `(u64, Word)` |

All four pass in both word configurations. The limb proof preserves the original 64-bit-word
assertions and adds the two-limb carry chain for 32-bit words. The 32-bit-word oracle expands
the exact native product into four native 32×32 partial products. Each intermediate fits u128;
it does not call the library's multiply implementation. No operand assumptions are added.
Separating the overflow circuit lets CaDiCaL prove it with forced 32-bit words; Z3 still times out
on that circuit. Z3 proves it much faster on the default 64-bit-word build. The attributes encode
this measured choice. Signed wide/overflow splits and extra high-half overflow branches did not
help and were removed; the original signed obligations remain.

### Coverage and finding validation

- `UintRef`: all four-word backing buffers and active lengths 0..4, with mixed lengths,
  arbitrary incoming carry/borrow and both conditional choices. Assertions include the carry
  or borrow and untouched backing storage. Word multiplication/squaring cover unrestricted word
  inputs with explicit zero-filled output premises; multiply-add covers an arbitrary two-word
  accumulator and unrestricted limb multiplier/addend/carry. These are bounded storage domains,
  not a general Karatsuba proof.
- Signed encoding: every i128 bit pattern in both byte orders, plus arbitrary bytes and the
  trait's byte-order dispatch. Boxed decoding checks every Word input at precisions 1 and
  Word::BITS−1, including the numeric mask and rounded one-limb storage precision.
- Montgomery: structural selection/assignment/equality across all parameter fields, fixed-form
  raw values and independently chosen valid parameter sets, full-u64 additive arithmetic at
  m=2^64−1, and retrieval/comparison over every residue modulo 257 at 64-bit precision.
  R=2^64≡1 mod 257 on both word widths. These are specified fixed moduli, not arbitrary-modulus
  proofs. Raw structural operations do not imply canonical arithmetic for unreduced inputs.
  Building modulo-257 inputs as u16 values below 257, then widening them to u64, preserves the
  complete residue domain and fixes upper zero bits before symbolic conversion. Boxed retrieval
  also checks numeric value and precision separately. New boxed selection experiments repeatedly
  timed out and were removed; no pre-existing obligation was weakened or removed.
- Nineteen new desired-behavior regressions reproduce unresolved findings, separately from the
  passing library harnesses. The suite now contains 32 Kani regressions; the prior 13 were not
  rerun this round. Each new regression fails exactly one intended assertion or panic check in
  both word configurations, with no tool errors or timeouts. Native debug/release probes distinguish implementation bugs, undocumented
  panics and potentially stale API docs. No application-level exploit is claimed.

Independent read-only review checked native identities and bounds, input domains, oracle
independence and assertion preservation. It corrected the signed short-input claim: padding is
inferred from Uint consistency; smaller-precision truncation is explicitly documented by the
trait. Native all-feature tests pass (901 with 64-bit words; 897 with forced 32-bit words).
Formatting and whitespace checks pass. No Kani, CBMC or Z3 processes remain after final checks.

### Solver commands and evidence

Do not append the Z3 process wrapper to a mixed-solver run. CBMC rejects its combination with
CaDiCaL, and its Z3 SSA conversion fails on several heap/dynamic-slice formulas (`FINDINGS` 4.7).
Run selected CaDiCaL proofs without external SMT options; run selected Z3 proofs separately with
the wrapper and the 120-second harness bound. For example:

```sh
rtk proxy env RUSTFLAGS='--cfg miri --cfg cpubits="32"' cargo kani --features alloc --harness uint_checked_saturating_mul_u64_correct --solver cadical -Z unstable-options --harness-timeout 120s --output-format terse
```

```sh
rtk proxy env RUSTFLAGS='--cfg miri' cargo kani --features alloc --harness uint_mul_limb_u64_correct --solver z3 -Z unstable-options --harness-timeout 120s --output-format terse --cbmc-args --external-smt2-solver ./scripts/kani-z3-timeout
```

Default library invocations omit `--tests`. Isolated unresolved reproducers use
`cargo kani --tests --features alloc --harness regression_`, and are expected to fail until fixed.
No safety, overflow or unwinding checks were disabled, and no unproved stubs were introduced.
The forced-word configuration still uses host 64-bit pointers; Kani uses the portable cmov backend.

Evidence: `/tmp/kani-round3-root{64,32}.json`, `/tmp/kani-round3-root-regressions{64,32}.json`,
`/tmp/kani-round3-nonzero{64,32}.json`, `target/kani-round3-refarith{64,32}/{dynamic,fixed,regression}-results.json`,
`/tmp/kani-round3-products64.json`, `/tmp/kani-round3-products32-{z3,cadical}.json`, and
`/tmp/kani-round3-modular-verified{64,32}.json`. Modular counterexamples are in
`/tmp/kani-round3-modular-regressions-final{64,32}.json`. The modular 64-bit default export also includes one
historical timeout for the subsequently deleted `boxed_monty_ct_operations_preserve_params`;
all six retained modular harnesses pass. Native logs are
`/tmp/kani-round3-native-normal{64,32}.log`; decoder/wrapping/nonzero native probes are in
`/private/tmp/kani-round3-native` and their debug/release logs under `/tmp/kani-round3-native*`.

## Second expansion: 2026-10-23

The upstream baseline remains `946a433`. This round adds **21 library harnesses with 64-bit
words**, and **22 with 32-bit words** (the latter includes the separate `u32_invert_odd`
implementation). All 21 new harnesses pass with 64-bit words. With 32-bit words, 21 new
harnesses pass; the full signed linear-combination property remains unproved.
The 13 unfixed regression harnesses are unchanged and were not rerun this round.

### Current cumulative results

| Word width | Library proofs passed | Unproved timeouts | Library counterexamples | Native tests passed |
|---|---:|---:|---:|---:|
| 64 bits | 173 / 182 | 9 | 0 | 901 |
| 32 bits | 171 / 183 | 12 | 0 | 897 |

These combine the first full library runs with final targeted checks of every new or changed
harness. Remaining division/GCD timeouts were also rerun after the solver-process cleanup.
This is not a fresh full-library run or a percentage of functions proved. Native tests were
rerun with all features in both word configurations; formatting and whitespace checks pass.
The 32-bit-word configuration still uses the host's 64-bit pointers and portable `cmov`.

### Existing timeouts resolved

- `int_mul_i64_correct`, 64-bit words: independently assert library magnitudes equal native
  `unsigned_abs`, then reuse those checked values in the native product oracle. The wide,
  checked and saturating results keep their full input domains. Wrapping multiplication is
  now a separate full-domain proof, so the original conjunction of properties is retained.
- `uint_square_u64_correct`, 32-bit words: expand the native square into exact native 32-bit
  partial products. The source states the identity and why the solver needs it. All inputs
  and concatenated/widened/wrapping/checked/saturating/alias assertions remain covered.
- `sqrt_rem_word_correct`, 32-bit words: the original width, reconstruction and remainder
  assertions pass with Z3 after resource cleanup. No domain or assertion changed.

### New coverage and assertion review

- Signed squaring: all `i64` values, including MIN; concatenating, wrapping, checked and
  saturating results use the unsigned output bound.
- Division: every valid two-word dividend at divisor `Word::MAX`; reciprocal adjustment and
  the deliberate saturated `div3by2` boundary; zero divisors in all four `Checked` operator
  forms and both operand-validity flags. The saturation proof explicitly distinguishes its
  sentinel remainder from an ordinary Euclidean remainder.
- Safegcd/bingcd: strict bit-length descent for one GCD step; low-tail dependence of a divstep;
  signed-tail extraction, reduced normalization, and full `u64` magnitude / `i64` coefficient
  linear combinations. The latter passes with 64-bit words and times out with 32-bit words.
  These are building blocks, not an end-to-end inversion or GCD proof.
- Boxed encoding: full 128-bit outputs and trimmed encodings; both byte orders at every input
  length 0..20, with exact truncation, checked rejection and output precision. Byte orders are
  separate harnesses to reduce the formula. Review added explicit checked-output precision
  assertions because `BoxedUint` numeric equality ignores precision.
- Unsafe views and slice operations: shared/mutable limb-word casts, copy/conditional copy,
  split and flattened views, write-through behavior, untouched regions, empty cases,
  conditional constants, and resize. Storage sizes are bounded to the stated arrays.
- Primitive arithmetic: every dividend at power-of-two precisions 32..4096; low 16 inverse
  bits for every odd input to each inversion implementation. The inverse property is partial
  output coverage, not proof of the upper bits.

Independent read-only review checked domains, oracle independence, partial-product ranges,
vacuity and property preservation. No blocking findings remain. Unsuccessful unsigned-product
and Montgomery oracle rewrites, square-root input construction, and extra narrow probes were
removed; the original full-domain obligations remain.

### Remaining obligations

Both configurations still time out on:

- `checked_div_option_composition`, `checked_div_absent_propagation`
- `classic_bingcd_ct_matches_vartime`
- `u32_rem_correct_for_bit_precisions`
- `boxed_gcd_vartime_zero_narrow_rhs`, `boxed_gcd_vartime_zero_wide_rhs`
- `uint_div_rem_limb_high_bit_u128_correct`, `uint_div_rem_limb_one_u128_correct`

Additionally, `sqrt_rem_word_correct` times out with 64-bit words. With 32-bit words,
`int_mul_i64_correct`, `uint_mul_u64_correct`, `montgomery_multiply_inner_matches_textbook`,
and the new `safegcd_signed_lincomb_u64_correct` remain unproved. No checks were disabled,
no unproved stubs were introduced, and these obligations were not narrowed to obtain passes.

### Reproduction and solver limits

Kani 0.68.0 / CBMC 6.11.0 remain installed. Bitwuzla remains excluded. Seventeen orphaned Z3
processes from earlier timeouts were confirmed and stopped; earlier timings are not clean
benchmarks. `scripts/kani-z3-timeout` bounds each Z3 process to 110 seconds, including any
child that outlives Kani's timeout. Timeouts and tool errors are unproved results even if the
printed failed-property count is zero. See FINDINGS 4.6/4.7 for the process leak and Z3 encoding
tool errors. The final encoding proofs use CaDiCaL.

Run from the repository root; leave `--solver` unset to honor measured per-harness choices.
For a mixed-solver run, omit the external solver wrapper (see the third expansion and FINDINGS 4.6):

```sh
rtk proxy env RUSTFLAGS='--cfg miri' cargo kani --features alloc -j 2 --output-format terse -Z unstable-options --harness-timeout 120s
```

For selected Z3 harnesses only, append `--cbmc-args --external-smt2-solver ./scripts/kani-z3-timeout`.
For 32-bit words, add `--cfg cpubits="32"` inside `RUSTFLAGS`. The wrapper has a fixed solver
limit; raise it along with the harness budget when deliberately running longer proofs.
Positive and negative multiplication controls verify argument/status forwarding through it.

Research used Kani's official [solver-selection documentation](https://model-checking.github.io/kani/reference/attributes.html),
[performance discussion](https://model-checking.github.io/kani-verifier-blog/2023/08/03/turbocharging-rust-code-verification.html),
and [loop-contract documentation](https://model-checking.github.io/kani/reference/experimental/loop-contracts.html).
Full slicing, arithmetic refinement and other solver trials did not resolve the tested general
division obligations. Verified low-level contracts and loop invariants are the next approach
for the long division/GCD algorithms; their premises still need proofs before verified stubbing
could establish unconditional callers.

Detailed exports/logs use these `/tmp` prefixes: `kani-round2-root-final{64,32}`,
`kani-round2-division-new{64,32}`, `kani-round2-coverage-final-{64,32}`,
`kani-round2-coverage-lincomb-64`, `kani-round2-coverage-decoder-precision-{64,32}`,
`kani-round2-multiply-final-z3-{64,32}`, `kani-round2-multiply-final-uint{64,32}`,
`kani-round2-root-probes32` (only the retained sqrt result), `kani-round2-sqrt-final64`,
and `kani-round2-recheck{64,32}`. The first round and older inventory below are historical.

## First expansion: 2026-10-23

Upstream baseline: `946a433` (merged into this branch; original uncommitted proofs preserved).

- [x] Fetch and inspect upstream fixes and PR status. Findings 1.12 and the boxed Montgomery
  invertibility branch are fixed in master (#1354/#1355). Most other listed functional bugs
  remain; #1352 (`mul_mod_special`) is open. Do not claim they are fixed.
- [x] Restore the full carry specification and verify it on both word widths; check that
  reinstating the original bug makes the strengthened proof fail.
- [x] Move subtraction coverage to the replacement primitive and check the new bit-count /
  precision helpers against independent native specifications.
- [x] Add unsigned, signed, and boxed proofs in independent source modules; preserve production code.
- [x] Run all harnesses on 64-bit and 32-bit words with per-harness timeouts. Record failures and
  timeouts separately from successes, then run ordinary tests and formatting checks.

### Scope and review

Library harness inventory increased from **85** (82 committed plus 3 existing uncommitted) to
**161**, a net addition of 76. There are also **13 separate regression harnesses** which assert
required behavior for unfixed findings. Harness counts are not a library coverage percentage.

- Unsigned: bitwise operations and their operators/assignment forms, limb-mask AND, conditional
  selection/assignment/swap and comparisons, exact division by powers of two (including mixed
  widths and the #1310 boundaries), remainder modulo powers of two, and limb-division wrappers.
- Signed: bitwise operations, conditional selection/assignment/comparison, resize sign extension
  and truncation, small native conversions, optional magnitude/sign conversion, left shifts,
  shift operators, and checked/wrapping assignment. Native MIN and overflow cases remain included.
- Boxed: concrete 64/128-bit precisions with symbolic values, bit operations and counts, resize,
  mixed-width comparisons, conditional operations, negation, shifts, modular arithmetic, and
  mixed-precision GCD zero operands. Special add/sub/negation include `c = 0` (modulus `2^128`).
- Core: word and limb arithmetic, borrow masks, bit counts and bitwise operations, precision
  helpers and safegcd iteration calculation. The old private subtraction proof now tests its
  replacement, with all incoming borrow words, rather than only the two canonical masks.

Readability changes preserve the specified properties: `short_div` now has two direct normalized
divisor domains instead of mostly vacuous branches; the boxed setters share one symbolic input
and expected result; a redundant matrix generator was removed. The squaring proof checks its
deprecated alias against the already specified pair of output halves instead of repeating the
native conversions. The safegcd row-bound comment
no longer claims that the separate linear combinations are proven. Checked division documents
that its oracle checks wrapper composition and leaves inner arithmetic as a separate obligation.

An independent read-only review checked every changed source file and the regression suite.
It prompted exact widened-remainder and inverse-precision assertions. No blocking oracle or
vacuity findings remain. Original uncommitted multiplication, remainder and square-root proofs
were preserved.

### Verification configuration and limits

Kani 0.68.0, CBMC 6.11.0. Broad runs use default CaDiCaL; targeted follow-ups also try Z3.
The safegcd iteration harness selects Z3 explicitly: the unchanged native arithmetic property
times out with CaDiCaL but verifies in under a second with Z3. Default checks and unwinding
assertions stay enabled. Bitwuzla remains excluded (FINDINGS 4.1), despite its upstream CBMC fix.
`Wrapping` multiplication, Montgomery FIOS and squaring also select Z3. Leave `--solver` unset
to honor these per-harness choices. Measured 64-bit-word verification times are about 0.2s for
`Wrapping` multiplication, 28s for Montgomery FIOS, and 9s for squaring; these are local run
observations, not general performance guarantees. The iteration and `Wrapping` results also
pass with 32-bit words; the other 32-bit cases remain subject to the recorded run results.
A symbolic multiplication sanity check accepts `3 * 5 == 15` and rejects `3 * 5 == 30` with Z3
(`/tmp/kani-z3-controls.log`); this checks the specific historical backend regression, not solver
soundness in general.
Kani runs enable default features plus `alloc`. Native tests enable all features: **901 passed**
with 64-bit words and **897 passed** with forced 32-bit words. Both native configurations also
reproduce the mixed-precision inverse, parsed-zero precision and floored-remainder bugs.

`--cfg cpubits="32"` selects 32-bit words on this aarch64 host; this is **not** a 32-bit-pointer
target run. `--cfg miri` selects portable `cmov`; no assembly backend or timing property is proven.
Widths and input domains are stated in each harness. The division-by-three harness covers all
`u16` dividends stored in `U64`; other division harnesses retain their full stated `u64`/`u128`
domains even if the solver times out. A timeout is an unproved obligation, never a passing proof.

Run library proofs (add `--cfg cpubits="32"` to `RUSTFLAGS` for the other word configuration):

```sh
rtk proxy env RUSTFLAGS='--cfg miri' cargo kani --features alloc -j 2 --output-format terse -Z unstable-options --harness-timeout 120s
```

Run unfixed regressions separately; failures are expected and are not `should_panic` proofs:

```sh
rtk proxy env RUSTFLAGS='--cfg miri' cargo kani --tests --features alloc --harness regression_ -j 2 --output-format terse -Z unstable-options --harness-timeout 120s
```

Full logs and detailed Kani exports are in `/tmp/kani-library-{64,32}.{log,json}` and
`/tmp/kani-regressions-{64,32}.{log,json}`. Targeted follow-ups use `kani-delta`, `kani-iterations`,
`kani-core-32`, `kani-solver-final`, `kani-square-final` and `kani-regressions-delta` prefixes in `/tmp`.
The final run results below supersede
the historical inventory.

### First-round results

Each library harness was checked in both word configurations with a 120s timeout. The broad
runs contain 160 harnesses; targeted runs cover the additional absent-division harness and
subsequent proof changes. The table combines these results by harness, using the final versions.

| Word width | Library proofs passed | Library timeouts | Library counterexamples | Native tests passed |
|---|---:|---:|---:|---:|
| 64 bits | 151 / 161 | 10 | 0 | 901 |
| 32 bits | 148 / 161 | 13 | 0 | 897 |

The following obligations timed out in both configurations:

- `checked_div_option_composition`, `checked_div_absent_propagation`
- `int_mul_i64_correct`
- `classic_bingcd_ct_matches_vartime`
- `u32_rem_correct_for_bit_precisions`
- `boxed_gcd_vartime_zero_wide_rhs`, `boxed_gcd_vartime_zero_narrow_rhs`
- `uint_div_rem_limb_high_bit_u128_correct`, `uint_div_rem_limb_one_u128_correct`
- `sqrt_rem_word_correct`

With 32-bit words, `montgomery_multiply_inner_matches_textbook`, `uint_mul_u64_correct` and
`uint_square_u64_correct` also timed out. These retain their stated input domains and assertions.

The separate regression suite has 13 harnesses. With 64-bit words, one passes, ten report
counterexamples or panics, and two time out. With 32-bit words, eleven report counterexamples
or panics and two time out. The passing case is single-limb inversion, whose known defect
requires 32-bit words. The timeouts are mixed-precision boxed inversion and parsed-zero
precision; fresh native reproducers confirm both defects in both word configurations.
The strengthened exact mixed-precision remainder assertion fails in both configurations too.

The restored carry proof passes with the upstream fix and fails when the old implementation
is reinstated. Expanded proofs also expose `BoxedUint::shr1_assign` returning a false carry for
odd inputs (FINDINGS 1.13); its only production caller currently discards that flag. No local
production fix was added. Formatting and diff-whitespace checks pass.

The earlier inventory below records historical runs. Entries marked "running" are not current
evidence. Whole-library correctness, arbitrary limb counts, constant-time assembly, and release
semantics are outside what these bounded Kani runs establish.

Difficulty labels below describe previous solver experience, not impossibility results.

Run all proofs with `RUSTFLAGS='--cfg miri' cargo kani`. Add `--cfg cpubits="32"` to check the 32-bit `Word` code.
Do not use `--solver bitwuzla`; see `FINDINGS.md` §4.1.

## Difficulty scale

- **Easy:** bit operations, adds and compares on 1–2 limbs. No symbolic multiply or divide. Seconds.
- **Medium:** one symbolic word×word multiply shared with the spec, or long unrolled loops (≤ ~130 steps) of easy steps. Seconds to minutes.
- **Hard:** chains of symbolic multiplies or divides whose correctness needs algebra (e.g., Newton iterations, reciprocal error bounds). SAT may need hours or never finish.
- **Infeasible:** whole algorithms with many multiply-heavy iterations (full safegcd/bingcd inversion, `pow`, Karatsuba at ≥ 16 limbs, recursive sqrt at ≥ 4 limbs). Prove the building blocks instead and cover the rest with differential tests.

## Ranked inventory

Importance × risk, highest first. "Risk" means the chance of being wrong: algorithmic subtlety, recent fixes, and constant-time tricks.

| # | Area / functions | Importance | Risk | Kani difficulty | Status |
|---|---|---|---|---|---|
| 1 | Möller–Granlund division: `reciprocal` (64-bit Alg. 2, 32-bit Alg. 3), `div2by1`, `div3by2`, `reciprocal_3by2` (`uint/div_limb.rs`) | Highest (all division, `mul_mod`, encoding, sqrt) | High | Hard. `short_div` is easy. | `short_div` **proven**. `reciprocal`/`div2by1`: 64-bit CaDiCaL and 32-bit CaDiCaL/Z3 runs stopped after 2–3 h without a result. A 32-bit run with `--refine-arithmetic` is still going. |
| 2 | Multi-limb division `UintRef::div_rem*`, `rem_wide*` (`uint/ref_type/div.rs`) | Highest | High (Knuth D in constant time) | Hard (inherits #1). Medium if the reciprocal is assumed through a contract. | Not started; blocked on #1 |
| 3 | Montgomery FIOS `montgomery_multiply_inner`, `mul_montgomery_form`, repeated squaring (`modular/mul.rs`) | Highest | Medium (fixed in 1ab28f6) | Medium–Hard | FIOS == textbook HAC 14.36 **proven** for 64-bit 1-limb (~3 h). The 32-bit 2-limb variant (inner carry chain): running. The `< 2m` bound for reduced inputs is not attempted (non-linear). |
| 4 | `try_sub_with_carry`, `add_mod`/`sub_mod`/`neg_mod`/`double_mod`, `*_special` (`uint/*_mod.rs`) | High (all modular arithmetic) | Medium | Easy | **Proven**. Found the always-0 carry bug in `try_sub_with_carry` (FINDINGS 1.12). |
| 5 | Word/primitive layer: `carrying_add`, `borrowing_sub`, `carrying_mul_add`, `choice_*`, `select`, masks, `join`/`split` | Highest (everything) | Low–Medium | Easy | **Proven** |
| 6 | `u64_invert_odd`/`u32_invert_odd` (Hurchalla Newton) | High (`mod_neg_inv`, safegcd, `div_exact`) | Medium | Hard (9 chained multiplies) | Stopped after ~2 h (CaDiCaL, 32- and 64-bit) without a result. A 32-bit exhaustive unit test (2^31 inputs) would be a cheap alternative. |
| 7 | `u32_rem` (Lemire), used by every `wrapping_sh*` | High | Medium | Hard for symbolic divisors. Easy for the fixed divisors actually used (`Uint::BITS`). | Symbolic `d ≥ 2`: stopped after ~2 h. All 128 precisions `32·k ≤ 4096`: running. `d == 1` overflows in debug (unreachable). Covered at `d = 128` by the `Uint` shift proofs. |
| 8 | `Limb` add/sub/neg mod, shifts, bits, cmp, ct | High | Low | Easy | **Proven**. Found the `restrict_bits(BITS)` bug (FINDINGS 1.10). |
| 9 | `Uint` add/sub/neg, shifts (ct log-shifter, vartime, wide, limb-carry), cmp (same and mixed width), bits, set/restrict bits | Highest | Medium (shift and carry bookkeeping) | Easy (unwind bounds needed) | **Proven** at 128 bits |
| 10 | safegcd `jump_step`/`jump` (constant-time divsteps), `shr_in_place_wide`, `conditional_negate_in_place_wide` | High (constant-time inversion) | Medium | Easy–Medium | ct == vartime, row bounds, `shr_in_place_wide` (64- and 32-bit words) and `conditional_negate_in_place_wide` **proven**. Low-bits dependence: running. The 32-bit `Uint<1>` panic (FINDINGS 1.1) was confirmed by a unit test. |
| 11 | safegcd `update_fg`/`update_de`/`lincomb_*` (`modular/safegcd.rs`) | High | Medium | Hard (multi-limb multiplies with bounds) | Not started |
| 12 | bingcd `bingcd_step`, classic gcd ct vs vartime, `compact`/`section`, `ExtendedUint`/`ExtendedInt` | Medium–High (`xgcd`, Jacobi, Limb gcd) | Medium | Easy–Medium | step, compact, section, extension **proven**. `Uint<1>` classic gcd ct == vartime: running. |
| 13 | bingcd optimized xgcd (approximation with K bits, `PatternMatrix`) | High (gcd at ≥ 2 limbs) | Medium (uncertain bound, FINDINGS §3) | Matrix ops easy; end-to-end infeasible | `PatternMatrix` ops and `abs_diff` **proven** (found the `conditional_add_right_column_to_left` doc mismatch). End-to-end 1-limb `binxgcd_nz` Bezout identity: running. |
| 14 | `Int` sign/neg/add/sub/cmp/shr, conversions | Medium | Low–Medium (MIN edge cases) | Easy | **Proven** at 128 bits. 64-bit `Int` multiply (`concatenating`/`widening`/`wrapping`/`checked`/`saturating`): running. |
| 15 | `Int` division (trunc, floor, unsigned) | Medium | High (two confirmed bugs, FINDINGS 1.3/1.4) | Hard (needs #1) | Bugs confirmed by unit tests; proofs blocked on #1 |
| 16 | Encoding: hex nibble/byte, byte roundtrips, `*_slice_truncated` (Uint, Limb), `EncodedUint` unsafe casts | Medium | Medium (recent code) | Easy | **Proven**. Limb precision 0 bug (FINDINGS 1.11). |
| 17 | `bitlen`, `nlimbs` | Medium | Medium (recent fixes) | Easy | **Proven** |
| 18 | `Uint` multiplication (schoolbook, overflow detection, squaring) | Highest | High (Karatsuba carry fixes) | 1 limb: medium. 2 limbs: hard. Karatsuba: infeasible (≥ 16 limbs). | 1-limb multiply (229 s) and squaring (130 s) **proven** |
| 19 | `Uint` sqrt (rewritten in 73f18b6), root | Medium | High | 1 limb: medium–hard. 2 limbs: hard (uses `div_rem_limb`). Recursion: infeasible. | `sqrt_rem_word` (= 1-limb constant-time `floor_sqrt`): running. `floor_root_vartime` panic confirmed (FINDINGS 1.6). |
| 20 | `MontyParams::new` (R, R², `mod_neg_inv`), pow, lincomb, sqrt mod p | High | Low–Medium | Hard / infeasible (needs division, many multiplies) | Not started; use differential tests |
| 21 | `div_by_2` (halving mod m) | Medium | Low | Easy | **Proven** |
| 22 | `concat`/`split`/`resize` | Medium | Low | Easy | **Proven** |
| 23 | `BoxedUint` mixed-precision ops (bit ops, add/sub, `neg_mod`, `invert_odd_mod`, constructors) | High (RSA users) | High (FINDINGS 1.2/1.8/1.9) | Easy–Medium (heap, 1–2 limbs) | Mixed-width `\|=`, bit ops, add, sub, and equal-precision `neg_mod` **proven** (concrete limb counts; symbolic `Vec` lengths exhaust memory). Bugs 1.2/1.8/1.9 confirmed by unit tests. |
| 24 | `NonZero`/`Odd` invariants, `Checked`/`Wrapping` wrappers | Low | Low (FINDINGS §3 `NonZero` mul) | Easy | `NonZero::new`, `Odd::new` **proven** (Limb, U128, I128) |

## Why the hard items are hard

SAT solvers handle multiplication badly when the proof needs algebra: associativity, Newton error bounds, or equivalence of two different multiplier circuits.

When the spec reuses the same product the code computes, as in the FIOS-vs-textbook and 1-limb `mul` harnesses, CBMC shares the circuit and the check is cheap. When it does not (e.g., `reciprocal`, `invert_odd`), runs take hours.

Options, in order of cost:
1. Leave the long runs going.
2. Use 32-bit words (`--cfg cpubits="32"`), which makes every multiply 4× smaller in bit-blasted size.
3. Split the input space into cases (e.g., by the 9-bit table index in `reciprocal`).
4. Use exhaustive native tests for 32-bit functions with ≤ 32 input bits.
5. Rebuild CBMC with diffblue/cbmc#9145 so Bitwuzla (much stronger on non-linear bit-vectors) can be used soundly.
