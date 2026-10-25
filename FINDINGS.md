# Findings from the Kani verification effort

Current status against upstream `f9e15c8` (v0.7.8), reviewed 2026-10-25. Reproduction
commits and scratch evidence are noted below; this cleanup does not rerun every reproducer.
**Confirmed** means native reproduction or a Kani counterexample; **code reading** remains
unconfirmed. Debug/release refer to Cargo profiles. No local production fixes are included.

Finding IDs remain stable for regression references. Fixed items are summarized at the end.
The unresolved suite contains 39 harnesses: 27 in `tests/kani_regressions.rs` and 12 inline
test-gated harnesses. These are separate from the passing library CI gate.

## 1. Confirmed unresolved bugs

### 1.1 `Uint<1>` modular inversion panics on 32-bit targets (safegcd)

- **Where:** `src/modular/safegcd.rs` (`shr_in_place_wide::<L, H>` and its callers).
  The callers use `H = I64::LIMBS`. On 32-bit targets that is 2, so `L = 1` gives `H > L`.
- **Effect:** panics in every build. Debug fails `debug_assert!(H <= L)`.
  Release panics in `Uint::shr_vartime` (`src/uint/shr.rs`, "`shift` exceeds upper bound"). It would also index `L - offs` out of bounds.
  The nonzero reproducer below enters a 62-step safegcd batch.
- **Reachable from:** `Uint::<1>::invert_odd_mod` and `invert_odd_mod_vartime` (confirmed). The same path also serves `FixedMontyForm<1>::invert`, `ConstMontyForm<_, 1>::invert`, and `Uint::<1>` safegcd. These were not run separately.
- **Repro** (`RUSTFLAGS='--cfg cpubits="32"'`, debug and release):
  ```rust
  let m = Odd::new(Uint::<1>::from(7u32)).unwrap();
  Uint::<1>::from(3u32).invert_odd_mod(&m); // panics; expected Some(5)
  ```
- **Severity:** medium. This is a DoS in a core crypto primitive on targets where `cpubits` picks 32-bit words (e.g. i686, wasm32). The reproducer uses forced 32-bit words on a 64-bit-pointer host.

### 1.3 `Int::checked_div_rem_floor` (and `_vartime`) returns the wrong remainder sign

- **Where:** `src/int/div.rs`, both variants. The remainder is negated on `opposing_signs`. A floored remainder takes the sign of the divisor.
- **Effect:** for every negative dividend with a non-zero remainder, `q * d + r != n`.
  `(-8, 3)` gives `r = -1` (correct: 1). `(-8, -3)` gives `r = 2` (correct: -2).
  The doctests assert the wrong values.
- **Severity:** medium. The public API returns a silently wrong result. There are no internal callers.

### 1.4 `Int::div_rem_unsigned` / `rem_unsigned` flip the remainder sign when `LIMBS > RHS_LIMBS`

- **Where:** `src/int/div_unsigned.rs`. The remainder is `< rhs`, but it is returned as `Int<RHS_LIMBS>`. A remainder `>= 2^(Word::BITS*RHS_LIMBS - 1)` does not fit.
- **Repro:** `I128::from_i128((1<<64) + (1<<63)).rem_unsigned(&U64::MAX.to_nz().unwrap())` returns `I64::MIN + 1`. The true value is `2^63 + 1`.
- **Severity:** medium. The result is silently wrong. A fix needs an API change: a wider return type, or magnitude plus sign.

### 1.6 `Uint::floor_root_vartime` / `checked_root_vartime` panic on valid inputs

- **Where:** `src/uint/root.rs`, `self.as_ref().shr(rt_bits * exp_m1)`. This uses the panicking `shr`, and the shift reaches `BITS`.
- **Repro:** `Uint::<1>::MAX.floor_root_vartime(9)` panics in debug and release ("`shift` exceeds upper bound").
  Any non-zero value with `exp > BITS` also panics. A large `exp` overflows the `u32` product.
- **Severity:** low-medium (panic on valid public input).
  Test gap: the test in `src/uint/root.rs` checks `s3 > s2`, which is always true. It should be `s3 > s`.

### 1.7 `Uint::from_u128` / `from_wide_word` compile for too few limbs, then panic

- **Where:** `src/uint/from.rs`. `check_limbs!($limbs, $min)` ignores `$min`. This regressed in `961acaf`.
- **Repro (64-bit):** `Uint::<1>::from_u128(1)` panics with an index out of bounds in debug and release. In const context it is a compile-time error.
- **Severity:** low (panic). The intended compile-time check is missing.

### 1.8 `BoxedUint::neg_mod` returns a nonzero result for zero when `p` is wider than `self`

- **Where:** `src/uint/boxed/neg_mod.rs`. The zeroing loop covers `self.nlimbs()` only. Only a `debug_assert_eq!` guards the precision.
- **Repro:** `self = 0` (64-bit), `p = 2^64 + 1` (128-bit). Debug panics; release returns `2^64`.
- **Fix:** zero all of `ret`, e.g. `conditional_set_zero` on `ret`.
- **Severity:** low-medium.

### 1.9 `BoxedUint::from_str_radix_vartime("0", radix)` builds a value with zero limbs

- **Where:** `src/uint/boxed/encoding.rs` (leading zeros are stripped and nothing is pushed).
  Other zero-limb constructors (code reading): `from_words([])`, `From<&[Limb]>(&[])`, `From<&UintRef>` (`debug_assert` only).
- **Confirmed:** `nlimbs() == 0` in release.
- **Downstream (code reading):** `to_nz`/`into_odd` index `limbs[0]`; `wrapping_sh{l,r}_vartime` compute `% 0`.
- **Confirmed consumers:** a parsed zero panics in `bits_vartime()`
  and `unbounded_sh{l,r}(0)` in native debug/release, with both word widths. See 1.19 for the
  underlying empty-`UintRef` contract mismatch and its direct reproducers.
- **Confirmed constructor:**
  `BoxedUint::from_words_with_precision([42], 0)` has zero limbs in native debug/release with
  both word widths. It takes zero iterator elements and bypasses the Vec normalization used
  by `zero_with_precision(0)`. This extends the existing empty-representation family.
- **Negative controls, 2026-10-23:** `zero_with_precision(0)` and `from_be_slice(&[], 0)` instead
  normalize to one zero limb through `From<Vec<Limb>>`; native probes confirm they do not expose
  this empty-storage defect. Do not add them as affected constructors.
- **Severity:** low-medium (panic later, far from the cause).

### 1.10 `Limb::restrict_bits(len)` is wrong for `len >= Limb::BITS`

- **Where:** `src/limb/bits.rs`. `overflowing_shl(len)` masks the shift.
- **Effect:** `restrict_bits(64)` returns `0` instead of `self`; `restrict_bits(65)` keeps only bit 0. Same in debug and release. Kani found the counterexample.
- **Severity:** low. In-crate callers only pass `1..BITS`, but the method is public. `UintRef::restrict_bits` handles this case correctly.

### 1.11 `Limb::from_{be,le}_slice_truncated(bytes, 0)` panics in debug

- **Where:** `src/limb/encoding.rs`, `Word::MAX >> Limb::BITS` at zero precision. Release masks the shift and returns the correct `0`.
- **Severity:** low. The docs only promise a panic for `bits_precision > BITS`.

### 1.13 `BoxedUint::shr1_assign` loses the shifted-out bit

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
- **Fix direction:** read the carry MSB. No local production fix is included.

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
  fail on both. Passing proofs explicitly state their zero-output premise.

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

- **`primitives::u32_rem(n, 1)`** overflows (`u64::MAX / 1 + 1`, `src/primitives.rs`) and panics in debug. Release is correct. Internally unreachable: divisors are always multiples of 32.
- **`Limb::borrowing_sub(self, rhs, borrow)`** reads only the MSB of `borrow` (mask convention), but the doc says `self - (rhs + borrow)`.
  `Limb::ZERO.borrowing_sub(Limb::ZERO, Limb::ONE)` returns `(0, 0)`.
  `carrying_add` instead adds the complete numeric carry word, so the two conventions are easy
  to mix up. The same documentation mismatch exists in public `BoxedUint::borrowing_sub`:
  zero at 64-bit precision minus `(zero + Limb::ONE)` returns zero and borrow zero. Native
  default-word dev-profile evidence is `/tmp/kani-round5-boxed-borrow.rs` and its `.log`.
  The arbitrary-borrow boxed proofs explicitly verify the implemented MSB convention,
  rather than treating it as the full numeric subtraction described by the docs.
- **`UintRef::shl1_assign` and its carry helper** describe the outgoing bit as the LSB; it is the
  original MSB. Native single-word examples and the mixed-length single-bit shift proof check
  the implemented carry convention. Right shifts deliberately return the bit in the carry MSB.
- **`PatternMatrix::conditional_add_right_column_to_left`** (`src/modular/bingcd/matrix.rs`) computes `left ← −(left + right)`, not `left ← left + right` as documented (proven by `pattern_matrix_ops_correct`).
  The magnitudes are only valid when `m01 >= m00` and `m11 >= m10`; otherwise `wrapping_sub` wraps.
  `binxgcd_nz` follows it with `conditional_negate`, which gives `[left + right, −right]`, the intended result.
  Whether those caller inequalities always hold remains unproved.
- **`classic_bingcd_` vs `classic_bingcd_vartime_` Jacobi sign** (`src/modular/bingcd/gcd.rs`): when `gcd ≠ 1`, the two can return different `jacobi_neg`.
  The constant-time loop keeps running after `a == 0`, and each extra step XORs in `(2|gcd)`. By hand, `m = 3, y = 6` gives 0 after the vartime loop's 2 steps but 1 after 127 steps. Kani confirmed that a mismatch exists, but not this specific input.
  Harmless to this caller, because `jacobi_symbol` maps any `gcd ≠ 1` to 0. The full CT/VT
  agreement harness remains unproved on both word widths; see `KANI_PLAN.md`.
  A future caller that uses the raw sign would be exposed.
- **`Uint::section_vartime{,_length}`** (`src/modular/bingcd/compact.rs`) document `idx + length <= BITS` but panic for `idx == BITS, length == 0` (shift out of range). Unreachable: `compact` never passes it.

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

- **`bitlen::mul`** (`src/bitlen.rs`) truncates and wraps in release. `usize_to_u32` only `debug_assert`s, and the `reason` text claims a panic.
  A byte slice over 2^29 bytes gives a `BoxedUint` with the wrong precision.
  Also: `debug_assert!(n < u32::MAX)` is off by one.
- **AMM property docs** at `src/modular/mul.rs`: properties 2 and 3 are false as stated.
  The `pow` conclusion still holds by a different argument, and only for base < m.
- **`div3by2`'s "maxed" branch** (`src/uint/div_limb.rs`) returns `r = d.saturating_add(u0)`, which is not a remainder. Latent: both callers discard `r`.
- **Uncertain, needs expert review:**
  - bingcd `compact` keeps K/K bits, but the doc and Pornin's analysis use K+1 high / K-1 low bits (`src/modular/bingcd/compact.rs`, `src/modular/bingcd/xgcd.rs`).
  - The safegcd iteration bound when the input is >= the modulus (`src/modular/safegcd.rs`).
- **Doc nits:**
  - `src/word.rs` says `Word::MAX`; it should be `WideWord::MAX`.
  - `Int::wrapping_shr` docs describe out-of-range shifts wrongly.
  - The `overflowing_sh{l,r}_assign_vartime` docs state the return value inverted.

## 4. Current verification-tool limitations

These observations apply to Kani 0.68.0 / bundled CBMC 6.11.0 unless stated otherwise.

### 4.1 Bitwuzla produces false proofs and counterexamples

CBMC's SMT2 encoding of checked arithmetic flattens `{result, overflowed}` in the wrong
bit order for backends without datatypes. Affected probes include multiplication and signed
addition/subtraction; unsigned addition/subtraction, wrapping arithmetic and SAT backends
were unaffected in the investigation. Constrained `3 * 5 == 30` and a real multiplication
overflow incorrectly reported success; valid products also produced false counterexamples.
The issue reproduced with plain CBMC, so this is not evidence of a Bitwuzla solver bug.

[CBMC #9145](https://github.com/diffblue/cbmc/pull/9145) fixed the upstream encoder, but the
bundled 6.11.0 remains affected. **No proof here uses Bitwuzla.** Z3 uses SMT datatypes and
passed the multiplication controls; that is not a general proof of solver soundness.

### 4.2 Symbolic Boolean ordering is unconstrained

`<`, `<=`, `>`, `>=` on symbolic `bool`, including tuple ordering, become fresh variables
in the SAT backend (`c_bool` is missing from CBMC's relation conversion). Terse output
hides the warning. Equality, `cmp`, `partial_cmp`, `min`/`max` and `u8` comparisons were
unaffected. Z3 instead aborts on the unsupported relation.
This over-approximation preserves passing assertion proofs, but can produce false
`cover!`/`should_panic` successes and invalid playback/coverage witnesses. Harnesses use
`u8` for ordered high bits. No upstream issue was filed during this investigation.

### 4.3 Valid-value checking crashes

`-Z valid-value-checks` crashes in Kani's `check_values.rs`. The `UintRef::cmp` conversion
from `i8` to `Ordering` is instead checked against an independently valid `Ordering`.

### 4.4 Assembly is outside the model

`--cfg miri` selects `cmov`'s portable backend because Kani cannot model its aarch64/x86
inline assembly. The proofs do not establish assembly correctness or constant-time execution.

### 4.5 Kani checks debug semantics

Debug assertions and overflow checks are enabled. Release-only truncation or wrapping
behavior needs separate native validation; a debug assertion is not a release precondition.

### 4.6 Timeouts can orphan Z3 processes

Seventeen orphaned Z3 processes were found after timeout runs, invalidating those timings as
benchmarks. CI now bounds each Z3 process and kills the verifier process group; see
[KANI_CI.md](KANI_CI.md). A timeout or tool error never establishes a proof, even with zero
failed checks. Global external-SMT arguments must not be attached to CaDiCaL runs.

### 4.7 Z3 conversion fails for some heap and dynamic-slice proofs

CBMC aborts with `map::at: key not found` on some boxed encoding, Montgomery, word-vector
and symbolic-length slice harnesses. These are tool errors, not counterexamples. The retained
versions pass with CaDiCaL; fixed-size arithmetic can still use Z3.

### 4.8 Fixed-width one-limb special multiplication triggers a compiler ICE

A direct `Uint<1>::mul_mod_special` harness fails while lowering an unreachable
`from_wide_word` branch (`length >= min_length`). It is not active because it blocks library
compilation. Native one-limb `c=0` tests pass; boxed one-word behavior has passing Kani proofs.
This is a verifier limitation, separate from the fixed runtime bug below.

## 5. Fixed findings

| Original finding | Upstream fix | Retained validation |
|---|---|---|
| 1.2: boxed inverse truncated to input precision | [#1367](https://github.com/RustCrypto/crypto-bigint/pull/1367) | Concrete CT/VT wider-modulus inverse and precision proofs pass on both word widths. |
| 1.5: maximum-carry special multiplication; also one-limb `c=0` panic | [#1368](https://github.com/RustCrypto/crypto-bigint/pull/1368) | Fixed/boxed carry and zero-parameter proofs; native one-limb zero-parameter tests. Fixed-width one-limb Kani remains blocked by 4.8. |
| 1.12: `try_sub_with_carry` returned the wrong carry | [#1354](https://github.com/RustCrypto/crypto-bigint/pull/1354) | Exact low-result and carry assertions; restoring the old implementation fails the proof. |
| Former boxed Montgomery invertibility branch on a secret | [#1355](https://github.com/RustCrypto/crypto-bigint/pull/1355) | Source recheck confirms removal; timing is not formally proved. |
