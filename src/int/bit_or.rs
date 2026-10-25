//! [`Int`] bitwise OR operations.

use core::ops::{BitOr, BitOrAssign};

use crate::{CtOption, Uint};

use super::Int;

impl<const LIMBS: usize> Int<LIMBS> {
    /// Computes bitwise `a | b`.
    #[inline(always)]
    #[must_use]
    pub const fn bitor(&self, rhs: &Self) -> Self {
        Self(Uint::bitor(&self.0, &rhs.0))
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

impl<const LIMBS: usize> BitOr for Int<LIMBS> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Int<LIMBS> {
        self.bitor(&rhs)
    }
}

impl<const LIMBS: usize> BitOr<&Int<LIMBS>> for Int<LIMBS> {
    type Output = Int<LIMBS>;

    #[allow(clippy::needless_borrow)]
    fn bitor(self, rhs: &Int<LIMBS>) -> Int<LIMBS> {
        (&self).bitor(rhs)
    }
}

impl<const LIMBS: usize> BitOr<Int<LIMBS>> for &Int<LIMBS> {
    type Output = Int<LIMBS>;

    fn bitor(self, rhs: Int<LIMBS>) -> Int<LIMBS> {
        self.bitor(&rhs)
    }
}

impl<const LIMBS: usize> BitOr<&Int<LIMBS>> for &Int<LIMBS> {
    type Output = Int<LIMBS>;

    fn bitor(self, rhs: &Int<LIMBS>) -> Int<LIMBS> {
        self.bitor(rhs)
    }
}

impl<const LIMBS: usize> BitOrAssign for Int<LIMBS> {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl<const LIMBS: usize> BitOrAssign<&Int<LIMBS>> for Int<LIMBS> {
    fn bitor_assign(&mut self, other: &Self) {
        *self = *self | other;
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::I128;

    #[kani::proof]
    #[kani::unwind(5)]
    fn int_bitor_native() {
        let (a, b): (i128, i128) = kani::any();
        let (ia, ib) = (I128::from_i128(a), I128::from_i128(b));
        assert_eq!(i128::from(ia.bitor(&ib)), a | b);
        assert_eq!(i128::from(ia.wrapping_or(&ib)), a | b);
        assert_eq!(
            Option::<I128>::from(ia.checked_or(&ib)).map(i128::from),
            Some(a | b)
        );
        assert_eq!(i128::from(ia | ib), a | b);
        assert_eq!(i128::from(&ia | ib), a | b);
        assert_eq!(i128::from(ia | &ib), a | b);
        assert_eq!(i128::from(&ia | &ib), a | b);
        let mut assigned = ia;
        assigned |= ib;
        assert_eq!(i128::from(assigned), a | b);
        assigned = ia;
        assigned |= &ib;
        assert_eq!(i128::from(assigned), a | b);
    }
}

#[cfg(test)]
mod tests {
    use crate::I128;

    #[test]
    fn checked_or_ok() {
        assert_eq!(I128::ZERO.checked_or(&I128::ONE).unwrap(), I128::ONE);
        assert_eq!(I128::ONE.checked_or(&I128::ONE).unwrap(), I128::ONE);
        assert_eq!(I128::MAX.checked_or(&I128::ONE).unwrap(), I128::MAX);
    }

    #[test]
    fn wrapping_or_ok() {
        assert_eq!(I128::ZERO.wrapping_or(&I128::ONE), I128::ONE);
        assert_eq!(I128::ONE.wrapping_or(&I128::ONE), I128::ONE);
        assert_eq!(I128::MAX.wrapping_or(&I128::ONE), I128::MAX);
    }
}
