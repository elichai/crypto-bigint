//! Constant-time support: impls of `Ct*` traits and constant-time `const fn` operations.

use super::UintRef;
use crate::{Choice, CtAssign, CtEq, CtLt};

impl CtAssign for UintRef {
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        assert!(self.bits_precision() >= other.bits_precision());
        let (lo, hi) = self.split_at_mut(other.nlimbs());
        lo.limbs.ct_assign(&other.limbs, choice);
        hi.conditional_set_zero(choice);
    }
}

impl<Rhs: AsRef<UintRef> + ?Sized> CtEq<Rhs> for UintRef {
    #[inline]
    fn ct_eq(&self, other: &Rhs) -> Choice {
        let rhs = other.as_ref();
        let overlap = if self.nlimbs() < rhs.nlimbs() {
            self.nlimbs()
        } else {
            rhs.nlimbs()
        };
        let (lhs, lhs_over) = self.split_at(overlap);
        let (rhs, rhs_over) = rhs.split_at(overlap);
        lhs.limbs
            .ct_eq(&rhs.limbs)
            .and(lhs_over.is_zero())
            .and(rhs_over.is_zero())
    }
}

impl CtLt for UintRef {
    #[inline]
    fn ct_lt(&self, other: &Self) -> Choice {
        Self::lt(self, other)
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::UintRef;
    use crate::uint::ref_type::kani_util::{check, value};
    use crate::{Choice, CtAssign, CtEq, CtLt, Limb, Word};

    /// Numeric comparison permits unequal widths; assignment zero-extends into the wider view.
    #[kani::proof]
    #[kani::unwind(5)]
    fn uint_ref_ct_mixed_lengths() {
        let a = kani::any::<[Word; 2]>().map(Limb);
        let b = kani::any::<[Word; 2]>().map(Limb);
        let (alen, blen, enabled): (usize, usize, bool) = kani::any();
        kani::assume(alen <= 2 && blen <= 2);
        let av = value(&a[..alen]);
        let bv = value(&b[..blen]);
        let lhs = UintRef::new(&a[..alen]);
        let rhs = UintRef::new(&b[..blen]);
        assert_eq!(lhs.ct_eq(rhs).to_bool(), av == bv);
        assert_eq!(rhs.ct_eq(lhs).to_bool(), bv == av);
        assert_eq!(lhs.ct_lt(rhs).to_bool(), av < bv);
        assert_eq!(rhs.ct_lt(lhs).to_bool(), bv < av);
        if alen >= blen {
            let mut out = a;
            UintRef::new_mut(&mut out[..alen]).ct_assign(rhs, Choice::from(u8::from(enabled)));
            check(&out, &a, alen, if enabled { bv } else { av });
        }
    }
}

#[cfg(test)]
mod tests {
    use ctutils::CtLt;

    use super::UintRef;
    use crate::{CtEq, Limb};

    #[test]
    fn ct_eq() {
        let z_small = UintRef::new(&[Limb::ZERO, Limb::ZERO]);
        let z_big = UintRef::new(&[Limb::ZERO, Limb::ZERO, Limb::ZERO]);
        let a = UintRef::new(&[Limb::ZERO, Limb::ZERO, Limb::ONE]);

        assert!(z_small.ct_eq(z_big).to_bool());
        assert!(z_big.ct_eq(z_small).to_bool());
        assert!(a.ct_eq(a).to_bool());
        assert!(!z_small.ct_eq(a).to_bool());
        assert!(!a.ct_eq(z_small).to_bool());
        assert!(!z_big.ct_eq(a).to_bool());
        assert!(!a.ct_eq(z_big).to_bool());
    }

    #[test]
    fn ct_lt() {
        let lesser = UintRef::new(&[Limb::ZERO, Limb::ZERO, Limb::ZERO]);
        let greater = UintRef::new(&[Limb::ZERO, Limb::ONE, Limb::ZERO]);
        assert!(lesser.ct_lt(greater).to_bool());
        assert!(!greater.ct_lt(lesser).to_bool());

        let smaller = UintRef::new(&[Limb::ZERO, Limb::ZERO]);
        let bigger = UintRef::new(&[Limb::ZERO, Limb::ZERO, Limb::ONE]);
        assert!(smaller.ct_lt(bigger).to_bool());
        assert!(!bigger.ct_lt(smaller).to_bool());
    }
}
