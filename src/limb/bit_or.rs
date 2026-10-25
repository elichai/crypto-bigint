//! Limb bit or operations.

use super::Limb;
use core::ops::{BitOr, BitOrAssign};

impl Limb {
    /// Calculates `a | b`.
    #[inline(always)]
    #[must_use]
    pub const fn bitor(self, rhs: Self) -> Self {
        Limb(self.0 | rhs.0)
    }
}

impl BitOr for Limb {
    type Output = Limb;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.bitor(rhs)
    }
}

impl BitOr<&Self> for Limb {
    type Output = Limb;

    fn bitor(self, rhs: &Self) -> Self::Output {
        self.bitor(*rhs)
    }
}

impl BitOrAssign for Limb {
    fn bitor_assign(&mut self, other: Self) {
        *self = self.bitor(other);
    }
}

impl BitOrAssign<&Limb> for Limb {
    fn bitor_assign(&mut self, other: &Self) {
        *self = self.bitor(*other);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{Limb, Word};

    /// All word values, including owned/borrowed operators and assignments.
    #[kani::proof]
    fn limb_bitor_native() {
        let (a, b): (Word, Word) = kani::any();
        let expected = a | b;
        assert_eq!((Limb(a) | Limb(b)).0, expected);
        assert_eq!((Limb(a) | &Limb(b)).0, expected);
        let mut owned = Limb(a);
        owned |= Limb(b);
        assert_eq!(owned.0, expected);
        let mut borrowed = Limb(a);
        borrowed |= &Limb(b);
        assert_eq!(borrowed.0, expected);
    }
}
