//! [`Uint`] bitwise XOR operations.

use super::Uint;
use crate::{BitXor, BitXorAssign, CtOption, Limb};

impl<const LIMBS: usize> Uint<LIMBS> {
    /// Computes bitwise `a ^ b`.
    #[inline(always)]
    #[must_use]
    pub const fn bitxor(&self, rhs: &Self) -> Self {
        let mut limbs = [Limb::ZERO; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            limbs[i] = self.limbs[i].bitxor(rhs.limbs[i]);
            i += 1;
        }

        Self { limbs }
    }

    /// Perform wrapping bitwise `XOR`.
    ///
    /// There's no way wrapping could ever happen.
    /// This function exists so that all operations are accounted for in the wrapping operations
    #[must_use]
    pub const fn wrapping_xor(&self, rhs: &Self) -> Self {
        self.bitxor(rhs)
    }

    /// Perform checked bitwise `XOR`, returning a [`CtOption`] which `is_some` always
    #[must_use]
    pub const fn checked_xor(&self, rhs: &Self) -> CtOption<Self> {
        CtOption::some(self.bitxor(rhs))
    }
}

impl<const LIMBS: usize> BitXor for Uint<LIMBS> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Uint<LIMBS> {
        self.bitxor(&rhs)
    }
}

impl<const LIMBS: usize> BitXor<&Uint<LIMBS>> for Uint<LIMBS> {
    type Output = Uint<LIMBS>;

    #[allow(clippy::needless_borrow)]
    fn bitxor(self, rhs: &Uint<LIMBS>) -> Uint<LIMBS> {
        (&self).bitxor(rhs)
    }
}

impl<const LIMBS: usize> BitXor<Uint<LIMBS>> for &Uint<LIMBS> {
    type Output = Uint<LIMBS>;

    fn bitxor(self, rhs: Uint<LIMBS>) -> Uint<LIMBS> {
        self.bitxor(&rhs)
    }
}

impl<const LIMBS: usize> BitXor<&Uint<LIMBS>> for &Uint<LIMBS> {
    type Output = Uint<LIMBS>;

    fn bitxor(self, rhs: &Uint<LIMBS>) -> Uint<LIMBS> {
        self.bitxor(rhs)
    }
}

impl<const LIMBS: usize> BitXorAssign for Uint<LIMBS> {
    fn bitxor_assign(&mut self, other: Self) {
        *self = *self ^ other;
    }
}

impl<const LIMBS: usize> BitXorAssign<&Uint<LIMBS>> for Uint<LIMBS> {
    fn bitxor_assign(&mut self, other: &Self) {
        *self = *self ^ other;
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::U128;

    #[kani::proof]
    #[kani::unwind(5)]
    fn uint_bitxor_u128_correct() {
        let (a, b): (u128, u128) = kani::any();
        let (x, y) = (U128::from_u128(a), U128::from_u128(b));
        let expected = a ^ b;
        assert_eq!(u128::from(x.bitxor(&y)), expected);
        assert_eq!(u128::from(x.wrapping_xor(&y)), expected);
        assert_eq!(
            Option::<U128>::from(x.checked_xor(&y)).map(u128::from),
            Some(expected)
        );
        assert_eq!(u128::from(x ^ y), expected);
        assert_eq!(u128::from(x ^ &y), expected);
        assert_eq!(u128::from(&x ^ y), expected);
        assert_eq!(u128::from(&x ^ &y), expected);
        let mut assigned = x;
        assigned ^= y;
        assert_eq!(u128::from(assigned), expected);
        assigned = x;
        assigned ^= &y;
        assert_eq!(u128::from(assigned), expected);
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
