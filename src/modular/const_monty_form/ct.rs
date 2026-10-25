//! Constant-time support: impls of `Ct*` traits and constant-time `const fn` operations.

use super::{ConstMontyForm, ConstMontyParams};
use crate::{Choice, CtAssign, CtEq};
use ctutils::{CtAssignSlice, CtEqSlice, CtGt, CtLt, CtSelectUsingCtAssign};

#[cfg(feature = "subtle")]
use crate::CtSelect;

impl<MOD, const LIMBS: usize> CtAssign for ConstMontyForm<MOD, LIMBS>
where
    MOD: ConstMontyParams<LIMBS>,
{
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        self.montgomery_form
            .ct_assign(&other.montgomery_form, choice);
    }
}
impl<MOD, const LIMBS: usize> CtAssignSlice for ConstMontyForm<MOD, LIMBS> where
    MOD: ConstMontyParams<LIMBS>
{
}

impl<MOD, const LIMBS: usize> CtEq for ConstMontyForm<MOD, LIMBS>
where
    MOD: ConstMontyParams<LIMBS>,
{
    fn ct_eq(&self, other: &Self) -> Choice {
        CtEq::ct_eq(&self.montgomery_form, &other.montgomery_form)
    }
}
impl<MOD, const LIMBS: usize> CtEqSlice for ConstMontyForm<MOD, LIMBS> where
    MOD: ConstMontyParams<LIMBS>
{
}

impl<MOD, const LIMBS: usize> CtGt for ConstMontyForm<MOD, LIMBS>
where
    MOD: ConstMontyParams<LIMBS>,
{
    fn ct_gt(&self, other: &Self) -> Choice {
        self.retrieve().ct_gt(&other.retrieve())
    }
}

impl<MOD, const LIMBS: usize> CtLt for ConstMontyForm<MOD, LIMBS>
where
    MOD: ConstMontyParams<LIMBS>,
{
    fn ct_lt(&self, other: &Self) -> Choice {
        self.retrieve().ct_lt(&other.retrieve())
    }
}

impl<MOD, const LIMBS: usize> CtSelectUsingCtAssign for ConstMontyForm<MOD, LIMBS> where
    MOD: ConstMontyParams<LIMBS>
{
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{Choice, CtAssign, CtEq, CtSelect, U64, modular::ConstMontyForm};
    use ctutils::{CtGt, CtLt};

    crate::const_monty_params!(Modulus257, U64, "0000000000000101");
    type Form = ConstMontyForm<Modulus257, { U64::LIMBS }>;

    /// Every reduced residue pair for m=257, stored in U64; R=1 mod m.
    #[kani::proof]
    #[kani::unwind(3)]
    fn const_monty_ct_comparison_retrieve_u64_257_modulus() {
        // u16 inputs fix upper zero bits before SSA without changing the residue domains.
        let (x, y, choose): (u16, u16, bool) = kani::any();
        kani::assume(x < 257 && y < 257);
        let a = Form::from_montgomery(U64::from_u64(u64::from(x)));
        let b = Form::from_montgomery(U64::from_u64(u64::from(y)));
        assert_eq!(u64::from(a.retrieve()), u64::from(x));
        assert_eq!(a.ct_eq(&b).to_bool(), x == y);
        assert_eq!(a.ct_lt(&b).to_bool(), x < y);
        assert_eq!(a.ct_gt(&b).to_bool(), x > y);
        let expected = U64::from_u64(u64::from(if choose { y } else { x }));
        let choice = Choice::from(u8::from(choose));
        assert_eq!(a.ct_select(&b, choice).to_montgomery(), expected);
        let mut assigned = a;
        assigned.ct_assign(&b, choice);
        assert_eq!(assigned.to_montgomery(), expected);
    }

    #[cfg(test)]
    #[kani::proof]
    #[kani::unwind(3)]
    fn regression_const_monty_retrieve_raw_modulus() {
        let value = if kani::any() {
            Form::from_montgomery(U64::from_u64(257))
        } else {
            let mut value = Form::ZERO;
            *value.as_montgomery_mut() = U64::from_u64(257);
            value
        };
        assert_eq!(value.retrieve(), U64::ZERO);
    }
}

#[cfg(feature = "subtle")]
impl<MOD, const LIMBS: usize> subtle::ConstantTimeEq for ConstMontyForm<MOD, LIMBS>
where
    MOD: ConstMontyParams<LIMBS>,
{
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        CtEq::ct_eq(&self.montgomery_form, &other.montgomery_form).into()
    }
}

#[cfg(feature = "subtle")]
impl<MOD, const LIMBS: usize> subtle::ConditionallySelectable for ConstMontyForm<MOD, LIMBS>
where
    MOD: ConstMontyParams<LIMBS> + Copy,
{
    fn conditional_select(a: &Self, b: &Self, choice: subtle::Choice) -> Self {
        CtSelect::ct_select(a, b, choice.into())
    }
}

#[cfg(feature = "subtle")]
impl<MOD, const LIMBS: usize> subtle::ConstantTimeGreater for ConstMontyForm<MOD, LIMBS>
where
    MOD: ConstMontyParams<LIMBS>,
{
    fn ct_gt(&self, other: &Self) -> subtle::Choice {
        CtGt::ct_gt(self, other).into()
    }
}

#[cfg(feature = "subtle")]
impl<MOD, const LIMBS: usize> subtle::ConstantTimeLess for ConstMontyForm<MOD, LIMBS>
where
    MOD: ConstMontyParams<LIMBS>,
{
    fn ct_lt(&self, other: &Self) -> subtle::Choice {
        CtLt::ct_lt(self, other).into()
    }
}
