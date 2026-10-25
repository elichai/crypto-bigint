# Findings from the Kani verification effort

Status legend:
- **Confirmed**: reproduced by a concrete test or by a Kani counterexample. Commands and outputs are kept in the session scratch directory.
- **Code reading**: found by reading the code. Not executed yet.

"Debug" and "release" refer to Cargo's corresponding profiles, including standalone native probes.
Unless explicitly dated below, line numbers refer to commit `d1a4e1c` (the branch base).

## Upstream recheck: 2026-10-25

The proof branch is rebuilt directly on upstream `f9e15c8` (v0.7.8). The merge commits mentioned
in older entries are historical evidence only; they are not part of the new branch's changes.

- **Fixed:** finding 1.5 and the one-limb `c = 0` case in section 2, by
  [PR #1368](https://github.com/RustCrypto/crypto-bigint/pull/1368). Both fixed-width and boxed
  APIs now document zero as wrapping multiplication. The carry correction uses a full two-word
  multiply-add instead of overflowing `carry + 1` in a word.
- Native all-feature tests with warnings denied pass on both word configurations, including
  the new fixed-width and boxed `mul_mod_special_zero_c` and `mul_mod_special_max_c` tests.
  The fixed-width tests include one-limb inputs. Evidence:
  `/tmp/kani-ci-round-native{64,32}.log` (908/904 tests, including doctests).
- The exact max-carry and boxed one-limb zero-parameter regressions have moved into library
  harnesses. Their original expected values remain intact. Further default-library coverage
  and Kani results are recorded in the latest `KANI_PLAN.md` entry. The unresolved integration
  suite now contains 39 regressions.
- A direct fixed-width `Uint<1>::mul_mod_special` Kani harness still triggers the previously
  documented compiler internal error in the unreachable `from_wide_word` branch. It is not
  retained as an active harness because it prevents compiling all library proofs. Native
  one-limb tests cover the runtime case; this is not a claim of a passing fixed-width one-limb
  Kani proof.

The other findings have not been reclassified by this update. Older status descriptions below
remain historical; the newest dated entry takes precedence.

## Upstream recheck: 2026-10-24

Fetched and merged upstream `f8d9dfc` as local merge `9b74396`, preserving the existing proof work.
This update supersedes the historical status below for finding 1.2 only.

- **Fixed:** 1.2, boxed odd-modulus inverse truncation, by
  [PR #1367](https://github.com/RustCrypto/crypto-bigint/pull/1367). Both CT and VT APIs now retain
  the full inverse. Result storage has `max(input.bits_precision(), modulus.bits_precision())`
  bits: the local modulus is widened before normalization, so the result need not have exactly
  the original modulus's precision when the input is wider.
- Native checks compare archived `946a433` with `9b74396`, using one unchanged harness in
  debug/release with both word widths. Inverting 3 and 2 modulo `2^127-1` from 64-bit input
  storage fails the exact-value assertion in all 16 old-snapshot cases and passes in all 16
  patched cases. The harness checks full numeric results and result precision.
- Forty baseline controls and their forty patched counterparts pass: reverse mixed precision,
  equal precision, rejection of zero and non-coprime inputs, and boxed Montgomery inversion,
  each through CT/VT APIs. All 112 checks reached their assertions; no build failure is counted
  as a reproduction. Evidence and the unchanged harness hash are retained in
  `/private/tmp/kani-upstream-20261024-native-validation/run-1/results.json` and its sibling logs.
- The original desired-property regression is now in the library suite, alongside its VT
  counterpart. Both pass Kani on both word widths. Their original one-word input/two-word
  modulus domain and full inverse/precision assertions are preserved. The 64-bit proofs need
  more than the old 120-second budget; final CT/VT checks pass in 156.50/141.98 seconds with
  a 300-second budget. The unresolved integration suite now contains 41 regressions.
- The other five newly fetched commits contain a release/dependency update, resolver change,
  documentation correction and additional `checkct` coverage. They do not fix another listed
  functional finding. This recheck does not establish constant-time execution.

Kani results and any verifier limits for the restored inverse obligations are recorded in the
latest section of `KANI_PLAN.md`. The earlier regression counts and statuses below are historical.

## Upstream recheck: 2026-10-23

Compared against upstream `946a433`, now merged into this proof branch.

- **Fixed:** 1.12 (`try_sub_with_carry`), [PR #1354](https://github.com/RustCrypto/crypto-bigint/pull/1354).
  The proof now specifies both the low result and the carry, rather than accepting the broken
  always-zero carry. Reinstating the old implementation makes the assertion fail; restoring the
  upstream fix makes it pass (`/tmp/kani-carry-{before,after}-fix.log`).
- **Fixed:** the boxed Montgomery invertibility branch from section 3,
  [PR #1355](https://github.com/RustCrypto/crypto-bigint/pull/1355). This is a source-level recheck;
  functional Kani proofs do not establish constant-time execution.
- Upstream also replaced subtraction and bit-count primitives and precision calculations in
  [#1358](https://github.com/RustCrypto/crypto-bigint/pull/1358) and
  [#1360](https://github.com/RustCrypto/crypto-bigint/pull/1360). Harnesses now exercise the
  replacements. The removed private primitive is no longer called by a stale proof.
- Findings 1.1–1.11 are **not fixed in this master snapshot**. In particular,
  [#1352](https://github.com/RustCrypto/crypto-bigint/pull/1352) is still open. Do not infer that a
  reported issue or an open PR has been fixed. `tests/kani_regressions.rs` asserts required behavior
  for reproducible cases; these are separate from the default library proof run and may fail.
- **New confirmed finding:** 1.13 below, found by the expanded shift harnesses.
- **Third expansion:** the earlier code-reading observations about `UintRef` multiplication,
  `PrimeParams`, signed decoding, boxed decoding, raw Montgomery values and wrapping division
  now have native reproducers and separate Kani regressions. The `NonZero` modular-product
  observation is also confirmed. See 1.14–1.18 and section 2.
  These are functional/API findings; no application-level exploit is established.
- **Fourth expansion:** 1.19 confirms empty-view bit-length and shift failures, including the
  parsed-zero consumer from 1.9. The fifth expansion also confirms the boxed variant of the
  borrow-word documentation trap in section 2; it has the same MSB convention as `Limb`.
- **Sixth expansion:** section 2 records `BoxedUint::max(0)`'s inconsistent numeric/allocated
  precision guarantee. A native probe also confirms `from_words_with_precision(_, 0)` as
  another empty-storage constructor in the existing 1.9 family. Findings 1.20–1.21 record
  zero-request resize inconsistency and safe resize operations that break wrapper invariants.
- **Seventh expansion:** section 2 records a missing `c != 0` precondition for one-limb
  special-modulus multiplication. Fixed and boxed one-limb calls panic at `c = 0`; their
  two-limb counterparts accept the same case.

Fresh results and remaining solver timeouts are recorded in `KANI_PLAN.md`. The original findings
below remain as historical descriptions; the status above takes precedence.

## 1. Confirmed bugs in the crate

### 1.1 `Uint<1>` modular inversion panics on 32-bit targets (safegcd)

- **Where:** `src/modular/safegcd.rs:323-342` (`shr_in_place_wide::<L, H>`), called from `:462` and `:497`.
  The callers use `H = I64::LIMBS`. On 32-bit targets that is 2, so `L = 1` gives `H > L`.
- **Effect:** panics in every build. Debug fails `debug_assert!(H <= L)` (`:328`).
  Release panics in `Uint::shr_vartime` (`src/uint/shr.rs:27`, "`shift` exceeds upper bound"). It would also index `L - offs` out of bounds.
  The first safegcd batch is always 62 steps, so any input triggers it.
- **Reachable from:** `Uint::<1>::invert_odd_mod` and `invert_odd_mod_vartime` (confirmed). The same path also serves `FixedMontyForm<1>::invert`, `ConstMontyForm<_, 1>::invert`, and `Uint::<1>` safegcd. These were not run separately.
- **Repro** (`RUSTFLAGS='--cfg cpubits="32"'`, debug and release):
  ```rust
  let m = Odd::new(Uint::<1>::from(7u32)).unwrap();
  Uint::<1>::from(3u32).invert_odd_mod(&m); // panics; expected Some(5)
  ```
- **Severity:** medium. This is a DoS in a core crypto primitive on targets where `cpubits` picks 32-bit words (e.g. i686, wasm32). Presumably CI misses it because no 32-bit test inverts a 1-limb value; I did not check the CI matrix.

### 1.2 `BoxedUint::invert_odd_mod` returns a wrong inverse when the modulus is wider than `self`

- **Fixed upstream:** `f8d9dfc` / PR #1367, merged and validated on 2026-10-24. See the recheck
  above. The following describes the old implementation.
- **Where:** `src/modular/safegcd/boxed.rs:122-124`. The result is cut down with `.resize_unchecked(a.bits_precision())`, which drops the high limbs.
- **Effect:** silently wrong. `is_some` is `true` in both debug and release.
  The `invert_mod` docs say mismatched limbs "will panic", but only a `debug_assert` exists, and `invert_odd_mod` has no check at all.
- **Repro (rechecked 2026-10-23):** `a = BoxedUint::from(3u64)`, `m = 2^127 - 1` (128-bit).
  The call returns `0x5555_5555_5555_5555` at 64-bit precision. The correct inverse is
  `0x5555_5555_5555_5555_5555_5555_5555_5555`. The earlier draft incorrectly reported the truncated
  value as `0xFFFF_FFFF_FFFF_FFFF`; the truncation bug and success flag are confirmed natively.
- **Severity:** high. A plausible mixed-precision call (for example, inverting a small value modulo an RSA modulus) gets a wrong result with a success flag. The existing tests only cover inverse = 1, which fits in 64 bits.

### 1.3 `Int::checked_div_rem_floor` (and `_vartime`) returns the wrong remainder sign

- **Where:** `src/int/div.rs:299` (ct) and `:189` (vartime). The remainder is negated on `opposing_signs`. A floored remainder takes the sign of the divisor.
- **Effect:** for every negative dividend with a non-zero remainder, `q * d + r != n`.
  `(-8, 3)` gives `r = -1` (correct: 1). `(-8, -3)` gives `r = 2` (correct: -2).
  The doctests at `:262-273` assert the wrong values.
- **Severity:** medium. The public API returns a silently wrong result. There are no internal callers.

### 1.4 `Int::div_rem_unsigned` / `rem_unsigned` flip the remainder sign when `LIMBS > RHS_LIMBS`

- **Where:** `src/int/div_unsigned.rs:43` and `:100`. The remainder is `< rhs`, but it is returned as `Int<RHS_LIMBS>`. A remainder `>= 2^(64*RHS_LIMBS - 1)` does not fit.
- **Repro:** `I128::from_i128((1<<64) + (1<<63)).rem_unsigned(&U64::MAX.to_nz().unwrap())` returns `I64::MIN + 1`. The true value is `2^63 + 1`.
- **Severity:** medium. The result is silently wrong. A fix needs an API change: a wider return type, or magnitude plus sign.

### 1.5 `Uint::mul_mod_special` overflows when `c == Limb::MAX` (LIMBS >= 3)

- **Fixed upstream:** `f205fa4` / PR #1368; see the 2026-10-25 recheck. The following describes
  the old implementation.
- **Where:** `src/uint/mul_mod.rs:45`, `(carry.0 + 1) as WideWord`. `carry` can equal `c`.
- **Repro (64-bit):** `U192`, `c = Limb::MAX`, `a = b = [0, MAX-1, MAX]` (`a < p`).
  Debug panics with an add overflow. Release returns a wrong value.
- **Fix:** `(carry.0 as WideWord + 1) * c.0 as WideWord`.
- **Severity:** low-medium. It needs an unusual modulus `2^(64L) - 2^64 + 1`, but the docs allow any single-limb `c`.

### 1.6 `Uint::floor_root_vartime` / `checked_root_vartime` panic on valid inputs

- **Where:** `src/uint/root.rs:57`, `self.as_ref().shr(rt_bits * exp_m1)`. This uses the panicking `shr`, and the shift reaches `BITS`.
- **Repro:** `Uint::<1>::MAX.floor_root_vartime(9)` panics in debug and release ("`shift` exceeds upper bound").
  Any non-zero value with `exp > BITS` also panics. A large `exp` overflows the `u32` product.
- **Severity:** low-medium (panic on valid public input).
  Test gap: `root.rs:151` checks `s3 > s2`, which is always true. It should be `s3 > s`.

### 1.7 `Uint::from_u128` / `from_wide_word` compile for too few limbs, then panic

- **Where:** `src/uint/from.rs:5-13`. `check_limbs!($limbs, $min)` ignores `$min`. This regressed in `961acaf`.
- **Repro (64-bit):** `Uint::<1>::from_u128(1)` panics with an index out of bounds (`from.rs:97`), in debug and release. In const context it is a compile-time error.
- **Severity:** low (panic). The intended compile-time check is missing.

### 1.8 `BoxedUint::neg_mod` returns `p` instead of `0` when `p` is wider than `self`

- **Where:** `src/uint/boxed/neg_mod.rs:9-21`. The zeroing loop covers `self.nlimbs()` only. Only a `debug_assert_eq!` guards the precision.
- **Repro:** `self = 0` (64-bit), `p = 2^64 + 1` (128-bit). Debug panics; release returns `2^64`.
- **Fix:** zero all of `ret`, e.g. `conditional_set_zero` on `ret`.
- **Severity:** low-medium.

### 1.9 `BoxedUint::from_str_radix_vartime("0", radix)` builds a value with zero limbs

- **Where:** `src/uint/boxed/encoding.rs:205` (leading zeros are stripped and nothing is pushed).
  Other zero-limb constructors (code reading): `from_words([])`, `From<&[Limb]>(&[])`, `From<&UintRef>` (`debug_assert` only).
- **Confirmed:** `nlimbs() == 0` in release.
- **Downstream (code reading):** `to_nz`/`into_odd` index `limbs[0]`; `wrapping_sh{l,r}_vartime` compute `% 0`.
- **Additional confirmed consumers, fourth expansion:** a parsed zero panics in `bits_vartime()`
  and `unbounded_sh{l,r}(0)` in native debug/release, with both word widths. See 1.19 for the
  underlying empty-`UintRef` contract mismatch and its direct reproducers.
- **Additional confirmed constructor, sixth expansion:**
  `BoxedUint::from_words_with_precision([42], 0)` has zero limbs in native debug/release with
  both word widths. It takes zero iterator elements and bypasses the Vec normalization used
  by `zero_with_precision(0)`. This extends the existing empty-representation family.
- **Negative controls, 2026-10-23:** `zero_with_precision(0)` and `from_be_slice(&[], 0)` instead
  normalize to one zero limb through `From<Vec<Limb>>`; native probes confirm they do not expose
  this empty-storage defect. Do not add them as affected constructors.
- **Severity:** low-medium (panic later, far from the cause).

### 1.10 `Limb::restrict_bits(len)` is wrong for `len >= Limb::BITS`

- **Where:** `src/limb/bits.rs:79-82`. `overflowing_shl(len)` masks the shift.
- **Effect:** `restrict_bits(64)` returns `0` instead of `self`; `restrict_bits(65)` keeps only bit 0. Same in debug and release. Kani found the counterexample.
- **Severity:** low. In-crate callers only pass `1..BITS`, but the method is public. `UintRef::restrict_bits` handles this case correctly.

### 1.11 `Limb::from_{be,le}_slice_truncated(bytes, 0)` panics in debug

- **Where:** `src/limb/encoding.rs:124`, `Word::MAX >> 64`. Release masks the shift and returns the correct `0`.
- **Severity:** low. The docs only promise a panic for `bits_precision > BITS`.

### 1.12 `Uint::try_sub_with_carry` always returns carry 0

- **Where:** `src/uint/sub_mod.rs:21-26`. `(_, carry) = carry.borrowing_sub(...)` keeps the *borrow* output of the subtraction, not its result.
- **Effect:** for `carry == 1` and `self >= rhs`, the true result is `>= 2^BITS`, but the returned carry is 0. The low part is correct; Kani proves it for all inputs.
- **Counterexample (Kani):** `self = 2^128 - 1`, `carry = 1`, `rhs = 2^127 - 1` returns `(2^127, 0)`. The correct result is `(2^127, 1)`.
- **Reachability:** none today, because all callers use `.0` only.
  Before `1ab28f6` ("Fix Montgomery reduction in repeated squaring"), `square_repeat_montgomery_form` chained this carry into a second `try_sub_with_carry`. This bug plausibly caused that issue, which the fix removed by restructuring rather than by correcting the carry. This is an inference; I did not reproduce the old failure.
- **Severity:** latent. The doc ("Returns the result of the subtraction and the new carry") invites future misuse.

### 1.13 `BoxedUint::shr1_assign` loses the shifted-out bit (2026-10-23)

- **Where:** `src/uint/boxed/shr.rs`, `shr1_assign` (upstream `946a433`).
  It calls `UintRef::shr1_assign().lsb_to_choice()`, but the returned `Limb` places the bit in its
  **MSB**, not its LSB. The returned `Choice` is always false on both supported word widths.
- **Repro:** `let mut x = BoxedUint::one(); assert!(x.shr1_assign().to_bool());` fails. The shifted
  value is correctly zero. This internal method was tested natively from its module, and the
  symbolic 128-bit carry assertion also fails in Kani with 64- and 32-bit words.
- **Reachability:** the only production caller is `div_by_2_boxed_assign`, which discards the flag.
  The bug is latent in current library arithmetic; it does not establish a public arithmetic failure.
- **Retained regression:** `uint::boxed::shr::kani_regressions::regression_boxed_shr1_carry`, enabled
  by `--tests --features alloc --harness regression_`. It asserts the required bit and shifted value.
- **Fix direction:** read the carry MSB. No production fix is included in this proof expansion.

### 1.14 `UintRef::wrapping_mul` / `wrapping_square` depend on old output contents

- **Confirmed against `946a433`, 2026-10-23:** multiplying 1 by 1 into an output containing 1
  produces 2; squaring 1 into an output containing 1 produces 3. The public methods document
  the product/square and do not require zero-filled output. Their `overflowing_*` wrappers inherit
  this behavior. All buffers are disjoint and constructed through safe public APIs.
- **Cause:** the small multiplication path invokes `wrapping_mul_add` even when Karatsuba's
  `add` flag is false. The small squaring path also consumes old output values.
- **Size dependence:** 16-limb multiplication and 17-limb squaring of the value 1 into an output
  initially containing 1 instead produce the correct 1. Clearing the output is not a uniform
  API rule inferable from the larger-input behavior.
- **Reachability:** public `UintRef` arithmetic is affected. Inspected fixed/boxed multiplication
  callers supply zeroed or explicitly cleared buffers; the square-root caller clears its scratch
  buffer. This is not evidence of a wrong result in those higher-level operations.
- **Evidence:** native probes on both word widths; desired-behavior Kani regressions
  `regression_uint_ref_mul_overwrites_output` and `regression_uint_ref_square_overwrites_output`
  fail on both. New passing proofs explicitly state their zero-output premise.

### 1.15 `UintRef` multiplication/squaring overflow uses the input width

- **Confirmed against `946a433`, 2026-10-23:** `check_mul_overflow` tests against
  `self.nlimbs()` rather than `out.nlimbs()`. Let B = 2^Word::BITS and use zero-filled outputs:
  - B * B and B² into four limbs store the correct B² but report overflow (false positives).
  - B * 1 into one limb stores zero and reports no overflow (false negative).
  - Squaring the three-limb representation `[1, 1, 0]` into one limb stores 1 and reports no
    overflow, although (B+1)² exceeds that output width.
- **Reachability:** the safe public API accepts these unequal widths without a documented
  restriction. The docs do not explicitly define the overflow width; the false-positive/negative
  labels above interpret overflow relative to the buffer receiving the wrapping result.
  Inspected boxed checked/saturating callers use output width equal to the left
  operand, which avoids these reproducers. The modular multiplier clears/resizes scratch space.
- **Evidence:** native probes on both word widths and four failing
  `regression_uint_ref_{mul,square}_overflow_uses_{wide,narrow}_output_width` Kani harnesses.
  Separate full-word, equal-width overflow properties pass on both word widths.

### 1.16 `PrimeParams` conditional assignment leaves `t` behind

- **Confirmed against `946a433`, 2026-10-23:** construct prime parameters for p=3, generator=2
  (`t=1`) and p=7, generator=3 (`t=3`). Assigning the latter into the former with `Choice::TRUE`
  updates the other fields but leaves `t=1`. `CtSelect` inherits this through `CtAssign`.
- **Equality also omits `t`:** the inconsistent selected value compares equal to the p=7
  parameters despite a different public `t()` result. For pristine constructor-produced
  parameters, `t` is determined by other fields; omission alone does not establish a false equality
  between two pristine values. The public assignment/selection creates the counterexample.
- **Impact:** an inconsistent parameter object and an incomplete equality relation. No current
  internal consumer of `t()` was found; no wrong square-root result or timing leak is established.
- **Evidence:** native debug/release probes with both word widths; failing Kani regressions
  `regression_prime_params_ct_assign_copies_t` and `regression_prime_params_ct_eq_observes_t`.

### 1.17 Signed `Encoding` truncating decoders reject supported input shapes

- **Confirmed against `946a433`, 2026-10-23:** `I128` inherits the default trait implementation.
  Both byte orders panic for short input `[1]` at precision 128, and for full-width encoding of 1
  at precision 8. These are separate failures: `expect("input too short")` and the exact-width
  `assert_eq!`. The trait documents a possible panic when precision *exceeds* the underlying size.
- **Expected behavior:** truncation at the requested precision is specified by the trait.
  Short-input zero-padding is an inference from `U128`'s accepting overrides, not an explicit
  padding rule in the trait docs; at minimum, the extra short-input panic is undocumented.
  These positive-value reproducers avoid any ambiguity about sign extension.
- **Evidence:** native debug/release on both word widths and four failing
  `regression_int_decode_{short,reduced_precision}_{be,le}` Kani harnesses. Full-width signed byte
  serialization and byte-order dispatch pass for every i128 bit pattern in separate proofs.
- **Impact:** a panic on otherwise permitted safe public input. No production fix is included.

### 1.18 Generic `NonZero<T>` multiplication can construct a zero residue

- **Confirmed against `946a433`, 2026-10-23:** for `ConstMontyForm` modulo 15, construct nonzero
  3 and 5 through `NonZero::new(...).unwrap()`. Multiplying the wrappers yields a `NonZero` whose
  retrieved value is zero. No unchecked constructor or invalid modulus is required.
- **Cause:** the generic `Mul` implementation directly wraps `self.0 * rhs.0`. Its comment assumes
  nonzero integers always have a nonzero product, but `T: Mul<T, Output=T>` also admits rings with
  zero divisors. Composite Montgomery moduli are supported by this API.
- **Impact:** violation of the wrapper invariant through safe checked construction and arithmetic.
  No memory-safety failure or application-level exploit is established; `NonZero`'s own unchecked
  constructor docs describe its invariant as a panic concern rather than a memory-safety one.
- **Evidence:** native debug/release probes on both word widths and the failing
  `regression_nonzero_modular_product_preserves_invariant` harness. A correction needs a stronger
  algebraic bound or a checked product; no production fix is included here.

### 1.19 Empty `UintRef` violates overflowing-shift contracts and exposes parsed-zero panics

- **Confirmed against `946a433`, 2026-10-23:** `UintRef::new(&[])` and `new_mut(&mut [])`
  are safe public constructors without a nonempty precondition. `bits()` returns 0, but
  `bits_vartime()` panics. `overflowing_shl_assign(0)`, `overflowing_shr_assign(0)` and
  `unbounded_shl_assign(0)` / `unbounded_shr_assign(0)` also panic on this representation.
- **Cause:** variable-time bit length computes `len - 1` and indexes the slice. The overflowing
  shifts correctly detect `shift >= bits_precision()` but still call a bounded shift with
  `(shift, bound) = (0, 0)`, which violates its strict upper bound. Unbounded constant-time shifts
  inherit that panic. The `UintRef` overflowing contract explicitly promises an unchanged value
  and truthy flag for this boundary. `BoxedUint`'s public unbounded shifts explicitly promise
  panic-free shifting; the parsed-zero call path below violates that promise. Empty `bits_vartime`
  also exposes an undocumented nonempty requirement, inconsistent with `bits()` and the tested
  bit-count/query functions. Safe construction alone does not imply that every method is total.
- **Negative controls:** both variable-time unbounded shifts accept the empty slice and preserve
  zero. `bits`, `leading_zeros`, trailing-zero/one counts and bit queries accept the empty view.
  `BitOps::leading_zeros_vartime` instead delegates to `bits_vartime` by code reading. Passing
  mixed-length proofs include empty views wherever the implementation supports them; nonempty-only proofs
  state the restricted domain rather than hiding this failure.
- **Reachability:** direct public `UintRef` use, and the empty `BoxedUint` constructed by the
  previously reported zero-string parser (1.9). Native probes confirm the latter also panics in
  `bits_vartime()` and both allocating constant-time unbounded shifts at shift 0. Ordinary
  nonempty fixed/boxed zero representations do not establish these reproducers.
- **Evidence:** `/private/tmp/kani-round4-native` and its debug/release logs for both word widths;
  five isolated `regression_uint_ref_empty_*` harnesses assert the required behavior. Each fails
  exactly one intended panic/overflow check on both word widths, without a timeout or tool error. This extends
  the known empty-representation problem; it is not a separate root cause for the parser bug.
  No production fix is included.

### 1.20 Zero-request boxed resize depends on ownership and can invalidate `Odd`

- **Confirmed against `946a433`, 2026-10-23:** for the same boxed integer 42,
  `(&n).resize_unchecked(0)` returns 42, while `n.resize_unchecked(0)` returns zero.
  Both results have one limb of storage. The owned `Odd::new(BoxedUint::from(1u64)).unwrap()`
  resized to zero requested bits becomes an `Odd<BoxedUint>` containing zero.
- **Cause:** the owned path shrinks the Vec to zero limbs, then normalizes the empty Vec to
  one zero limb. The borrowed path allocates the normalized one-limb destination first and
  copies the input's low limb. `OddBoxedUint` directly wraps the owned result without checking
  that the low odd bit survived. Positive requested widths retain at least one original limb.
- **Contract/impact:** unchecked resizing intentionally permits truncation; that does not
  explain different values for equivalent owned/borrowed inputs or a returned `Odd` containing
  zero. `Resize` documents no caller precondition that permits violating wrapper invariants.
  This is a correctness/API issue. No application-level exploit or memory-safety failure is
  established; the wrapper's unchecked-constructor warning describes possible consumer panics.
- **Evidence:** native debug/release probes on both word widths in
  `/tmp/kani-round6-resize-native{64,32}-{debug,release}.log`; separate desired-property Kani
  regressions compare owned/borrowed values and require the resized odd value to remain odd.
  `regression_boxed_resize_owned_borrowed_zero_precision` and
  `regression_boxed_resize_odd_preserves_invariant` each fail their intended assertion on both
  widths in `/tmp/kani-round6-reg{64,32}.json`, with no other failed checks or tool errors.
  The passing owned resize boundary proof explicitly describes current zero-request behavior,
  rather than certifying it as the desired contract. No production fix is included.

### 1.21 Unchecked boxed resize can return a zero `NonZero`

- **Confirmed against `946a433`, 2026-10-23:** construct `NonZero<BoxedUint>` from `1u128 << 64`
  using `NonZero::new(...).unwrap()`, then resize to 64 bits. Both owned and borrowed
  `resize_unchecked(64)` return a `NonZero<BoxedUint>` whose limbs are all zero. This reproduces
  on both word widths and does not depend on zero requested precision.
- **Cause:** both `Resize` implementations delegate to truncating boxed resize and then call
  `NonZero::new_unchecked` without checking the truncated value. Unchecked numeric truncation
  is documented, but preservation of the returned wrapper's invariant is not conditional on
  an explicit caller precondition. The checked `try_resize` path rejects this input instead.
- **Impact:** a safe public operation escapes the nonzero invariant from a valid checked
  input. This is a correctness/API sharp edge, distinct from modular-product finding 1.18.
  No memory-safety failure, exploitable in-crate caller or application-level exploit is
  established. Consumers may assume nonzero and panic; a correction must preserve the invariant,
  change the output type, or explicitly resolve the unchecked API's contract.
- **Evidence:** the same four native resize logs above and a separate desired-property Kani
  regression `regression_boxed_resize_nonzero_preserves_invariant` requiring both resized values
  to remain nonzero. Both assertions fail on both widths in `/tmp/kani-round6-reg{64,32}.json`,
  with no other failed checks or tool errors. No production fix is included.

## 2. Confirmed API traps (documentation or unreachable)

- **`primitives::u32_rem(n, 1)`** overflows (`u64::MAX / 1 + 1`, `src/primitives.rs:87`) and panics in debug. Release is correct. Internally unreachable: divisors are always multiples of 32.
- **`Limb::borrowing_sub(self, rhs, borrow)`** reads only the MSB of `borrow` (mask convention), but the doc says `self - (rhs + borrow)`.
  `Limb::ZERO.borrowing_sub(Limb::ZERO, Limb::ONE)` returns `(0, 0)`.
  `carrying_add` instead adds the complete numeric carry word, so the two conventions are easy
  to mix up. The same documentation mismatch exists in public `BoxedUint::borrowing_sub`:
  zero at 64-bit precision minus `(zero + Limb::ONE)` returns zero and borrow zero. Native
  default-word dev-profile evidence is `/tmp/kani-round5-boxed-borrow.rs` and its `.log`.
  The new arbitrary-borrow boxed proofs explicitly verify the implemented MSB convention,
  rather than treating it as the full numeric subtraction described by the docs.
- **`UintRef::shl1_assign` and its carry helper** describe the outgoing bit as the LSB; it is the
  original MSB. Native single-word examples and the mixed-length single-bit shift proof check
  the implemented carry convention. Right shifts deliberately return the bit in the carry MSB.
- **`PatternMatrix::conditional_add_right_column_to_left`** (`src/modular/bingcd/matrix.rs:183-188`) computes `left ← −(left + right)`, not `left ← left + right` as documented (proven by `pattern_matrix_ops_correct`).
  The magnitudes are only valid when `m01 >= m00` and `m11 >= m10`; otherwise `wrapping_sub` wraps.
  `binxgcd_nz` follows it with `conditional_negate`, which gives `[left + right, −right]`, the intended result.
  Whether `m01 >= m00` always holds there is not proven. The end-to-end Bezout harness for 1-limb `binxgcd_nz` is meant to settle it.
- **`classic_bingcd_` vs `classic_bingcd_vartime_` Jacobi sign** (`src/modular/bingcd/gcd.rs:69-110`): when `gcd ≠ 1`, the two can return different `jacobi_neg`.
  The constant-time loop keeps running after `a == 0`, and each extra step XORs in `(2|gcd)`. By hand, `m = 3, y = 6` gives 0 after the vartime loop's 2 steps but 1 after 127 steps. Kani confirmed that a mismatch exists, but not this specific input.
  Harmless today, because `jacobi_symbol` maps any `gcd ≠ 1` to 0. Kani proves (32-bit words) that the gcds always agree and the signs agree when `gcd = 1`.
  A future caller that uses the raw sign would be exposed.
- **`Uint::section_vartime{,_length}`** (`src/modular/bingcd/compact.rs`) document `idx + length <= BITS` but panic for `idx == BITS, length == 0` (shift out of range). Unreachable: `compact` never passes it.

### One-limb special-modulus multiplication omits its nonzero-`c` requirement

- **Fixed upstream:** `f205fa4` / PR #1368 now supports and documents `c = 0` as wrapping
  multiplication. See the 2026-10-25 recheck. The following evidence describes the old behavior.

Confirmed against `946a433` on 2026-10-23 in native debug/release with both word widths:
`Uint::<1>::from_word(3).mul_mod_special(&Uint::<1>::from_word(7), Limb::ZERO)` panics.
The one-limb `BoxedUint` equivalent also panics. Both methods document `p = MAX+1-c` with
`c` small enough to fit a limb, without excluding zero or documenting this panic. At `c = 0`,
that formula denotes the well-defined modulus `2^BITS`; the expected product here is 21.

The one-limb special case constructs the ordinary limb modulus `0-c`, then rejects zero in
`NonZero::new_unwrap`. Two-limb fixed and boxed values instead use the general special-modulus
algorithm and return 21 at `c = 0`. This storage-dependent boundary is a documentation/API
precondition mismatch. If zero is intentionally unsupported, the public methods must state
`c != 0`; this finding does not assume that supporting modulus `2^BITS` is the intended design.
No application-level exploit or memory-safety failure is established, and no production fix
is included. Native evidence: `/private/tmp/kani-round7-native` and
`/tmp/kani-round7-special-zero{64,32}-{debug,release}.log`. The desired-property boxed regression is
`regression_boxed_mul_mod_special_zero_c`. The fixed one-limb Kani attempt triggers a compiler
ICE while lowering `Uint::<1>::from_wide_word` from the unreachable multi-limb branch of
`mul_mod_special` (`place.rs:812`, `length >= min_length`); its native evidence remains valid.
That new experimental harness was removed so it does not prevent running the regression suite.
Compiler evidence: `/tmp/kani-round7-reg64-fixed.log`. This is a verifier limitation, not another
observed runtime failure.

The boxed regression fails exactly the intended `NonZero::<Limb>::new_unwrap` assertion on
both word widths, with no other failed checks or backend errors:
`/tmp/kani-round7-reg{64,32}.json`. A separate passing boxed control checks every Word-sized
operand pair in two-word storage; it does not claim arbitrary two-word operands or fixed-Uint
coverage. The concrete fixed-Uint two-word control is included in the native probes above.

### `BoxedUint::max(0)` contradicts the documented maximum

Confirmed against `946a433` in native debug/release with both word widths: `BoxedUint::max(0)`
returns zero with one limb of storage. Its resulting `bits_precision()` is 32 or 64, but the
method explicitly documents the value `2^self.bits_precision() - 1`. That value would be
`Word::MAX`, not zero. The cause is the empty maximum-filled Vec being normalized to one
**zero** limb. Positive requested precisions have a separate constructor harness; the exceptional
zero request is retained as `regression_boxed_max_zero_precision`.

This is an API/documentation mismatch. A zero-bit mathematical maximum is also zero, so resolving
it may mean clarifying the zero-request guarantee rather than changing the returned value.
No application-level arithmetic failure is established. The regression asserts the current
resulting-precision guarantee and fails exactly one intended value assertion on both word widths.
Evidence: `/private/tmp/kani-round6-native`, `/tmp/kani-round6-native{64,32}-{debug,release}.log`
and `/tmp/kani-round6-max-reg{64,32}.json`. No production fix is included.

### Boxed checked decoding silently truncates instead of returning `Precision`

Confirmed against `946a433` on 2026-10-23: `BoxedUint::from_be_slice(&[2], 1)` and its LE
equivalent return `Ok(0)`. The input length fits the rounded byte count; the value exceeds the
requested bit precision. The docs promise `DecodeError::Precision`, but both methods delegate
to truncating decoders. Existing tests also accept masked values, so this may be intended behavior
with stale docs rather than an unintended implementation change. Callers cannot use the
documented error as input validation. Native debug/release probes on both word widths and
`regression_boxed_decode_precision_{be,le}` confirm the mismatch.

### Public raw Montgomery APIs omit the reduced-representation requirement

Confirmed against `946a433` on 2026-10-23: pass the modulus itself through public
`from_montgomery`, or write it through `as_montgomery_mut`. Fixed/const forms panic on retrieval
in debug and return m in release; the boxed form returns m even in debug. The public retrieval
contract promises a reduced result, but these raw APIs do not state the requirement x<m.
The private `montgomery_retrieve` **does** document and debug-assert that precondition: this is
a public invariant/documentation boundary defect, not a failure for valid reduced representations.
Normal `new()` construction is unaffected by these reproducers. Native debug/release probes on
both word widths and three `regression_{fixed,const,boxed}_monty_retrieve_raw_modulus` harnesses
retain the evidence. Raw input m is mathematically zero in the residue class.

### `Wrapping<Int>` division still panics on MIN / -1

Confirmed against `946a433` on 2026-10-23: `Wrapping(I128::MIN) / I128::from_i128(-1).to_nz().unwrap()`
panics in debug and release, on both word widths. Its implementation unwraps checked signed
division. The wrapper advertises wrapping arithmetic; the corresponding
[`core::num::Wrapping` implementation](https://doc.rust-lang.org/stable/src/core/num/wrapping.rs.html)
uses `wrapping_div` and returns MIN. `regression_wrapping_int_division_min` asserts that wrapping
behavior and fails in Kani. Bare `Int::div_vartime` also has an overflow-panic path by code reading;
it is not a wrapping API, and is not claimed as the same defect here.

## 3. Code-reading findings (not yet confirmed)

- **`bitlen::mul`** (`src/bitlen.rs:57-63`) truncates and wraps in release. `usize_to_u32` only `debug_assert`s, and the `reason` text claims a panic.
  A byte slice over 2^29 bytes gives a `BoxedUint` with the wrong precision.
  Also: `debug_assert!(n < u32::MAX)` is off by one.
- **Fixed upstream:** the former `BoxedMontyForm::invert{,_vartime}` secret-dependent branch was
  removed by PR #1355; see the upstream recheck above. Functional proofs do not establish timing.
- **AMM property docs** at `src/modular/mul.rs:19-29`: properties 2 and 3 are false as stated.
  The `pow` conclusion still holds by a different argument, and only for base < m.
- **`div3by2`'s "maxed" branch** (`src/uint/div_limb.rs:176-178`) returns `r = d.saturating_add(u0)`, which is not a remainder. Latent: both callers discard `r`.
- **Uncertain, needs expert review:**
  - bingcd `compact` keeps K/K bits, but the doc and Pornin's analysis use K+1 high / K-1 low bits (`src/modular/bingcd/compact.rs:63-79`, `xgcd.rs:245-263`).
  - The safegcd iteration bound when the input is >= the modulus (`src/modular/safegcd.rs:356-363`).
- **Doc nits:**
  - `src/word.rs:146` says `Word::MAX`; it should be `WideWord::MAX`.
  - `Int::wrapping_shr` docs describe out-of-range shifts wrongly.
  - The `overflowing_sh{l,r}_assign_vartime` docs state the return value inverted.

## 4. Tooling findings

### 4.1 Kani with `--solver bitwuzla` is unsound here

**2026-10-23 upstream recheck:** [CBMC #9145](https://github.com/diffblue/cbmc/pull/9145) merged on
2026-10-12 as `32441ee38f8b2be63281a88a0df1a585dd4fdc43`; issue #9144 is closed. The installed
Kani 0.68.0 still bundles CBMC 6.11.0, so the affected solver configuration remains excluded from
these runs. The bundled verifier was not changed. A targeted Z3 follow-up is recorded in
`KANI_PLAN.md`; Bitwuzla was not used.

With Kani 0.68.0, CBMC 6.11.0, Bitwuzla 0.9.1 (aarch64-apple-darwin), this fully concrete harness reports **VERIFICATION FAILED** with `--solver bitwuzla`. It is **SUCCESSFUL** with the default CaDiCaL:

```rust
#[kani::proof]
fn u64_mul_probe() {
    let (a, b): (u64, u64) = kani::any();
    kani::assume(a == 0x8000_0001 && b == 0xFFFF_FFFE);
    assert_eq!(a * b, 0x7FFF_FFFF_FFFF_FFFE);
}
```

The value assertion fails, not the overflow check. The same happens for a `u64 x u64 -> u128` product. A `u128` shift passes.

Bitwuzla also produced a spurious counterexample for `u32_rem` (`n = 2^31, d = 2^32 - 2`); natively the result is correct.

**Root cause: CBMC's SMT2 back end, not Bitwuzla or Kani.**
- Kani lowers checked arithmetic to CBMC's `overflow_result_exprt`, a `{result, overflowed}` pair.
- For solvers without datatypes (Bitwuzla), CBMC flattens the pair into one bit-vector, but emits `(concat result ovf_bit)` in the wrong order (`smt2_conv.cpp`).
- So `result` reads as `(true_result << 1) | ovf`, and the overflow flag reads a wrong bit.
- Historical status as of 2026-10-07: [diffblue/cbmc#9144](https://github.com/diffblue/cbmc/issues/9144)
  and its fix [diffblue/cbmc#9145](https://github.com/diffblue/cbmc/pull/9145) were open. See the
  updated status above. CBMC 6.11.0, the version bundled with the installed Kani 0.68.0, is affected.

**Scope** (from the investigation; the probes are in the session scratch directory):
- Affected: checked/overflow-checked `*` at every width, and signed `+`/`-`.
- Not affected: unsigned `+`/`-`, `wrapping_*`, and the SAT back ends.
- `--solver z3` gives correct results, because Z3 gets SMT datatypes.
- **It also produces false proofs.** With Bitwuzla, `assert_eq!(3u64 * 5, 30)` and a real `2^32 * 2^32` overflow both report SUCCESS. The same false proof reproduces in plain C with CBMC alone (`__builtin_mul_overflow`).
- A dumped SMT2 file is `sat` under both Bitwuzla and Z3, which confirms that the encoding, not the solver, is wrong.

**No proof in this branch relies on Bitwuzla.** The original proofs used default CaDiCaL;
the 2026-10-23 expansion also uses Z3 for explicitly selected harnesses (see `KANI_PLAN.md`).
A draft Kani tracking issue (not filed) is in the scratch directory at `bitwuzla-investigation/DRAFT-kani-issue.md`.

### 4.2 Kani: ordering comparisons on symbolic `bool` are unconstrained

**Effect:**
- With Kani 0.68.0 / CBMC 6.11.0, `<`, `<=`, `>`, `>=` on symbolic `bool` produce an arbitrary value. This covers `PartialOrd::lt/le/gt/ge`, comparisons against a constant, and any compared tuple containing a `bool` (tuple ordering calls `bool::lt`).
- Not affected: `==`, `!=`, `cmp`, `partial_cmp`, `min`/`max`, fully concrete comparisons, and `(a as u8) < (b as u8)`.

**Root cause:**
- Kani's `comparison_expr` (`kani-compiler/src/codegen_cprover_gotoc/codegen/rvalue.rs`) emits a relation on CBMC `c_bool` operands.
- CBMC's `boolbvt::convert_bv_rel` has no `c_bool` case. It falls back to `convert_rest`, which returns a fresh variable and prints `warning: ignoring <`. That warning is hidden by `--output-format terse`.
- With `--solver z3`, CBMC aborts instead ("unsupported type for >=: c_bool"), so that path fails closed.

**Soundness impact:**
- Because the value is unconstrained (an over-approximation), a passing `assert!` is still sound.
- Anything that relies on reachability is not: `cover!`, `#[kani::should_panic]`, concrete playback, and coverage. All of these were confirmed to give false SATISFIED/SUCCESS.

**Existing reports:** none found in model-checking/kani or diffblue/cbmc. A draft Kani issue (not filed) is in the scratch directory at `kani-bool-investigation/DRAFT_KANI_ISSUE.md`.

**Impact on this branch:**
- One harness (`uint_add_mod_correct`) used `bool` tuple ordering in a `kani::assume`. That only widened the assumed input set, so the original proof was sound.
- The follow-up commit "kani: avoid bool ordering in the Uint::add_mod harness" overstates this as a possible loss of coverage.
- A spurious `add_mod_special` failure from the same bug led to finding it. All harnesses now use `u8` for such high bits.

### 4.3 Kani: `-Z valid-value-checks` crashes

`cargo kani -Z valid-value-checks` panics in `kani-compiler/src/kani_middle/transform/check_values.rs:959` on this crate. The `UintRef::cmp` transmute (`i8` to `Ordering`) is instead covered by asserting equality with a valid `Ordering`.

### 4.4 `cmov` inline assembly

`ctutils` uses `cmov`, whose aarch64/x86 backends use inline assembly, which Kani cannot model.
`cfg(miri)` selects `cmov`'s portable backend, so proofs run with `RUSTFLAGS='--cfg miri' cargo kani`. No other dependency reads `cfg(miri)`; in this crate it only affects test-only code.
Consequence: the proofs cover the portable `cmov` backend, not the assembly backends.

### 4.5 Kani verifies debug semantics

Kani builds with `debug_assertions` and overflow checks on. A `debug_assert!` precondition is enforced, and a debug-only panic counts as a failure.
Release-only behaviour (silent truncation in `bitlen::mul`, wrapping where debug panics) is not what is verified.

### 4.6 Kani timeouts can leave Z3 running (2026-10-23)

During the second expansion, 17 Z3 processes from completed timeout runs were still consuming
CPU. Each had parent PID 1 and this worktree as its working directory; they were stopped only
after checking both facts. This contention makes the earlier wall-clock observations unsuitable
as performance benchmarks. A timeout still establishes no proof, regardless of contention.

`scripts/kani-z3-timeout` wraps Z3 with its own 110-second process limit. Pass it through CBMC's
`--external-smt2-solver` option **only for selected Z3 harnesses**, alongside Kani's 120-second
harness limit. It preserves arguments
and exit status. Preprocessing can consume part of Kani's budget before Z3 starts, so the child
can briefly outlive its parent; the wrapper bounds that lifetime rather than guaranteeing an
earlier exit. Solver timeouts remain unproved/error results, even when Kani prints zero failed
checks. Positive and negative multiplication controls were rechecked with the wrapper.
Do not attach this option to a mixed CaDiCaL/Z3 run: with an explicitly selected CaDiCaL backend,
CBMC fails with `Invalid User Input; Option: --outfile; Reason: required filename not provided`.

### 4.7 CBMC/Z3 SSA conversion fails on heap and dynamic-slice harnesses (2026-10-23)

The new boxed byte encoding harnesses reached an internal `map::at: key not found` error during
CBMC's SSA conversion with Z3. This is a tool error, not a counterexample. The final encoding
properties pass with default CaDiCaL. Logs are in `/tmp/kani-round2-coverage-gcd-encoding-64.log`;
no standalone minimization or upstream report was made.
The third expansion also reproduced the same error on stack-backed symbolic-length `UintRef`
addition, subtraction and multiplication proofs. The final dynamic-slice properties pass with
CaDiCaL on both word widths; fixed-size arithmetic proofs can still use Z3. This extends the
known trigger beyond boxed allocation and is not a newly discovered library counterexample.
Boxed Montgomery retrieval/selection trials hit the same Z3 conversion error; see
`/tmp/kani-round3-modular-z3-error64.log`. The retained bounded retrieval proof passes with CaDiCaL.
