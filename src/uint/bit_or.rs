//! [`Uint`] bitwise OR operations.

use super::Uint;
use crate::{BitOr, BitOrAssign, CtOption, Limb};

impl<const LIMBS: usize> Uint<LIMBS> {
    /// Computes bitwise `a | b`.
    #[inline(always)]
    #[must_use]
    pub const fn bitor(&self, rhs: &Self) -> Self {
        let mut limbs = [Limb::ZERO; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            limbs[i] = self.limbs[i].bitor(rhs.limbs[i]);
            i += 1;
        }

        Self { limbs }
    }

    /// Perform wrapping bitwise `OR`.
    ///
    /// There's no way wrapping could ever happen.
    /// This function exists so that all operations are accounted for in the wrapping operations
    #[must_use]
    pub const fn wrapping_or(&self, rhs: &Self) -> Self {
        self.bitor(rhs)
    }

    /// Perform checked bitwise `OR`, returning a [`CtOption`] which `is_some` always
    #[must_use]
    pub const fn checked_or(&self, rhs: &Self) -> CtOption<Self> {
        CtOption::some(self.bitor(rhs))
    }
}

impl<const LIMBS: usize> BitOr for Uint<LIMBS> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Uint<LIMBS> {
        self.bitor(&rhs)
    }
}

impl<const LIMBS: usize> BitOr<&Uint<LIMBS>> for Uint<LIMBS> {
    type Output = Uint<LIMBS>;

    #[allow(clippy::needless_borrow)]
    fn bitor(self, rhs: &Uint<LIMBS>) -> Uint<LIMBS> {
        (&self).bitor(rhs)
    }
}

impl<const LIMBS: usize> BitOr<Uint<LIMBS>> for &Uint<LIMBS> {
    type Output = Uint<LIMBS>;

    fn bitor(self, rhs: Uint<LIMBS>) -> Uint<LIMBS> {
        self.bitor(&rhs)
    }
}

impl<const LIMBS: usize> BitOr<&Uint<LIMBS>> for &Uint<LIMBS> {
    type Output = Uint<LIMBS>;

    fn bitor(self, rhs: &Uint<LIMBS>) -> Uint<LIMBS> {
        self.bitor(rhs)
    }
}

impl<const LIMBS: usize> BitOrAssign for Uint<LIMBS> {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl<const LIMBS: usize> BitOrAssign<&Uint<LIMBS>> for Uint<LIMBS> {
    fn bitor_assign(&mut self, other: &Self) {
        *self = *self | other;
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::U128;

    #[kani::proof]
    #[kani::unwind(5)]
    fn uint_bitor_u128_correct() {
        let (a, b): (u128, u128) = kani::any();
        let (x, y) = (U128::from_u128(a), U128::from_u128(b));
        let expected = a | b;
        assert_eq!(u128::from(x.bitor(&y)), expected);
        assert_eq!(u128::from(x.wrapping_or(&y)), expected);
        assert_eq!(
            Option::<U128>::from(x.checked_or(&y)).map(u128::from),
            Some(expected)
        );
        assert_eq!(u128::from(x | y), expected);
        assert_eq!(u128::from(x | &y), expected);
        assert_eq!(u128::from(&x | y), expected);
        assert_eq!(u128::from(&x | &y), expected);
        let mut assigned = x;
        assigned |= y;
        assert_eq!(u128::from(assigned), expected);
        assigned = x;
        assigned |= &y;
        assert_eq!(u128::from(assigned), expected);
    }
}

#[cfg(test)]
mod tests {
    use crate::U128;

    #[test]
    fn checked_or_ok() {
        let result = U128::ZERO.checked_or(&U128::ONE);
        assert_eq!(result.unwrap(), U128::ONE);
    }

    #[test]
    fn overlapping_or_ok() {
        let result = U128::MAX.wrapping_or(&U128::ONE);
        assert_eq!(result, U128::MAX);
    }
}
