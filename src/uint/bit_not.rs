//! [`Uint`] bitwise NOT operations.

use super::Uint;
use crate::Limb;
use core::ops::Not;

impl<const LIMBS: usize> Uint<LIMBS> {
    /// Computes bitwise `!a`.
    #[inline(always)]
    #[must_use]
    pub const fn not(&self) -> Self {
        let mut limbs = [Limb::ZERO; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            limbs[i] = self.limbs[i].not();
            i += 1;
        }

        Self { limbs }
    }
}

impl<const LIMBS: usize> Not for Uint<LIMBS> {
    type Output = Self;

    fn not(self) -> Self {
        Self::not(&self)
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::U128;

    #[kani::proof]
    #[kani::unwind(5)]
    fn uint_bitnot_u128_correct() {
        let a: u128 = kani::any();
        let x = U128::from_u128(a);
        assert_eq!(u128::from(x.not()), !a);
        assert_eq!(u128::from(!x), !a);
    }
}

#[cfg(test)]
mod tests {
    use crate::U128;

    #[test]
    fn bitnot_ok() {
        assert_eq!(U128::ZERO.not(), U128::MAX);
        assert_eq!(U128::MAX.not(), U128::ZERO);
    }
}
