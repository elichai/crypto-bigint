//! Constant-time support: impls of `Ct*` traits.

use super::{BoxedMontyForm, BoxedMontyParams};
use crate::{Choice, CtAssign, CtEq, CtSelect};
use ctutils::{CtAssignSlice, CtEqSlice, CtSelectUsingCtAssign};

impl CtAssign for BoxedMontyForm {
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        self.montgomery_form
            .ct_assign(&other.montgomery_form, choice);
        self.params.ct_assign(&other.params, choice);
    }
}
impl CtAssignSlice for BoxedMontyForm {}

impl CtEq for BoxedMontyForm {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.montgomery_form.ct_eq(&other.montgomery_form) & self.params.ct_eq(&other.params)
    }
}
impl CtEqSlice for BoxedMontyForm {}

impl CtSelectUsingCtAssign for BoxedMontyForm {}

impl CtAssign for BoxedMontyParams {
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        *self = self.ct_select(other, choice);
    }
}

impl CtEq for BoxedMontyParams {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.modulus().ct_eq(other.modulus())
            & self.one().ct_eq(other.one())
            & self.r2().ct_eq(other.r2())
            & self.mod_inv().ct_eq(&other.mod_inv())
            & self.mod_leading_zeros().ct_eq(&other.mod_leading_zeros())
    }
}
impl CtEqSlice for BoxedMontyParams {}

impl CtSelect for BoxedMontyParams {
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        self.as_ref().ct_select(other.as_ref(), choice).into()
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::BoxedMontyForm;
    #[cfg(test)]
    use super::BoxedMontyParams;
    use crate::uint::boxed::kani_util::value;
    use crate::{BoxedUint, Odd, U64, modular::FixedMontyParams};

    /// Every reduced residue for m=257, at 64-bit precision; R=1 mod m.
    #[kani::proof]
    #[kani::unwind(3)]
    fn boxed_monty_retrieve_u64_257_modulus() {
        // A u16 fixes the upper zero bits before SSA while covering exactly the same 257 residues.
        let x: u16 = kani::any();
        kani::assume(x < 257);
        const FIXED: FixedMontyParams<{ U64::LIMBS }> =
            FixedMontyParams::new_vartime(Odd::<U64>::from_be_hex("0000000000000101"));
        let params = FIXED.into();
        let form = BoxedMontyForm::from_montgomery(BoxedUint::from(u64::from(x)), &params);
        let retrieved = form.retrieve();
        assert_eq!(value(&retrieved), u128::from(x));
        assert_eq!(retrieved.bits_precision(), 64);
    }

    #[cfg(test)]
    #[kani::proof]
    #[kani::unwind(3)]
    fn regression_boxed_monty_retrieve_raw_modulus() {
        const FIXED: FixedMontyParams<{ U64::LIMBS }> =
            FixedMontyParams::new_vartime(Odd::<U64>::from_be_hex("0000000000000011"));
        let params: BoxedMontyParams = FIXED.into();
        let raw = params.modulus().as_ref().clone();
        let form = BoxedMontyForm::from_montgomery(raw, &params);
        assert_eq!(value(&form.retrieve()), 0);
    }
}
