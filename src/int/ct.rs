//! Constant-time support: impls of `Ct*` traits and constant-time `const fn` operations.

use crate::{Choice, CtAssign, CtEq, CtGt, CtLt, CtSelect, Int, Uint};
use ctutils::{CtAssignSlice, CtEqSlice};

impl<const LIMBS: usize> Int<LIMBS> {
    /// Return `b` if `c` is truthy, otherwise return `a`.
    #[inline]
    pub(crate) const fn select(a: &Self, b: &Self, c: Choice) -> Self {
        Self(Uint::select(&a.0, &b.0, c))
    }
}

impl<const LIMBS: usize> CtAssign for Int<LIMBS> {
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        self.0.ct_assign(&other.0, choice);
    }
}
impl<const LIMBS: usize> CtAssignSlice for Int<LIMBS> {}

impl<const LIMBS: usize> CtEq for Int<LIMBS> {
    #[inline]
    fn ct_eq(&self, other: &Self) -> Choice {
        CtEq::ct_eq(&self.0, &other.0)
    }
}
impl<const LIMBS: usize> CtEqSlice for Int<LIMBS> {}

impl<const LIMBS: usize> CtGt for Int<LIMBS> {
    #[inline]
    fn ct_gt(&self, other: &Self) -> Choice {
        Int::gt(self, other)
    }
}

impl<const LIMBS: usize> CtLt for Int<LIMBS> {
    #[inline]
    fn ct_lt(&self, other: &Self) -> Choice {
        Int::lt(self, other)
    }
}

impl<const LIMBS: usize> CtSelect for Int<LIMBS> {
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        Self(self.0.ct_select(&other.0, choice))
    }
}

#[cfg(feature = "subtle")]
impl<const LIMBS: usize> subtle::ConditionallySelectable for Int<LIMBS> {
    #[inline]
    fn conditional_select(a: &Self, b: &Self, choice: subtle::Choice) -> Self {
        a.ct_select(b, choice.into())
    }
}

#[cfg(feature = "subtle")]
impl<const LIMBS: usize> subtle::ConstantTimeEq for Int<LIMBS> {
    #[inline]
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        CtEq::ct_eq(self, other).into()
    }
}

#[cfg(feature = "subtle")]
impl<const LIMBS: usize> subtle::ConstantTimeGreater for Int<LIMBS> {
    #[inline]
    fn ct_gt(&self, other: &Self) -> subtle::Choice {
        CtGt::ct_gt(self, other).into()
    }
}

#[cfg(feature = "subtle")]
impl<const LIMBS: usize> subtle::ConstantTimeLess for Int<LIMBS> {
    #[inline]
    fn ct_lt(&self, other: &Self) -> subtle::Choice {
        Int::lt(self, other).into()
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{Choice, CtAssign, CtEq, CtGt, CtLt, CtSelect, I128};

    #[kani::proof]
    #[kani::unwind(5)]
    fn int_ct_select_assign_native() {
        let (a, b, flag): (i128, i128, bool) = kani::any();
        let (ia, ib) = (I128::from_i128(a), I128::from_i128(b));
        let choice = Choice::from_u8_lsb(flag as u8);
        let expected = if flag { b } else { a };
        assert_eq!(i128::from(I128::select(&ia, &ib, choice)), expected);
        assert_eq!(i128::from(ia.ct_select(&ib, choice)), expected);
        let mut assigned = ia;
        assigned.ct_assign(&ib, choice);
        assert_eq!(i128::from(assigned), expected);
    }

    #[kani::proof]
    #[kani::unwind(5)]
    fn int_ct_comparisons_native() {
        let (a, b): (i128, i128) = kani::any();
        let (ia, ib) = (I128::from_i128(a), I128::from_i128(b));
        assert_eq!(ia.ct_eq(&ib).to_bool(), a == b);
        assert_eq!(ia.ct_lt(&ib).to_bool(), a < b);
        assert_eq!(ia.ct_gt(&ib).to_bool(), a > b);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Choice, CtGt, CtLt, CtSelect, I128};

    #[test]
    fn ct_gt() {
        let a = I128::MIN;
        let b = I128::ZERO;
        let c = I128::MAX;

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
        let a = I128::ZERO;
        let b = I128::ONE;
        let c = I128::MAX;

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
        let a = I128::from_be_hex("00002222444466668888AAAACCCCEEEE");
        let b = I128::from_be_hex("11113333555577779999BBBBDDDDFFFF");

        let select_0 = I128::ct_select(&a, &b, Choice::FALSE);
        assert_eq!(a, select_0);

        let select_1 = I128::ct_select(&a, &b, Choice::TRUE);
        assert_eq!(b, select_1);
    }
}
