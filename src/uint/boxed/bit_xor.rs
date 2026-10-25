//! [`BoxedUint`] bitwise XOR operations.

use super::BoxedUint;
use crate::{BitXor, BitXorAssign, CtOption};

impl BoxedUint {
    /// Computes bitwise `a ^ b`.
    #[inline(always)]
    #[must_use]
    pub fn bitxor(&self, rhs: &Self) -> Self {
        Self::map_limbs(self, rhs, crate::limb::Limb::bitxor)
    }

    /// Perform wrapping bitwise `XOR`.
    ///
    /// There's no way wrapping could ever happen.
    /// This function exists so that all operations are accounted for in the wrapping operations
    #[must_use]
    pub fn wrapping_xor(&self, rhs: &Self) -> Self {
        self.bitxor(rhs)
    }

    /// Perform checked bitwise `XOR`, returning a [`CtOption`] which `is_some` always
    #[must_use]
    pub fn checked_xor(&self, rhs: &Self) -> CtOption<Self> {
        CtOption::some(self.bitxor(rhs))
    }
}

impl BitXor for BoxedUint {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> BoxedUint {
        self.bitxor(&rhs)
    }
}

impl BitXor<&BoxedUint> for BoxedUint {
    type Output = BoxedUint;

    fn bitxor(self, rhs: &BoxedUint) -> BoxedUint {
        Self::bitxor(&self, rhs)
    }
}

impl BitXor<BoxedUint> for &BoxedUint {
    type Output = BoxedUint;

    fn bitxor(self, rhs: BoxedUint) -> BoxedUint {
        self.bitxor(&rhs)
    }
}

impl BitXor<&BoxedUint> for &BoxedUint {
    type Output = BoxedUint;

    fn bitxor(self, rhs: &BoxedUint) -> BoxedUint {
        self.bitxor(rhs)
    }
}

impl BitXorAssign for BoxedUint {
    fn bitxor_assign(&mut self, other: Self) {
        *self = Self::bitxor(self, &other);
    }
}

impl BitXorAssign<&BoxedUint> for BoxedUint {
    fn bitxor_assign(&mut self, other: &Self) {
        *self = Self::bitxor(self, other);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{BoxedUint, Wrapping};

    // Separate calls keep 64/128-bit heap lengths concrete for Kani.
    fn check128(result: BoxedUint, expected: u128) {
        assert_eq!((value(&result), result.bits_precision()), (expected, 128));
    }

    fn check_methods(lhs: &BoxedUint, rhs: &BoxedUint, expected: u128) {
        check128(lhs.bitxor(rhs), expected);
        check128(lhs.wrapping_xor(rhs), expected);
        check128(
            lhs.checked_xor(rhs).into_option().expect("always Some"),
            expected,
        );
    }

    fn check_operators(lhs: &BoxedUint, rhs: &BoxedUint, expected: u128) {
        check128(lhs ^ rhs, expected);
        check128(lhs.clone() ^ rhs, expected);
        check128(lhs ^ rhs.clone(), expected);
        check128(lhs.clone() ^ rhs.clone(), expected);
        let mut assigned = lhs.clone();
        assigned ^= rhs;
        check128(assigned, expected);
        let mut assigned = lhs.clone();
        assigned ^= rhs.clone();
        check128(assigned, expected);
        let mut wrapped = Wrapping(lhs.clone());
        let wrapped_rhs = Wrapping(rhs.clone());
        check128((&wrapped ^ &wrapped_rhs).0, expected);
        wrapped ^= wrapped_rhs;
        check128(wrapped.0, expected);
    }

    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_bitxor_methods_mixed_precision() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let expected = value(&a) ^ value(&b);
        check_methods(&a, &b, expected);
        check_methods(&b, &a, expected);
    }

    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_bitxor_operators_mixed_precision() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let expected = value(&a) ^ value(&b);
        check_operators(&a, &b, expected);
        check_operators(&b, &a, expected);
    }
}

#[cfg(test)]
mod tests {
    use crate::U128;

    #[test]
    fn checked_xor_ok() {
        let result = U128::ZERO.checked_xor(&U128::ONE);
        assert_eq!(result.unwrap(), U128::ONE);
    }

    #[test]
    fn overlapping_xor_ok() {
        let result = U128::ZERO.wrapping_xor(&U128::ONE);
        assert_eq!(result, U128::ONE);
    }
}
