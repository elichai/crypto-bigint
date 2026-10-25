//! Constant-time support: impls of `Ct*` traits and constant-time `const fn` operations.

use crate::{Choice, CtAssign, CtEq, modular::FixedMontyForm};
use ctutils::{CtAssignSlice, CtEqSlice, CtSelectUsingCtAssign};

#[cfg(feature = "subtle")]
use crate::CtSelect;

impl<const LIMBS: usize> CtAssign for FixedMontyForm<LIMBS> {
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        self.montgomery_form
            .ct_assign(&other.montgomery_form, choice);
        self.params.ct_assign(&other.params, choice);
    }
}
impl<const LIMBS: usize> CtAssignSlice for FixedMontyForm<LIMBS> {}
impl<const LIMBS: usize> CtSelectUsingCtAssign for FixedMontyForm<LIMBS> {}

impl<const LIMBS: usize> CtEq for FixedMontyForm<LIMBS> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.montgomery_form.ct_eq(&other.montgomery_form) & self.params.ct_eq(&other.params)
    }
}
impl<const LIMBS: usize> CtEqSlice for FixedMontyForm<LIMBS> {}

#[cfg(kani)]
mod kani_proofs {
    use crate::{
        Choice, CtAssign, CtEq, CtSelect, Odd, U64,
        modular::{FixedMontyForm, FixedMontyParams},
    };

    const SMALL: FixedMontyParams<{ U64::LIMBS }> =
        FixedMontyParams::new_vartime(Odd::<U64>::from_be_hex("0000000000000011"));
    const LARGE: FixedMontyParams<{ U64::LIMBS }> =
        FixedMontyParams::new_vartime(Odd::<U64>::from_be_hex("0000000000000101"));
    const MAX: FixedMontyParams<{ U64::LIMBS }> =
        FixedMontyParams::new_vartime(Odd::<U64>::from_be_hex("ffffffffffffffff"));

    /// All raw 64-bit representations, both choices, and two independently chosen valid params.
    #[kani::proof]
    #[kani::unwind(3)]
    fn fixed_monty_ct_operations_preserve_params() {
        let (x, y, ax, by, choose): (u64, u64, bool, bool, bool) = kani::any();
        let a = FixedMontyForm::from_montgomery(U64::from_u64(x), if ax { &SMALL } else { &LARGE });
        let b = FixedMontyForm::from_montgomery(U64::from_u64(y), if by { &SMALL } else { &LARGE });
        assert_eq!(a.ct_eq(&b).to_bool(), x == y && ax == by);
        let expected = if choose { b } else { a };
        let choice = Choice::from(u8::from(choose));
        assert_eq!(a.ct_select(&b, choice), expected);
        let mut assigned = a;
        assigned.ct_assign(&b, choice);
        assert_eq!(assigned, expected);
    }

    /// Every pair of reduced u64 representations at m=2^64-1, including carry boundaries.
    #[kani::proof]
    #[kani::unwind(3)]
    fn fixed_monty_additive_u64_max_modulus() {
        let (x, y): (u64, u64) = kani::any();
        kani::assume(x < u64::MAX && y < u64::MAX);
        let a = FixedMontyForm::from_montgomery(U64::from_u64(x), &MAX);
        let b = FixedMontyForm::from_montgomery(U64::from_u64(y), &MAX);
        // Both exact native sums are below 2m, so one subtraction is the remainder.
        let reduce = |sum: u128| {
            let m = u128::from(u64::MAX);
            (if sum >= m { sum - m } else { sum }) as u64
        };
        let expected = reduce(u128::from(x) + u128::from(y));
        assert_eq!(u64::from(a.add(&b).to_montgomery()), expected);
        assert_eq!(a.add(&b).params(), &MAX);
        let doubled = reduce(u128::from(x) * 2);
        assert_eq!(u64::from(a.double().to_montgomery()), doubled);
        assert_eq!(a.double().params(), &MAX);
        let half = if x & 1 == 0 {
            x / 2
        } else {
            x / 2 + (1u64 << 63)
        };
        assert_eq!(u64::from(a.div_by_2().to_montgomery()), half);
        assert_eq!(a.div_by_2().params(), &MAX);
    }

    /// Every reduced residue for m=257, stored in U64; R=1 mod m on either Word width.
    #[kani::proof]
    #[kani::unwind(3)]
    fn fixed_monty_retrieve_u64_257_modulus() {
        // A u16 fixes the upper zero bits before SSA while covering exactly the same 257 residues.
        let x: u16 = kani::any();
        kani::assume(x < 257);
        let value = FixedMontyForm::from_montgomery(U64::from_u64(u64::from(x)), &LARGE);
        assert_eq!(u64::from(value.retrieve()), u64::from(x));
    }

    #[cfg(test)]
    #[kani::proof]
    #[kani::unwind(3)]
    fn regression_fixed_monty_retrieve_raw_modulus() {
        let raw = *SMALL.modulus().as_ref();
        let value = if kani::any() {
            FixedMontyForm::from_montgomery(raw, &SMALL)
        } else {
            let mut value = FixedMontyForm::zero(&SMALL);
            *value.as_montgomery_mut() = raw;
            value
        };
        assert_eq!(value.retrieve(), U64::ZERO);
    }
}

#[cfg(feature = "subtle")]
impl<const LIMBS: usize> subtle::ConstantTimeEq for FixedMontyForm<LIMBS> {
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        CtEq::ct_eq(self, other).into()
    }
}

#[cfg(feature = "subtle")]
impl<const LIMBS: usize> subtle::ConditionallySelectable for FixedMontyForm<LIMBS> {
    fn conditional_select(a: &Self, b: &Self, choice: subtle::Choice) -> Self {
        a.ct_select(b, choice.into())
    }
}
