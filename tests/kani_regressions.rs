//! Reproducers for findings not yet fixed upstream; these assert the required behavior.
//! Run separately with `cargo kani --tests --features alloc --harness regression_`.
//! Failures are intentional evidence of unresolved bugs, never expected-panic proofs.

#![cfg(kani)]

use core::num::NonZeroU32;
use crypto_bigint::{Encoding, I128, Limb, Uint, UintRef, Word, Wrapping};

/// Empty UintRef construction is public; bit length must agree with the constant-time variant.
#[kani::proof]
#[kani::unwind(3)]
fn regression_uint_ref_empty_bit_length() {
    assert_eq!(UintRef::new(&[]).bits_vartime(), 0);
}

#[kani::proof]
#[kani::unwind(3)]
fn regression_uint_ref_empty_overflowing_shl() {
    assert!(
        UintRef::new_mut(&mut [])
            .overflowing_shl_assign(0)
            .to_bool()
    );
}

#[kani::proof]
#[kani::unwind(3)]
fn regression_uint_ref_empty_overflowing_shr() {
    assert!(
        UintRef::new_mut(&mut [])
            .overflowing_shr_assign(0)
            .to_bool()
    );
}

#[kani::proof]
#[kani::unwind(3)]
fn regression_uint_ref_empty_unbounded_shl() {
    let mut empty = [];
    UintRef::new_mut(&mut empty).unbounded_shl_assign(0);
    assert_eq!(UintRef::new(&empty).bits(), 0);
}

#[kani::proof]
#[kani::unwind(3)]
fn regression_uint_ref_empty_unbounded_shr() {
    let mut empty = [];
    UintRef::new_mut(&mut empty).unbounded_shr_assign(0);
    assert_eq!(UintRef::new(&empty).bits(), 0);
}

/// Nonzero residue classes can multiply to zero for a composite modulus.
#[kani::proof]
#[kani::unwind(5)]
fn regression_nonzero_modular_product_preserves_invariant() {
    use crypto_bigint::{NonZero, U64, modular::ConstMontyForm};
    crypto_bigint::const_monty_params!(Modulus15, U64, "000000000000000f");
    type Form = ConstMontyForm<Modulus15, { U64::LIMBS }>;
    let three = NonZero::new(Form::new(&U64::from_u8(3))).unwrap();
    let five = NonZero::new(Form::new(&U64::from_u8(5))).unwrap();
    assert!((three * five).as_ref().retrieve() != U64::ZERO);
}

/// Smaller precision is documented; short-input padding is the behavior of the Uint overrides.
#[kani::proof]
#[kani::unwind(18)]
fn regression_int_decode_short_be() {
    assert_eq!(I128::from_be_slice_truncated(&[1], 128), I128::ONE);
}

#[kani::proof]
#[kani::unwind(18)]
fn regression_int_decode_short_le() {
    assert_eq!(I128::from_le_slice_truncated(&[1], 128), I128::ONE);
}

#[kani::proof]
#[kani::unwind(18)]
fn regression_int_decode_reduced_precision_be() {
    assert_eq!(
        I128::from_be_slice_truncated(&1i128.to_be_bytes(), 8),
        I128::ONE
    );
}

#[kani::proof]
#[kani::unwind(18)]
fn regression_int_decode_reduced_precision_le() {
    assert_eq!(
        I128::from_le_slice_truncated(&1i128.to_le_bytes(), 8),
        I128::ONE
    );
}

#[kani::proof]
#[kani::unwind(140)]
fn regression_wrapping_int_division_min() {
    let negative_one = I128::from_i128(-1).to_nz().unwrap();
    assert_eq!((Wrapping(I128::MIN) / negative_one).0, I128::MIN);
}

/// FINDINGS 1.10: restriction at or above the word width must preserve every bit.
#[kani::proof]
fn regression_limb_restrict_bits_full_domain() {
    let (x, len): (Word, u32) = kani::any();
    let expected = if len >= Word::BITS {
        x
    } else {
        x & ((1 << len) - 1)
    };
    assert_eq!(Limb(x).restrict_bits(len).0, expected);
}

/// FINDINGS 1.11: zero precision is allowed by both decoder contracts.
#[kani::proof]
#[kani::unwind(10)]
fn regression_limb_decode_zero_precision_be() {
    let bytes: [u8; Limb::BYTES] = kani::any();
    assert_eq!(Limb::from_be_slice_truncated(&bytes, 0), Limb::ZERO);
}

#[kani::proof]
#[kani::unwind(10)]
fn regression_limb_decode_zero_precision_le() {
    let bytes: [u8; Limb::BYTES] = kani::any();
    assert_eq!(Limb::from_le_slice_truncated(&bytes, 0), Limb::ZERO);
}

/// FINDINGS 1.3: floor division must satisfy n = q*d + r, for either divisor sign.
#[kani::proof]
#[kani::unwind(140)]
fn regression_int_floor_division_identity() {
    let negative: bool = kani::any();
    let d = if negative { -3i128 } else { 3 };
    let divisor = I128::from_i128(d).to_nz().unwrap();
    let (q, r) = I128::from_i128(-8).checked_div_rem_floor(&divisor);
    assert_eq!(i128::from(q.unwrap()) * d + i128::from(r), -8);
}

#[kani::proof]
#[kani::unwind(140)]
fn regression_int_floor_division_identity_vartime() {
    let negative: bool = kani::any();
    let d = if negative { -3i128 } else { 3 };
    let divisor = I128::from_i128(d).to_nz().unwrap();
    let (q, r) = I128::from_i128(-8).checked_div_rem_floor_vartime(&divisor);
    assert_eq!(i128::from(q.unwrap()) * d + i128::from(r), -8);
}

/// FINDINGS 1.4: the remainder must represent the exact positive value, not just its low bits.
#[kani::proof]
#[kani::unwind(140)]
fn regression_int_unsigned_mixed_precision_remainder() {
    let n = (1i128 << Word::BITS) + (1i128 << (Word::BITS - 1));
    let d = Uint::<1>::MAX.to_nz().unwrap();
    let (q, r) = I128::from_i128(n).div_rem_unsigned(&d);
    assert_eq!(i128::from(q), 1);
    assert_eq!(
        i128::from(r.resize::<{ I128::LIMBS }>()),
        (1i128 << (Word::BITS - 1)) + 1
    );
}

/// FINDINGS 1.6: exponents greater than the precision are valid, and root(1) = 1.
#[kani::proof]
#[kani::unwind(140)]
fn regression_root_large_exponent() {
    let exp = NonZeroU32::new(Word::BITS + 1).unwrap();
    assert_eq!(Uint::<1>::ONE.floor_root_vartime(exp), Uint::<1>::ONE);
}

#[kani::proof]
#[kani::unwind(140)]
fn regression_root_max_ninth_power() {
    let s = Uint::<1>::MAX
        .floor_root_vartime(NonZeroU32::new(9).unwrap())
        .as_words()[0];
    assert!(
        u128::from(s)
            .checked_pow(9)
            .is_some_and(|v| v <= u128::from(Word::MAX))
    );
    assert!(
        (u128::from(s) + 1)
            .checked_pow(9)
            .is_none_or(|v| v > u128::from(Word::MAX))
    );
}

/// FINDINGS 1.1: also exercises a single 32-bit limb with a 64-bit safegcd high part.
#[kani::proof]
#[kani::unwind(140)]
fn regression_single_limb_invert_odd_mod() {
    let m = Uint::<1>::from_u8(7).to_odd().unwrap();
    assert_eq!(
        Uint::<1>::from_u8(3).invert_odd_mod(&m).unwrap(),
        Uint::<1>::from_u8(5)
    );
}

#[cfg(feature = "alloc")]
mod boxed {
    use super::*;
    use crypto_bigint::{BoxedUint, Integer, NonZero, Resize};

    /// Borrowing the input must not change the value produced by the same resize request.
    #[kani::proof]
    #[kani::unwind(6)]
    fn regression_boxed_resize_owned_borrowed_zero_precision() {
        let input = BoxedUint::from(42u64);
        let borrowed = (&input).resize_unchecked(0);
        let owned = input.resize_unchecked(0);
        assert_eq!(owned, borrowed);
    }

    /// An unchecked resize must not return an even value inside an Odd wrapper.
    #[kani::proof]
    #[kani::unwind(6)]
    fn regression_boxed_resize_odd_preserves_invariant() {
        let input = BoxedUint::from(1u64).to_odd().unwrap();
        let result = input.resize_unchecked(0);
        assert!(result.as_ref().is_odd().to_bool());
    }

    /// Truncation must not return zero inside a NonZero wrapper, owned or borrowed.
    #[kani::proof]
    #[kani::unwind(6)]
    fn regression_boxed_resize_nonzero_preserves_invariant() {
        let input = NonZero::new(BoxedUint::from(1u128 << 64)).unwrap();
        // Keep each desired invariant reachable independently of the other failing assertion.
        if kani::any() {
            let result = (&input).resize_unchecked(64);
            assert!(result.as_ref().is_nonzero().to_bool());
        } else {
            let result = input.resize_unchecked(64);
            assert!(result.as_ref().is_nonzero().to_bool());
        }
    }

    /// max promises the maximum at the result's precision, which is one word even for request 0.
    #[kani::proof]
    #[kani::unwind(3)]
    fn regression_boxed_max_zero_precision() {
        let result = BoxedUint::max(0);
        assert_eq!(result.bits_precision(), Word::BITS);
        assert_eq!(result.as_limbs()[0].0, Word::MAX);
    }

    /// Checked decoding promises a precision error instead of discarding a set high bit.
    #[kani::proof]
    #[kani::unwind(10)]
    fn regression_boxed_decode_precision_be() {
        assert_eq!(
            BoxedUint::from_be_slice(&[2], 1),
            Err(crypto_bigint::DecodeError::Precision)
        );
    }

    #[kani::proof]
    #[kani::unwind(10)]
    fn regression_boxed_decode_precision_le() {
        assert_eq!(
            BoxedUint::from_le_slice(&[2], 1),
            Err(crypto_bigint::DecodeError::Precision)
        );
    }

    /// FINDINGS 1.9: a parsed zero must have a limb before it enters arithmetic.
    #[kani::proof]
    #[kani::unwind(20)]
    fn regression_boxed_parse_zero_has_precision() {
        let zero = BoxedUint::from_str_radix_vartime("0", 10).unwrap();
        assert!(zero.nlimbs() >= 1);
        assert!(zero.is_zero().to_bool());
    }
}
