//! Constant-time support: impls of `Ct*` traits.

use super::BoxedUint;
use crate::{Choice, CtAssign, CtEq, CtGt, CtLt, CtNeg, CtSelect, Limb, UintRef};
use ctutils::{CtAssignSlice, CtEqSlice};

impl CtAssign for BoxedUint {
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        self.as_mut_uint_ref()
            .ct_assign(other.as_uint_ref(), choice);
    }
}
impl CtAssignSlice for BoxedUint {}

impl<Rhs: AsRef<UintRef> + ?Sized> CtEq<Rhs> for BoxedUint {
    #[inline]
    fn ct_eq(&self, other: &Rhs) -> Choice {
        self.as_uint_ref().ct_eq(other)
    }
}
impl CtEqSlice for BoxedUint {}

impl CtGt for BoxedUint {
    #[inline]
    fn ct_gt(&self, other: &Self) -> Choice {
        UintRef::lt(other.as_uint_ref(), self.as_uint_ref())
    }
}

impl CtLt for BoxedUint {
    #[inline]
    fn ct_lt(&self, other: &Self) -> Choice {
        UintRef::lt(self.as_uint_ref(), other.as_uint_ref())
    }
}

impl CtNeg for BoxedUint {
    fn ct_neg(&self, choice: Choice) -> Self {
        let mut neg = self.clone();
        neg.conditional_wrapping_neg_assign(choice);
        neg
    }

    fn ct_neg_assign(&mut self, choice: Choice) {
        self.conditional_wrapping_neg_assign(choice);
    }
}

impl CtSelect for BoxedUint {
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        assert_eq!(self.bits_precision(), other.bits_precision());
        let mut limbs = vec![Limb::ZERO; self.nlimbs()].into_boxed_slice();

        for i in 0..self.nlimbs() {
            limbs[i] = self.limbs[i].ct_select(&other.limbs[i], choice);
        }

        Self { limbs }
    }

    #[inline]
    fn ct_swap(&mut self, other: &mut Self, choice: Choice) {
        assert_eq!(self.bits_precision(), other.bits_precision());

        for i in 0..self.nlimbs() {
            self.limbs[i].ct_swap(&mut other.limbs[i], choice);
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{Choice, CtAssign, CtSelect};

    /// Conditional selection, assignment and swapping preserve either complete input value.
    #[kani::proof]
    #[kani::unwind(5)]
    fn boxed_ct_select_assign_swap_correct() {
        let (mut a, mut b) = (any_boxed::<{ 2 * L64 }>(), any_boxed::<{ 2 * L64 }>());
        let (va, vb) = (value(&a), value(&b));
        let condition: bool = kani::any();
        let choice = if condition {
            Choice::TRUE
        } else {
            Choice::FALSE
        };
        let expected = if condition { vb } else { va };
        let selected = a.ct_select(&b, choice);
        assert_eq!(
            (value(&selected), selected.bits_precision()),
            (expected, 128)
        );
        a.ct_swap(&mut b, choice);
        assert_eq!(value(&a), expected);
        assert_eq!(value(&b), if condition { va } else { vb });
        let mut assigned = any_boxed::<{ 2 * L64 }>();
        let before = value(&assigned);
        assigned.ct_assign(&selected, choice);
        assert_eq!(value(&assigned), if condition { expected } else { before });
    }
}

#[cfg(feature = "subtle")]
impl subtle::ConditionallyNegatable for BoxedUint {
    #[inline]
    fn conditional_negate(&mut self, choice: subtle::Choice) {
        self.ct_neg_assign(choice.into());
    }
}

#[cfg(feature = "subtle")]
impl subtle::ConstantTimeEq for BoxedUint {
    #[inline]
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        CtEq::ct_eq(self, other).into()
    }
}

#[cfg(feature = "subtle")]
impl subtle::ConstantTimeGreater for BoxedUint {
    #[inline]
    fn ct_gt(&self, other: &Self) -> subtle::Choice {
        CtGt::ct_gt(self, other).into()
    }
}

#[cfg(feature = "subtle")]
impl subtle::ConstantTimeLess for BoxedUint {
    #[inline]
    fn ct_lt(&self, other: &Self) -> subtle::Choice {
        CtLt::ct_lt(self, other).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::{BoxedUint, Choice, CtEq, CtGt, CtLt, CtSelect};

    #[test]
    fn ct_eq() {
        let a = BoxedUint::zero();
        let b = BoxedUint::one();

        assert!(a.ct_eq(&a).to_bool());
        assert!(!a.ct_eq(&b).to_bool());
        assert!(!b.ct_eq(&a).to_bool());
        assert!(b.ct_eq(&b).to_bool());
    }

    #[test]
    fn ct_gt() {
        let a = BoxedUint::zero();
        let b = BoxedUint::one();
        let c = BoxedUint::max(64);

        assert!(b.ct_gt(&a).to_bool());
        assert!(c.ct_gt(&a).to_bool());
        assert!(c.ct_gt(&b).to_bool());

        assert!(!a.ct_gt(&a).to_bool());
        assert!(!b.ct_gt(&b).to_bool());
        assert!(!c.ct_gt(&c).to_bool());

        assert!(!a.ct_gt(&b).to_bool());
        assert!(!a.ct_gt(&c).to_bool());
        assert!(!b.ct_gt(&c).to_bool());
    }

    #[test]
    fn ct_lt() {
        let a = BoxedUint::zero();
        let b = BoxedUint::one();
        let c = BoxedUint::max(64);

        assert!(a.ct_lt(&b).to_bool());
        assert!(a.ct_lt(&c).to_bool());
        assert!(b.ct_lt(&c).to_bool());

        assert!(!a.ct_lt(&a).to_bool());
        assert!(!b.ct_lt(&b).to_bool());
        assert!(!c.ct_lt(&c).to_bool());

        assert!(!b.ct_lt(&a).to_bool());
        assert!(!c.ct_lt(&a).to_bool());
        assert!(!c.ct_lt(&b).to_bool());
    }

    #[test]
    fn ct_select() {
        let a = BoxedUint::zero_with_precision(128);
        let b = BoxedUint::max(128);

        assert_eq!(a, BoxedUint::ct_select(&a, &b, Choice::FALSE));
        assert_eq!(b, BoxedUint::ct_select(&a, &b, Choice::TRUE));
    }
}
