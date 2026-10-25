//! Limb bit and operations.

use super::Limb;
use core::ops::{BitAnd, BitAndAssign};

impl Limb {
    /// Calculates `a & b`.
    #[inline(always)]
    #[must_use]
    pub const fn bitand(self, rhs: Self) -> Self {
        Limb(self.0 & rhs.0)
    }
}

impl BitAnd for Limb {
    type Output = Limb;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.bitand(rhs)
    }
}

impl BitAnd<&Limb> for Limb {
    type Output = Limb;

    #[inline(always)]
    fn bitand(self, rhs: &Self) -> Self::Output {
        self.bitand(*rhs)
    }
}

impl BitAndAssign for Limb {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitAndAssign<&Limb> for Limb {
    fn bitand_assign(&mut self, rhs: &Limb) {
        self.0 &= rhs.0;
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{Limb, Word};

    /// All word values, including owned/borrowed operators and assignments.
    #[kani::proof]
    fn limb_bitand_native() {
        let (a, b): (Word, Word) = kani::any();
        let expected = a & b;
        assert_eq!((Limb(a) & Limb(b)).0, expected);
        assert_eq!((Limb(a) & &Limb(b)).0, expected);
        let mut owned = Limb(a);
        owned &= Limb(b);
        assert_eq!(owned.0, expected);
        let mut borrowed = Limb(a);
        borrowed &= &Limb(b);
        assert_eq!(borrowed.0, expected);
    }
}
