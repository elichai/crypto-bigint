# Kani verification plan and status

## Current status — 2026-11-09

Upstream baseline: `cd67842` (after v0.7.11, `master`). Local library changes are proof modules,
not runtime fixes. Kani 0.68.0 / CBMC 6.11.0 and Z3 5.3.1 are the tested versions.

The previous [full hosted run](https://github.com/elichai/crypto-bigint/actions/runs/40190158002)
passed 663 selected configurations against `f9e15c8`. That result does not certify this rebase:
upstream changed arithmetic and helper implementations. The rebased gate must be rerun.

| Arithmetic Word | Library inventory | Selected for CI | Pending/unproved | Native tests passed |
|---|---:|---:|---:|---:|
| 64 bits | 344 | 334 | 10 | 918 |
| 32 bits | 345 | 335 | 10 | 914 |

Rebased local Kani checks pass for zero parsing, Word arithmetic, software zero counts and
the native addition-decomposition lemma on both widths. Full Word64 FIOS/reference agreement
also passes (about 49 seconds). The forced32 comparison still times out at 290 seconds without
a counterexample. These focused checks do not replace the full hosted gate.

Native counts include all-feature tests and doctests with warnings denied. The seven CI-runner
tests and formatting also pass. The 38 unresolved regression harnesses are
separate from this gate; see [FINDINGS.md](FINDINGS.md).

Both configurations use 64-bit pointers and portable `cmov` (`--cfg miri`). Forced32 changes
the arithmetic Word, not the target architecture. Harness counts are not function-coverage
percentages. Proofs establish their stated domains and unwinding bounds, not arbitrary limb
counts, release-only behavior, assembly correctness or constant-time execution.

## Coverage and important limits

Exact domains and assertions live beside each function in its `kani_proofs` module.

| Area | Current coverage and limits |
|---|---|
| Word/Limb primitives | Native arithmetic, full carry/borrow words, bit operations, comparisons, selection, shifts, precision and split/join helpers. Low-16 inverse-bit checks do not prove the full inverse. |
| Fixed unsigned/signed integers | Native 64/128-bit arithmetic and conversions, full `u64`/`i64` products and squares, overflow/checked/saturating forms, bits/shifts and exact split/resize positions. Larger products/Karatsuba remain open. |
| `UintRef` | Bounded backing arrays and mixed active lengths, numeric results, carry/borrow and untouched storage. Multiplication proofs state zero-output premises; empty-view failures remain regressions. |
| `BoxedUint` | Stated 64/128-bit scalar domains, mixed precision, numeric results and allocated precision, constructor/resize boundaries and unsafe views. Word-vector conversion covers lengths 0–4 with 0–2 spare words, not arbitrary heap sizes. |
| Division and roots | Powers of two, fixed limb-divisor boundaries and selected small domains. Remainder-only passes do not discharge quotient/remainder obligations. Word square-root reconstruction passes on forced32; Word64 uses the separate lemmas below. |
| GCD/inversion | Divsteps, matrix/extension operations, bounds and signed linear combinations. CT/VT agreement is not an independent GCD/inverse specification. Concrete inverse regressions do not prove arbitrary inversion. |
| Modular/Montgomery | Reduced `u64` add/double/subtract, selected reduction/retrieval moduli and multiplication boundaries. U64 FIOS/reference agreement passes with Word64 but remains pending with Word32. Canonical bounds and general parameter/power correctness remain open. |
| Encoding/wrappers | Native bit-pattern round trips, stated byte lengths/precisions, checked validity and payloads, wrapper construction/selection. Documented API mismatches remain separate regressions. |

Recent upstream fixes have passing regressions for the full boxed inverse value/precision,
special-modulus maximum carry and boxed one-word zero-parameter behavior. Fixed two-word `c=0`
checks every operand pair against wrapping multiplication. Maximum-`c` three-word proofs use
three explicit boundaries, not arbitrary operands. Direct fixed-width `Uint<1>` special
multiplication remains blocked at the repository's opt-level=2 by a Kani compiler ICE; an isolated
Word64 opt-level=0 zero-parameter proof and native tests pass (see FINDINGS 4.8).
The zero-parser regression now belongs to the library suite. Modular `NonZero` multiplication
is rejected by the new `Integer` bound; its obsolete runtime harness was removed. The separate
`NonZero::one()` modulus-one invariant failure remains a regression (FINDINGS 1.22).

## Pending obligations and next work

[scripts/kani-ci-pending.json](scripts/kani-ci-pending.json) is the authoritative list of
unproved harnesses and word configurations. Nine obligations are shared by both widths:

- Checked division option composition and absent-value propagation.
- Classic CT/VT binary-GCD agreement and `u32_rem` over all stated bit precisions.
- Boxed GCD with a zero operand and wider/narrower RHS storage; out-of-range VT `invert_mod2k`.
- Full `u128` quotient/remainder at limb divisors one and the high-bit boundary.

Word64 additionally retains `sqrt_rem_word_correct`; Word32 additionally retains
`montgomery_multiply_inner_matches_textbook`. Timeouts and tool errors are unproved results,
not counterexamples or passes. Smaller-domain lemmas do not replace these obligations.

Priorities:

1. Prove reciprocal/division building blocks, then compose multi-limb division and Checked APIs.
2. Establish independent GCD/Bezout/inverse specifications and remaining safegcd invariants.
3. Prove Montgomery canonical bounds, parameter construction and power/multiplier wrappers.
4. Extend larger multiplication, mixed signed products, roots and optional encoding/array paths.

Use verified premises before contracts/stubs can justify callers. CI currently rejects contract
inventories, so contract checking needs an explicit gate policy before it is introduced.
Keep original input domains, overflow/carry assertions and default safety/unwinding checks.

## Proof structure and performance

- Separate independent product, overflow and wrapping properties. Source attributes select
  measured solvers; leave `--solver` unset. The full forced32 wide-product proof passed hosted
  CaDiCaL in about 51 seconds after Z3 exceeded 300 seconds, with no property changes.
- The Montgomery oracle splits a native sum into high halves and a low-word overflow carry.
  An independent all-u128-input lemma checks equality with the original overflowing-sum
  reconstruction on both Word widths. This preserves the full product and extra carry while
  sharing product circuits with upstream's new FIOS implementation.
- Native radix-2^32 partial products preserve full operands/results while sharing arithmetic
  circuits. They do not call the library multiplier. For the carry-ordered references in boxed
  multiplication and safegcd, B=2^32 and the intermediates satisfy
  `p01 <= B²-B-1`, `p10 <= B²-B`, `p11 <= B²-2`; each therefore fits u64. The assembled value is
  the exact u64×u64 product. The alternative Uint reference sums widened partial products in u128.
- Split allocation cases rather than symbolic heap lengths or large conjunctions. Seven
  word-vector proofs retain all 15 original length/capacity combinations and all assertions.
  Hosted empty cases take about 4–11 seconds; nonempty cases about 2 seconds. Earlier combined
  runs lost the hosted runner; shutdown cause is unconfirmed, rather than established OOM.
- Keep full numeric value and precision assertions separate: `BoxedUint` equality ignores
  allocated precision. Preserve high halves, outgoing carries, MIN cases and cancellation.
- Bitwuzla is excluded with the bundled CBMC. Heap/dynamic-slice Z3 conversion errors and other
  tool limits are recorded in [FINDINGS.md](FINDINGS.md#4-current-verification-tool-limitations).

### Square-root induction argument

`sqrt_rem_word_bounds` and `sqrt_rem_word_radix_four_step` pass for every Word on both widths,
without input assumptions. The verified base is `(s,r)=(0,0)` at zero. For v>0, let
p=v>>2<v, d=v&3 and let `(s,r)` be the prefix result. The verified recurrence is:

```text
c = 4r + d,  b = (c >= 4s + 1)
q = 2s + b,  r' = c - b(4s + 1)
```

Inductively, `s²+r=p` gives `q²+r'=4p+d=v`. With `0<=r<=2s`, either branch gives
`0<=r'<=2q`, so q is floor(sqrt(v)). Widened intermediates cannot wrap:
even unrestricted Word r gives `4r+d <= 2^(W+2)-1 < 2^(2W)` for W=32/64.
The induction composition is a human argument, not a machine-proved Kani theorem. It does
not reclassify the original Word64 reconstruction harness as passing.

## Running and maintaining the suite

See [KANI_CI.md](KANI_CI.md) for pinned installation, local commands, pending diagnostics,
timeouts, cache policy and artifact validation. CI discovers new `--lib --features alloc`
harnesses automatically and fails on stale exclusions, missing results or non-success statuses.
It recomputes proofs; caches contain tools/downloads, not previous successful proof results.
Known bug counterexamples remain separate test-gated harnesses until their upstream fixes
are checked and their required properties can move into the library suite.
