//! Support for computing greatest common divisor of two `BoxedUint`s.

use super::BoxedUint;
use crate::{Gcd, NonZero, Odd, modular::safegcd};

impl Gcd for BoxedUint {
    type Output = Self;

    /// Compute the greatest common divisor (GCD) of this number and another.
    fn gcd(&self, rhs: &Self) -> Self {
        safegcd::boxed::gcd::<false>(self, rhs)
    }

    fn gcd_vartime(&self, rhs: &Self) -> Self::Output {
        safegcd::boxed::gcd::<true>(self, rhs)
    }
}

impl Gcd<BoxedUint> for NonZero<BoxedUint> {
    type Output = NonZero<BoxedUint>;

    fn gcd(&self, rhs: &BoxedUint) -> Self::Output {
        safegcd::boxed::gcd_nz::<false>(self, rhs)
    }

    fn gcd_vartime(&self, rhs: &BoxedUint) -> Self::Output {
        safegcd::boxed::gcd_nz::<true>(self, rhs)
    }
}

impl Gcd<BoxedUint> for Odd<BoxedUint> {
    type Output = Odd<BoxedUint>;

    fn gcd(&self, rhs: &BoxedUint) -> Self::Output {
        safegcd::boxed::gcd_odd::<false>(self, rhs)
    }

    fn gcd_vartime(&self, rhs: &BoxedUint) -> Self::Output {
        safegcd::boxed::gcd_odd::<true>(self, rhs)
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{BoxedUint, Gcd, Word};

    /// `gcd(0, x) = x`, with the maximum operand precision, including `x = 0`.
    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_gcd_vartime_zero_lhs_mixed_precision() {
        let (narrow, wide) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let (vn, vw) = (value(&narrow), value(&wide));
        let zero_narrow = BoxedUint::zero_with_precision(64);
        let zero_wide = BoxedUint::zero_with_precision(128);
        let result = zero_narrow.gcd_vartime(&wide);
        assert_eq!((value(&result), result.bits_precision()), (vw, 128));
        let result = zero_wide.gcd_vartime(&narrow);
        assert_eq!((value(&result), result.bits_precision()), (vn, 128));
    }

    /// `gcd(x, 0) = x`, even when trailing zeros exceed the zero operand's precision.
    /// Split operand precisions so each proof contains only one normalization path.
    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_gcd_vartime_zero_narrow_rhs() {
        let wide = any_boxed::<{ 2 * L64 }>();
        let expected = value(&wide);
        let zero_narrow = BoxedUint::zero_with_precision(64);
        let result = wide.gcd_vartime(&zero_narrow);
        assert_eq!((value(&result), result.bits_precision()), (expected, 128));
    }

    /// `gcd(x, 0) = x`, widened to the zero operand's precision when necessary.
    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_gcd_vartime_zero_wide_rhs() {
        let narrow = any_boxed::<L64>();
        let expected = value(&narrow);
        let zero_wide = BoxedUint::zero_with_precision(128);
        let result = narrow.gcd_vartime(&zero_wide);
        assert_eq!((value(&result), result.bits_precision()), (expected, 128));
    }

    /// Every odd input at the selected precision; both mixed-precision results have 128 bits.
    fn check_odd_zero<const N: usize>(zero_bits: u32) {
        let mut words: [Word; N] = kani::any();
        words[0] |= 1;
        let input = BoxedUint::from_words(words);
        let expected = value(&input);
        let odd = input.to_odd().unwrap();
        let zero = BoxedUint::zero_with_precision(zero_bits);
        let result = odd.gcd_vartime(&zero).get();
        assert_eq!((value(&result), result.bits_precision()), (expected, 128));
    }

    #[kani::proof]
    #[kani::unwind(9)]
    #[kani::solver(z3)]
    fn boxed_gcd_vartime_odd_zero_wide_rhs() {
        check_odd_zero::<L64>(128);
    }

    #[kani::proof]
    #[kani::unwind(9)]
    #[kani::solver(z3)]
    fn boxed_gcd_vartime_odd_zero_narrow_rhs() {
        check_odd_zero::<{ 2 * L64 }>(64);
    }
}

#[cfg(test)]
mod tests {
    use crate::{BoxedUint, Gcd, Limb, Resize};

    #[test]
    fn gcd_relatively_prime() {
        // Two semiprimes with no common factors
        let f = BoxedUint::from(59u32 * 67).to_odd().unwrap();
        let g = BoxedUint::from(61u32 * 71);
        let gcd = f.gcd(&g);
        assert_eq!(gcd.get(), BoxedUint::one());
    }

    #[test]
    fn gcd_nonprime() {
        let f = BoxedUint::from(4391633u32).to_odd().unwrap();
        let g = BoxedUint::from(2022161u32);
        let gcd = f.gcd(&g);
        assert_eq!(gcd.get(), BoxedUint::from(1763u32));
    }

    #[test]
    fn gcd_zero() {
        let zero = BoxedUint::from(0u32);
        let one = BoxedUint::from(1u32);

        assert_eq!(zero.gcd(&zero), zero);
        assert_eq!(zero.gcd(&one), one);
        assert_eq!(one.gcd(&zero), one);
    }

    #[test]
    fn gcd_zero_lhs_wider_than_rhs() {
        let zero = BoxedUint::zero_with_precision(2 * Limb::BITS);
        let one = BoxedUint::one();

        let gcd = zero.gcd(&one);
        assert_eq!(gcd, one);
        assert_eq!(gcd.bits_precision(), zero.bits_precision());
    }

    #[test]
    fn gcd_zero_rhs_wider_than_lhs() {
        let one = BoxedUint::one();
        let zero = BoxedUint::zero_with_precision(2 * Limb::BITS);

        let gcd = one.gcd(&zero);
        assert_eq!(gcd, one);
    }

    #[test]
    fn gcd_zero_rhs_narrower_than_power_of_two_lhs() {
        let x = BoxedUint::one_with_precision(3 * Limb::BITS).wrapping_shl_vartime(Limb::BITS + 1);
        let zero = BoxedUint::zero_with_precision(Limb::BITS);

        let gcd = x.gcd(&zero);
        assert_eq!(gcd, x);
    }

    #[test]
    fn gcd_vartime_zero_rhs_narrower_than_power_of_two_lhs() {
        let x = BoxedUint::one_with_precision(3 * Limb::BITS).wrapping_shl_vartime(Limb::BITS + 1);
        let zero = BoxedUint::zero_with_precision(Limb::BITS);

        let gcd = x.gcd_vartime(&zero);
        assert_eq!(gcd, x);
    }

    #[test]
    fn gcd_nonzero_lhs_wider_than_rhs_precision() {
        let wide = BoxedUint::from(6u8).resize(2 * Limb::BITS);
        let narrow = BoxedUint::from(4u8);

        let gcd = wide.gcd(&narrow);
        assert_eq!(gcd, BoxedUint::from(2u8));
        assert_eq!(gcd.bits_precision(), wide.bits_precision());
    }

    #[test]
    fn gcd_nonzero_rhs_wider_than_lhs_precision() {
        let narrow = BoxedUint::from(4u8);
        let wide = BoxedUint::from(6u8).resize(2 * Limb::BITS);

        let gcd = narrow.gcd(&wide);
        assert_eq!(gcd, BoxedUint::from(2u8));
        assert_eq!(gcd.bits_precision(), wide.bits_precision());
    }

    #[test]
    fn gcd_vartime_zero_lhs_wider_than_rhs() {
        let zero = BoxedUint::zero_with_precision(2 * Limb::BITS);
        let one = BoxedUint::one();

        let gcd = zero.gcd_vartime(&one);
        assert_eq!(gcd, one);
        assert_eq!(gcd.bits_precision(), zero.bits_precision());
    }

    #[test]
    fn gcd_vartime_zero_rhs_wider_than_lhs() {
        let one = BoxedUint::one();
        let zero = BoxedUint::zero_with_precision(2 * Limb::BITS);

        let gcd = one.gcd_vartime(&zero);
        assert_eq!(gcd, one);
        assert_eq!(gcd.bits_precision(), zero.bits_precision());
    }

    #[test]
    fn gcd_zero_zero_mixed_precision() {
        let narrow = BoxedUint::zero_with_precision(Limb::BITS);
        let wide = BoxedUint::zero_with_precision(2 * Limb::BITS);

        let gcd = narrow.gcd(&wide);
        assert_eq!(gcd, narrow);
        assert_eq!(gcd.bits_precision(), wide.bits_precision());
        assert_eq!(gcd, wide.gcd(&narrow));
    }

    #[test]
    fn gcd_vartime_zero_zero_mixed_precision() {
        let narrow = BoxedUint::zero_with_precision(Limb::BITS);
        let wide = BoxedUint::zero_with_precision(2 * Limb::BITS);

        let gcd = narrow.gcd_vartime(&wide);
        assert_eq!(gcd, narrow);
        assert_eq!(gcd.bits_precision(), wide.bits_precision());
        assert_eq!(gcd, wide.gcd_vartime(&narrow));
    }

    #[test]
    fn gcd_one() {
        let f = BoxedUint::from(1u32);
        assert_eq!(BoxedUint::from(1u32), f.gcd(&BoxedUint::from(1u32)));
        assert_eq!(BoxedUint::from(1u32), f.gcd(&BoxedUint::from(2u8)));
    }

    #[test]
    fn gcd_two() {
        let f = BoxedUint::from(2u32);
        assert_eq!(f, f.gcd(&f));

        let g = BoxedUint::from(4u32);
        assert_eq!(f, f.gcd(&g));
        assert_eq!(f, g.gcd(&f));
    }

    #[test]
    fn gcd_different_sizes() {
        // Test that gcd works for boxed Uints with different numbers of limbs
        let f = BoxedUint::from(4391633u32).resize(128).to_odd().unwrap();
        let g = BoxedUint::from(2022161u32);
        let gcd = f.gcd(&g);
        assert_eq!(gcd.get(), BoxedUint::from(1763u32));
    }

    #[test]
    fn gcd_vartime_different_sizes() {
        // Test that gcd works for boxed Uints with different numbers of limbs
        let f = BoxedUint::from(4391633u32).resize(128).to_odd().unwrap();
        let g = BoxedUint::from(2022161u32);
        let gcd = f.gcd_vartime(&g);
        assert_eq!(gcd.get(), BoxedUint::from(1763u32));
    }
}
