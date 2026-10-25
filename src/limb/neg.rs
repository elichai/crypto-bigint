//! Limb negation

use crate::{Limb, NegMod, NonZero, WrappingNeg};

impl Limb {
    /// Perform wrapping negation.
    #[inline(always)]
    #[must_use]
    pub const fn wrapping_neg(self) -> Self {
        Limb(self.0.wrapping_neg())
    }
}

impl WrappingNeg for Limb {
    #[inline]
    fn wrapping_neg(&self) -> Self {
        Self(self.0.wrapping_neg())
    }
}

impl NegMod for Limb {
    type Output = Self;

    fn neg_mod(&self, p: &NonZero<Self>) -> Self::Output {
        let nz = self.is_nonzero();
        let res = p.borrowing_sub(*self, Limb::ZERO).0;
        Self::select(Limb::ZERO, res, nz)
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{Limb, NegMod, NonZero, Word};

    #[kani::proof]
    fn limb_wrapping_neg_correct() {
        let a: Word = kani::any();
        assert_eq!(Limb(a).wrapping_neg().0, a.wrapping_neg());
    }

    /// For a reduced input `a < p`, the result is `-a mod p`.
    #[kani::proof]
    fn neg_mod_correct() {
        let (a, p): (Word, Word) = kani::any();
        kani::assume(a < p);
        let res = Limb(a).neg_mod(&NonZero::new(Limb(p)).unwrap());
        assert_eq!(res.0, if a == 0 { 0 } else { p - a });
    }
}
