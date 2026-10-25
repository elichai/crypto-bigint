//! [`BoxedUint`] bitwise AND operations.

use super::BoxedUint;
use crate::{BitAnd, BitAndAssign, CtOption, Limb};

impl BoxedUint {
    /// Computes bitwise `a & b`.
    #[inline(always)]
    #[must_use]
    pub fn bitand(&self, rhs: &Self) -> Self {
        Self::map_limbs(self, rhs, Limb::bitand)
    }

    /// Perform bitwise `AND` between `self` and the given [`Limb`], performing the `AND` operation
    /// on every limb of `self`.
    #[must_use]
    pub fn bitand_limb(&self, rhs: Limb) -> Self {
        Self {
            limbs: self.limbs.iter().map(|limb| limb.bitand(rhs)).collect(),
        }
    }

    /// Perform wrapping bitwise `AND`.
    ///
    /// There's no way wrapping could ever happen.
    /// This function exists so that all operations are accounted for in the wrapping operations
    #[must_use]
    pub fn wrapping_and(&self, rhs: &Self) -> Self {
        self.bitand(rhs)
    }

    /// Perform checked bitwise `AND`, returning a [`CtOption`] which `is_some` always
    #[must_use]
    pub fn checked_and(&self, rhs: &Self) -> CtOption<Self> {
        CtOption::some(self.bitand(rhs))
    }
}

impl BitAnd for BoxedUint {
    type Output = Self;

    fn bitand(self, rhs: Self) -> BoxedUint {
        self.bitand(&rhs)
    }
}

impl BitAnd<&BoxedUint> for BoxedUint {
    type Output = BoxedUint;

    #[allow(clippy::needless_borrow)]
    fn bitand(self, rhs: &BoxedUint) -> BoxedUint {
        (&self).bitand(rhs)
    }
}

impl BitAnd<BoxedUint> for &BoxedUint {
    type Output = BoxedUint;

    fn bitand(self, rhs: BoxedUint) -> BoxedUint {
        self.bitand(&rhs)
    }
}

impl BitAnd<&BoxedUint> for &BoxedUint {
    type Output = BoxedUint;

    fn bitand(self, rhs: &BoxedUint) -> BoxedUint {
        self.bitand(rhs)
    }
}

impl BitAndAssign for BoxedUint {
    #[allow(clippy::assign_op_pattern)]
    fn bitand_assign(&mut self, other: Self) {
        *self = BoxedUint::bitand(self, &other);
    }
}

impl BitAndAssign<&BoxedUint> for BoxedUint {
    #[allow(clippy::assign_op_pattern)]
    fn bitand_assign(&mut self, other: &Self) {
        *self = BoxedUint::bitand(self, other);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{BoxedUint, Limb, Word, Wrapping};

    // Separate calls keep 64/128-bit heap lengths concrete for Kani.
    fn check128(result: BoxedUint, expected: u128) {
        assert_eq!((value(&result), result.bits_precision()), (expected, 128));
    }

    fn check_methods(lhs: &BoxedUint, rhs: &BoxedUint, expected: u128) {
        check128(lhs.bitand(rhs), expected);
        check128(lhs.wrapping_and(rhs), expected);
        check128(
            lhs.checked_and(rhs).into_option().expect("always Some"),
            expected,
        );
    }

    fn check_operators(lhs: &BoxedUint, rhs: &BoxedUint, expected: u128) {
        check128(lhs & rhs, expected);
        check128(lhs.clone() & rhs, expected);
        check128(lhs & rhs.clone(), expected);
        check128(lhs.clone() & rhs.clone(), expected);
        let mut assigned = lhs.clone();
        assigned &= rhs;
        check128(assigned, expected);
        let mut assigned = lhs.clone();
        assigned &= rhs.clone();
        check128(assigned, expected);
        let mut wrapped = Wrapping(lhs.clone());
        let wrapped_rhs = Wrapping(rhs.clone());
        check128((&wrapped & &wrapped_rhs).0, expected);
        wrapped &= wrapped_rhs;
        check128(wrapped.0, expected);
    }

    /// Both operand orders zero-extend 64/128-bit inputs and retain 128-bit output precision.
    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_bitand_methods_mixed_precision() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let expected = value(&a) & value(&b);
        check_methods(&a, &b, expected);
        check_methods(&b, &a, expected);
    }

    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_bitand_operators_mixed_precision() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let expected = value(&a) & value(&b);
        check_operators(&a, &b, expected);
        check_operators(&b, &a, expected);
    }

    fn check_limb(input: BoxedUint, limb: Word, repeated: u128) {
        let expected = (value(&input) & repeated, input.bits_precision());
        let result = input.bitand_limb(Limb(limb));
        assert_eq!((value(&result), result.bits_precision()), expected);
    }

    /// The limb mask repeats across the entire 64/128-bit value.
    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_bitand_limb_repeated_mask() {
        let limb: Word = kani::any();
        let mut repeated = u128::from(limb);
        if Word::BITS == 32 {
            repeated |= repeated << 32;
        }
        repeated |= repeated << 64;
        check_limb(any_boxed::<L64>(), limb, repeated);
        check_limb(any_boxed::<{ 2 * L64 }>(), limb, repeated);
    }
}

#[cfg(test)]
mod tests {
    use crate::BoxedUint;

    #[test]
    fn checked_and_ok() {
        let result = BoxedUint::zero().checked_and(&BoxedUint::one());
        assert_eq!(result.unwrap(), BoxedUint::zero());
    }

    #[test]
    fn overlapping_and_ok() {
        let result = BoxedUint::max(128).wrapping_and(&BoxedUint::one());
        assert_eq!(result, BoxedUint::one());
    }
}
