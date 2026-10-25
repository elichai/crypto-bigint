//! [`BoxedUint`] addition operations.

use crate::{
    Add, AddAssign, BoxedUint, CheckedAdd, Choice, CtOption, Limb, U64, U128, UintRef, Wrapping,
    WrappingAdd,
};
use core::cmp;

impl BoxedUint {
    /// Computes `self + rhs + carry`, returning the result along with the new carry.
    #[deprecated(since = "0.7.0", note = "please use `carrying_add` instead")]
    #[must_use]
    pub fn adc(&self, rhs: &Self, carry: Limb) -> (Self, Limb) {
        self.carrying_add(rhs, carry)
    }

    /// Computes `self + rhs + carry`, returning the result along with the new carry.
    ///
    /// The result is widened to the same width as the widest input.
    #[inline(always)]
    #[must_use]
    pub fn carrying_add(&self, rhs: impl AsRef<UintRef>, carry: Limb) -> (Self, Limb) {
        let rhs = rhs.as_ref();
        let precision = cmp::max(self.bits_precision(), rhs.bits_precision());
        let mut result = Self::zero_with_precision(precision);
        let carry =
            result
                .as_mut_uint_ref()
                .fold_limbs(self.as_uint_ref(), rhs, carry, Limb::carrying_add);
        (result, carry)
    }

    /// Computes `self + rhs + carry` in-place, returning the new carry.
    ///
    /// # Panics
    /// - if `rhs` has a larger precision than `self`.
    #[deprecated(since = "0.7.0", note = "please use `carrying_add_assign` instead")]
    pub fn adc_assign(&mut self, rhs: impl AsRef<[Limb]>, carry: Limb) -> Limb {
        self.carrying_add_assign(UintRef::new(rhs.as_ref()), carry)
    }

    /// Computes `self + rhs + carry` in-place, returning the new carry.
    ///
    /// # Panics
    /// - if `rhs` has a larger precision than `self`.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn carrying_add_assign(&mut self, rhs: impl AsRef<UintRef>, carry: Limb) -> Limb {
        let rhs = rhs.as_ref();
        assert!(rhs.nlimbs() <= self.nlimbs());

        self.as_mut_uint_ref()
            .fold_limbs_assign(rhs, carry, Limb::carrying_add)
    }

    /// Computes `self + rhs`, returning a result which is concatenated with the overflow limb which
    /// would be returned if `carrying_add` were called with the same operands.
    #[must_use]
    pub fn concatenating_add(&self, rhs: impl AsRef<UintRef>) -> Self {
        let rhs = rhs.as_ref();
        let bits = cmp::max(self.bits_precision(), rhs.bits_precision()) + 1;
        let mut result = Self::zero_with_precision(bits);
        let overflow = result.as_mut_uint_ref().fold_limbs(
            self.as_uint_ref(),
            rhs,
            Limb::ZERO,
            Limb::carrying_add,
        );
        // Overflow should always be zero here because we added a zero limb to the result
        debug_assert_eq!(overflow, Limb::ZERO);
        result
    }

    /// Computes `self + rhs`, returning a tuple of the sum along with a [`Choice`] which indicates
    /// whether an overflow occurred.
    ///
    /// If an overflow occurred, then the wrapped value is returned.
    #[must_use]
    pub fn overflowing_add(&self, rhs: impl AsRef<UintRef>) -> (Self, Choice) {
        let rhs = rhs.as_ref();
        let nlimbs = cmp::min(self.nlimbs(), rhs.nlimbs());
        let (rhs, rhs_over) = rhs.split_at(nlimbs);
        let (ret, carry) = self.carrying_add(rhs, Limb::ZERO);
        (ret, carry.is_nonzero().or(rhs_over.is_nonzero()))
    }

    /// Adds `rhs` to self, returning a [`Choice`] which indicates whether an overflow occurred.
    ///
    /// If an overflow occurred, then the wrapped value is returned.
    pub fn overflowing_add_assign(&mut self, rhs: impl AsRef<UintRef>) -> Choice {
        let rhs = rhs.as_ref();
        let nlimbs = cmp::min(self.nlimbs(), rhs.nlimbs());
        let (rhs, rhs_over) = rhs.split_at(nlimbs);
        let carry = self.carrying_add_assign(rhs, Limb::ZERO);
        carry.is_nonzero().or(rhs_over.is_nonzero())
    }

    /// Perform wrapping addition, discarding overflow.
    #[must_use]
    pub fn wrapping_add(&self, rhs: impl AsRef<UintRef>) -> Self {
        let rhs = rhs.as_ref();
        let nlimbs = cmp::min(self.nlimbs(), rhs.nlimbs());
        self.carrying_add(rhs.leading(nlimbs), Limb::ZERO).0
    }

    /// Perform wrapping addition of `rhs` to `self`, discarding overflow.
    pub fn wrapping_add_assign(&mut self, rhs: impl AsRef<UintRef>) {
        let rhs = rhs.as_ref();
        let nlimbs = cmp::min(self.nlimbs(), rhs.nlimbs());
        self.carrying_add_assign(rhs.leading(nlimbs), Limb::ZERO);
    }

    /// Perform a conditional in-place wrapping addition, returning the truthy value
    /// if an overflow has occurred.
    pub(crate) fn conditional_carrying_add_assign(&mut self, rhs: &Self, choice: Choice) -> Choice {
        self.as_mut_uint_ref()
            .conditional_add_assign(rhs.as_uint_ref(), Limb::ZERO, choice)
            .lsb_to_choice()
    }
}

impl<Rhs: AsRef<UintRef>> Add<Rhs> for BoxedUint {
    type Output = Self;

    fn add(self, rhs: Rhs) -> Self {
        Add::add(&self, rhs)
    }
}

impl<Rhs: AsRef<UintRef>> Add<Rhs> for &BoxedUint {
    type Output = BoxedUint;

    fn add(self, rhs: Rhs) -> BoxedUint {
        let (res, overflow) = self.overflowing_add(rhs);
        assert!(overflow.not().to_bool(), "attempted to add with overflow");
        res
    }
}

impl<Rhs: AsRef<UintRef>> AddAssign<Rhs> for BoxedUint {
    fn add_assign(&mut self, rhs: Rhs) {
        let overflow = self.overflowing_add_assign(rhs);
        assert!(overflow.not().to_bool(), "attempted to add with overflow");
    }
}

impl<Rhs: AsRef<UintRef>> AddAssign<Rhs> for Wrapping<BoxedUint> {
    fn add_assign(&mut self, other: Rhs) {
        self.0.wrapping_add_assign(other);
    }
}

impl<Rhs: AsRef<UintRef>> CheckedAdd<Rhs> for BoxedUint {
    fn checked_add(&self, rhs: &Rhs) -> CtOption<Self> {
        let (result, overflow) = self.overflowing_add(rhs);
        CtOption::new(result, overflow.not())
    }
}

impl WrappingAdd for BoxedUint {
    fn wrapping_add(&self, v: &Self) -> Self {
        self.wrapping_add(v)
    }
}

macro_rules! impl_add_primitive {
    ($($primitive:ty),+) => {
        $(
            impl Add<$primitive> for BoxedUint {
                type Output = <Self as Add<U64>>::Output;

                #[inline]
                fn add(self, rhs: $primitive) -> Self::Output {
                    self + U64::from(rhs)
                }
            }

            impl Add<$primitive> for &BoxedUint {
                type Output = <Self as Add<U64>>::Output;

                #[inline]
                fn add(self, rhs: $primitive) -> Self::Output {
                    self + U64::from(rhs)
                }
            }

            impl AddAssign<$primitive> for BoxedUint {
                fn add_assign(&mut self, rhs: $primitive) {
                    *self += U64::from(rhs);
                }
            }
        )+
    };
}

impl_add_primitive!(u8, u16, u32, u64);

impl Add<u128> for BoxedUint {
    type Output = BoxedUint;

    #[inline]
    fn add(self, rhs: u128) -> BoxedUint {
        self + U128::from(rhs)
    }
}

impl Add<u128> for &BoxedUint {
    type Output = BoxedUint;

    #[inline]
    fn add(self, rhs: u128) -> BoxedUint {
        self + U128::from(rhs)
    }
}

impl AddAssign<u128> for BoxedUint {
    fn add_assign(&mut self, rhs: u128) {
        *self += U128::from(rhs);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{BoxedUint, CheckedAdd, Choice, Limb, Word, Wrapping};

    fn assert_value(result: &BoxedUint, expected: u128, bits: u32) {
        assert_eq!(value(result), expected);
        assert_eq!(result.bits_precision(), bits);
    }

    fn concatenating<const N: usize, const M: usize>() {
        let (lhs, rhs) = (any_boxed::<N>(), any_boxed::<M>());
        let limbs = N.max(M);
        let bits = limbs as u32 * Word::BITS;
        let (sum, overflow) = value(&lhs).overflowing_add(value(&rhs));
        let carry = if bits == 64 {
            (sum >> 64) as Word
        } else {
            Word::from(overflow)
        };
        let result = lhs.concatenating_add(&rhs);
        assert_eq!(result.nlimbs(), limbs + 1);
        assert_eq!(result.bits_precision(), bits + Word::BITS);
        // The full sum can have 129 bits, so compare low limbs and the extra limb separately.
        for i in 0..limbs {
            assert_eq!(
                result.as_limbs()[i].0,
                (sum >> (i as u32 * Word::BITS)) as Word
            );
        }
        assert_eq!(result.as_limbs()[limbs].0, carry);
    }

    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_concatenating_add_64() {
        concatenating::<L64, L64>();
    }

    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_concatenating_add_128() {
        concatenating::<{ 2 * L64 }, { 2 * L64 }>();
    }

    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_concatenating_add_mixed() {
        concatenating::<L64, { 2 * L64 }>();
        concatenating::<{ 2 * L64 }, L64>();
    }

    fn conditional<const N: usize, const M: usize>() {
        let (mut lhs, rhs) = (any_boxed::<N>(), any_boxed::<M>());
        let enabled: bool = kani::any();
        let bits = N as u32 * Word::BITS;
        let mask = u128::MAX >> (u128::BITS - bits);
        let (sum, overflow) = if enabled {
            value(&lhs).overflowing_add(value(&rhs))
        } else {
            (value(&lhs), false)
        };
        let carry =
            lhs.conditional_carrying_add_assign(&rhs, Choice::from_u8_lsb(u8::from(enabled)));
        assert_value(&lhs, sum & mask, bits);
        assert_eq!(carry.to_bool(), overflow || sum > mask);
    }

    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_conditional_add_64() {
        conditional::<L64, L64>();
    }

    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_conditional_add_128() {
        conditional::<{ 2 * L64 }, { 2 * L64 }>();
    }

    /// The in-place API requires the RHS to fit the destination's storage width.
    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_conditional_add_shorter_rhs() {
        conditional::<{ 2 * L64 }, L64>();
    }

    fn carrying<const N: usize, const M: usize>() {
        let (mut lhs, rhs) = (any_boxed::<N>(), any_boxed::<M>());
        let carry: Word = kani::any();
        let bits = (N.max(M) as u32) * Limb::BITS;
        let (sum, overflow1) = value(&lhs).overflowing_add(value(&rhs));
        let (sum, overflow2) = sum.overflowing_add(u128::from(carry));
        let (expected, expected_carry) = if bits == 64 {
            (sum & u128::from(u64::MAX), (sum >> 64) as Word)
        } else {
            (sum, Word::from(overflow1) + Word::from(overflow2))
        };

        let (result, carry_out) = lhs.carrying_add(&rhs, Limb(carry));
        assert_value(&result, expected, bits);
        assert_eq!(carry_out.0, expected_carry);
        if N >= M {
            let carry_out = lhs.carrying_add_assign(&rhs, Limb(carry));
            assert_value(&lhs, expected, bits);
            assert_eq!(carry_out.0, expected_carry);
        }
    }

    /// Any incoming word is added in full; carry-out and precision match the native oracle.
    #[kani::proof]
    fn boxed_add_carrying_64() {
        carrying::<L64, L64>();
    }

    #[kani::proof]
    fn boxed_add_carrying_128() {
        carrying::<{ 2 * L64 }, { 2 * L64 }>();
    }

    /// Carrying addition zero-extends the shorter operand in either order.
    #[kani::proof]
    fn boxed_add_carrying_mixed() {
        carrying::<L64, { 2 * L64 }>();
        carrying::<{ 2 * L64 }, L64>();
    }

    fn forms<const N: usize, const M: usize, const GROUP: u8>() {
        let (lhs, rhs) = (any_boxed::<N>(), any_boxed::<M>());
        let bits = N as u32 * Limb::BITS;
        let mask = if bits == 64 {
            u128::from(u64::MAX)
        } else {
            u128::MAX
        };
        let (sum, native_overflow) = value(&lhs).overflowing_add(value(&rhs));
        let expected = sum & mask;
        let overflow = native_overflow || sum > mask;

        if GROUP == 0 {
            let (result, actual_overflow) = lhs.overflowing_add(&rhs);
            assert_value(&result, expected, bits);
            assert_eq!(actual_overflow.to_bool(), overflow);
            assert_value(&lhs.wrapping_add(&rhs), expected, bits);
            let checked = lhs.checked_add(&rhs);
            assert_eq!(checked.is_some().to_bool(), !overflow);
            if !overflow {
                assert_value(&checked.unwrap(), expected, bits);
            }
        }
        if GROUP == 1 {
            let mut assigned = lhs.clone();
            assert_eq!(assigned.overflowing_add_assign(&rhs).to_bool(), overflow);
            assert_value(&assigned, expected, bits);
            let mut wrapped = Wrapping(lhs.clone());
            wrapped += &rhs;
            assert_value(&wrapped.0, expected, bits);
            if !overflow {
                let mut assigned = lhs;
                assigned += &rhs;
                assert_value(&assigned, expected, bits);
            }
        } else if GROUP == 2 && !overflow {
            assert_value(&(&lhs + &rhs), expected, bits);
            assert_value(&(lhs + &rhs), expected, bits);
        }
    }

    // Separate API groups keep the number of live heap objects small for CBMC.
    macro_rules! prove_forms {
        ($plain:ident, $assign:ident, $operators:ident, $n:expr, $m:expr) => {
            #[kani::proof]
            fn $plain() {
                forms::<{ $n }, { $m }, 0>();
            }
            #[kani::proof]
            fn $assign() {
                forms::<{ $n }, { $m }, 1>();
            }
            #[kani::proof]
            fn $operators() {
                forms::<{ $n }, { $m }, 2>();
            }
        };
    }

    prove_forms!(
        boxed_add_forms_64,
        boxed_add_assign_64,
        boxed_add_operators_64,
        L64,
        L64
    );
    prove_forms!(
        boxed_add_forms_128,
        boxed_add_assign_128,
        boxed_add_operators_128,
        2 * L64,
        2 * L64
    );
    prove_forms!(
        boxed_add_forms_wider_rhs,
        boxed_add_assign_wider_rhs,
        boxed_add_operators_wider_rhs,
        L64,
        2 * L64
    );
    prove_forms!(
        boxed_add_forms_shorter_rhs,
        boxed_add_assign_shorter_rhs,
        boxed_add_operators_shorter_rhs,
        2 * L64,
        L64
    );
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{BoxedUint, CheckedAdd, Limb, UintRef, Wrapping};
    use crate::Resize;

    #[test]
    fn add_assign() {
        let mut h = BoxedUint::one().resize(1024);
        h += BoxedUint::one();
        assert_eq!(h, BoxedUint::from(2u8));
    }

    #[test]
    fn add_with_u32_rhs() {
        let a = BoxedUint::from(1u64);
        let b = a + u32::MAX;
        assert_eq!(b, BoxedUint::from(0x100000000u64));
    }

    #[test]
    fn carrying_add_no_carry() {
        let (res, carry) = BoxedUint::zero().carrying_add(BoxedUint::one(), Limb::ZERO);
        assert_eq!(res, BoxedUint::one());
        assert_eq!(carry, Limb::ZERO);
    }

    #[test]
    fn carrying_add_with_carry() {
        let (res, carry) = BoxedUint::max(Limb::BITS).carrying_add(BoxedUint::one(), Limb::ZERO);
        assert_eq!(res, BoxedUint::zero());
        assert_eq!(carry, Limb::ONE);
    }

    #[test]
    fn checked_add_ok() {
        let result = BoxedUint::zero().checked_add(&BoxedUint::one());
        assert_eq!(result.unwrap(), BoxedUint::one());
    }

    #[test]
    fn checked_add_overflow() {
        let result = BoxedUint::max(Limb::BITS).checked_add(&BoxedUint::one());
        assert!(!bool::from(result.is_some()));
    }

    #[test]
    fn concatenating_add() {
        let result = BoxedUint::max(Limb::BITS).concatenating_add(BoxedUint::one());
        assert_eq!(result.as_limbs(), &[Limb::ZERO, Limb::ONE]);
    }

    #[test]
    fn overflowing_add() {
        let one = BoxedUint::one();
        let (ret, overflow) = one.overflowing_add(&one);
        assert_eq!(ret, &one + &one);
        assert!(!overflow.to_bool());

        let (ret, overflow) = BoxedUint::max(2 * Limb::BITS).overflowing_add(&one);
        assert!(ret.is_zero().to_bool());
        assert!(overflow.to_bool());
    }

    #[test]
    fn wrapping_add() {
        let ret = BoxedUint::one().wrapping_add(Limb::ONE);
        assert_eq!(ret, BoxedUint::from(2u8));

        let mut ret = Wrapping(BoxedUint::max(2 * Limb::BITS));
        ret += Wrapping(Limb::ONE);
        assert!(ret.0.is_zero_vartime());
    }

    #[test]
    fn add_uintref() {
        let a = BoxedUint::from(1234567890u64);
        let b = UintRef::new(&[Limb(456), Limb(0)]);
        assert_eq!(
            a.carrying_add(b, Limb::ZERO).0,
            BoxedUint::from(1234568346u64)
        );
    }
}
