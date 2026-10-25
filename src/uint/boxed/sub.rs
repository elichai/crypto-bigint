//! [`BoxedUint`] subtraction operations.

use crate::{
    BoxedUint, CheckedSub, Choice, CtOption, Limb, Sub, SubAssign, U64, U128, UintRef, Wrapping,
    WrappingSub,
};
use core::cmp;

impl BoxedUint {
    /// Computes `self - (rhs + borrow)`, returning the result along with the new borrow.
    #[deprecated(since = "0.7.0", note = "please use `borrowing_sub` instead")]
    #[must_use]
    pub fn sbb(&self, rhs: &Self, borrow: Limb) -> (Self, Limb) {
        self.borrowing_sub(rhs, borrow)
    }

    /// Computes `self - (rhs + borrow)`, returning the result along with the new borrow.
    ///
    /// The result is widened to the same width as the widest input.
    #[inline(always)]
    #[must_use]
    pub fn borrowing_sub(&self, rhs: impl AsRef<UintRef>, borrow: Limb) -> (Self, Limb) {
        let rhs = rhs.as_ref();
        let precision = cmp::max(self.bits_precision(), rhs.bits_precision());
        let mut result = Self::zero_with_precision(precision);
        let borrow = result.as_mut_uint_ref().fold_limbs(
            self.as_uint_ref(),
            rhs,
            borrow,
            Limb::borrowing_sub,
        );
        (result, borrow)
    }

    /// Computes `a - (b + borrow)` in-place, returning the new borrow.
    ///
    /// # Panics
    /// - if `rhs` has a larger precision than `self`.
    #[deprecated(since = "0.7.0", note = "please use `borrowing_sub_assign` instead")]
    pub fn sbb_assign(&mut self, rhs: impl AsRef<[Limb]>, borrow: Limb) -> Limb {
        self.borrowing_sub_assign(UintRef::new(rhs.as_ref()), borrow)
    }

    /// Computes `a - (b + borrow)` in-place, returning the new borrow.
    ///
    /// # Panics
    /// - if `rhs` has a larger precision than `self`.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn borrowing_sub_assign(&mut self, rhs: impl AsRef<UintRef>, borrow: Limb) -> Limb {
        let rhs = rhs.as_ref();
        assert!(rhs.nlimbs() <= self.nlimbs());

        self.as_mut_uint_ref()
            .fold_limbs_assign(rhs, borrow, Limb::borrowing_sub)
    }

    /// Computes `self - rhs`, returning a tuple of the difference along with a [`Choice`] which
    /// indicates whether an underflow occurred.
    ///
    /// If an underflow occurred, then the wrapped value is returned.
    #[must_use]
    pub fn underflowing_sub(&self, rhs: impl AsRef<UintRef>) -> (Self, Choice) {
        let rhs = rhs.as_ref();
        let nlimbs = cmp::min(self.nlimbs(), rhs.nlimbs());
        let (rhs, rhs_over) = rhs.split_at(nlimbs);
        let (ret, borrow) = self.borrowing_sub(rhs, Limb::ZERO);
        (ret, borrow.is_nonzero().or(rhs_over.is_nonzero()))
    }

    /// Subtracts `rhs` from self, returning a [`Choice`] which indicates whether an underflow occurred.
    ///
    /// If an underflow occurred, then the wrapped value is returned.
    pub fn underflowing_sub_assign(&mut self, rhs: impl AsRef<UintRef>) -> Choice {
        let rhs = rhs.as_ref();
        let nlimbs = cmp::min(self.nlimbs(), rhs.nlimbs());
        let (rhs, rhs_over) = rhs.split_at(nlimbs);
        let borrow = self.borrowing_sub_assign(rhs, Limb::ZERO);
        borrow.is_nonzero().or(rhs_over.is_nonzero())
    }

    /// Perform wrapping subtraction, discarding underflow.
    #[must_use]
    pub fn wrapping_sub(&self, rhs: impl AsRef<UintRef>) -> Self {
        let rhs = rhs.as_ref();
        let nlimbs = cmp::min(self.nlimbs(), rhs.nlimbs());
        self.borrowing_sub(rhs.leading(nlimbs), Limb::ZERO).0
    }

    /// Perform wrapping subtraction of `rhs` from `self`, discarding underflow.
    pub fn wrapping_sub_assign(&mut self, rhs: impl AsRef<UintRef>) {
        let rhs = rhs.as_ref();
        let nlimbs = cmp::min(self.nlimbs(), rhs.nlimbs());
        self.borrowing_sub_assign(rhs.leading(nlimbs), Limb::ZERO);
    }
}

impl<Rhs: AsRef<UintRef>> CheckedSub<Rhs> for BoxedUint {
    fn checked_sub(&self, rhs: &Rhs) -> CtOption<Self> {
        let (result, underflow) = self.underflowing_sub(rhs);
        CtOption::new(result, underflow.not())
    }
}

impl<Rhs: AsRef<UintRef>> Sub<Rhs> for BoxedUint {
    type Output = Self;

    fn sub(self, rhs: Rhs) -> Self {
        Sub::sub(&self, rhs)
    }
}

impl<Rhs: AsRef<UintRef>> Sub<Rhs> for &BoxedUint {
    type Output = BoxedUint;

    fn sub(self, rhs: Rhs) -> BoxedUint {
        let (res, underflow) = self.underflowing_sub(rhs);
        assert!(
            underflow.not().to_bool(),
            "attempted to subtract with underflow"
        );
        res
    }
}

impl<Rhs: AsRef<UintRef>> SubAssign<Rhs> for BoxedUint {
    fn sub_assign(&mut self, rhs: Rhs) {
        let underflow = self.underflowing_sub_assign(rhs);
        assert!(
            underflow.not().to_bool(),
            "attempted to subtract with underflow"
        );
    }
}

impl<Rhs: AsRef<UintRef>> SubAssign<Rhs> for Wrapping<BoxedUint> {
    fn sub_assign(&mut self, other: Rhs) {
        self.0.wrapping_sub_assign(other);
    }
}

impl WrappingSub for BoxedUint {
    fn wrapping_sub(&self, v: &Self) -> Self {
        self.wrapping_sub(v)
    }
}

macro_rules! impl_sub_primitive {
    ($($primitive:ty),+) => {
        $(
            impl Sub<$primitive> for BoxedUint {
                type Output = BoxedUint;

                #[inline]
                fn sub(self, rhs: $primitive) -> BoxedUint {
                     self - U64::from(rhs)
                }
            }

            impl Sub<$primitive> for &BoxedUint {
                type Output = BoxedUint;

                #[inline]
                fn sub(self, rhs: $primitive) -> BoxedUint {
                     self - U64::from(rhs)
                }
            }

            impl SubAssign<$primitive> for BoxedUint {
                fn sub_assign(&mut self, rhs: $primitive) {
                    *self -= U64::from(rhs);
                }
            }
        )+
    };
}

impl_sub_primitive!(u8, u16, u32, u64);

impl Sub<u128> for BoxedUint {
    type Output = BoxedUint;

    #[inline]
    fn sub(self, rhs: u128) -> BoxedUint {
        self - U128::from(rhs)
    }
}

impl Sub<u128> for &BoxedUint {
    type Output = BoxedUint;

    #[inline]
    fn sub(self, rhs: u128) -> BoxedUint {
        self - U128::from(rhs)
    }
}

impl SubAssign<u128> for BoxedUint {
    fn sub_assign(&mut self, rhs: u128) {
        *self -= U128::from(rhs);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{BoxedUint, CheckedSub, Limb, Word, Wrapping};

    fn assert_value(result: &BoxedUint, expected: u128, bits: u32) {
        assert_eq!(value(result), expected);
        assert_eq!(result.bits_precision(), bits);
    }

    fn borrowing<const N: usize, const M: usize>() {
        let (mut lhs, rhs) = (any_boxed::<N>(), any_boxed::<M>());
        let borrow: Word = kani::any();
        let bits = N.max(M) as u32 * Limb::BITS;
        // A borrow word contributes its MSB; the output is a full-word mask.
        let incoming = u128::from(borrow >> (Word::BITS - 1));
        let (difference, underflow1) = value(&lhs).overflowing_sub(value(&rhs));
        let (difference, underflow2) = difference.overflowing_sub(incoming);
        let expected = if bits == 64 {
            difference & u128::from(u64::MAX)
        } else {
            difference
        };
        let expected_borrow = Word::from(underflow1 || underflow2).wrapping_neg();

        let (result, borrow_out) = lhs.borrowing_sub(&rhs, Limb(borrow));
        assert_value(&result, expected, bits);
        assert_eq!(borrow_out.0, expected_borrow);
        if N >= M {
            let borrow_out = lhs.borrowing_sub_assign(&rhs, Limb(borrow));
            assert_value(&lhs, expected, bits);
            assert_eq!(borrow_out.0, expected_borrow);
        }
    }

    #[kani::proof]
    fn boxed_sub_borrowing_64() {
        borrowing::<L64, L64>();
    }

    #[kani::proof]
    fn boxed_sub_borrowing_128() {
        borrowing::<{ 2 * L64 }, { 2 * L64 }>();
    }

    /// Borrowing subtraction zero-extends the shorter operand in either order.
    #[kani::proof]
    fn boxed_sub_borrowing_mixed() {
        borrowing::<L64, { 2 * L64 }>();
        borrowing::<{ 2 * L64 }, L64>();
    }

    fn forms<const N: usize, const M: usize, const GROUP: u8>() {
        let (lhs, rhs) = (any_boxed::<N>(), any_boxed::<M>());
        let bits = N as u32 * Limb::BITS;
        let mask = if bits == 64 {
            u128::from(u64::MAX)
        } else {
            u128::MAX
        };
        let (difference, underflow) = value(&lhs).overflowing_sub(value(&rhs));
        let expected = difference & mask;

        if GROUP == 0 {
            let (result, actual_underflow) = lhs.underflowing_sub(&rhs);
            assert_value(&result, expected, bits);
            assert_eq!(actual_underflow.to_bool(), underflow);
            assert_value(&lhs.wrapping_sub(&rhs), expected, bits);
            let checked = lhs.checked_sub(&rhs);
            assert_eq!(checked.is_some().to_bool(), !underflow);
            if !underflow {
                assert_value(&checked.unwrap(), expected, bits);
            }
        }
        if GROUP == 1 {
            let mut assigned = lhs.clone();
            assert_eq!(assigned.underflowing_sub_assign(&rhs).to_bool(), underflow);
            assert_value(&assigned, expected, bits);
            let mut wrapped = Wrapping(lhs.clone());
            wrapped -= &rhs;
            assert_value(&wrapped.0, expected, bits);
            if !underflow {
                let mut assigned = lhs;
                assigned -= &rhs;
                assert_value(&assigned, expected, bits);
            }
        } else if GROUP == 2 && !underflow {
            assert_value(&(&lhs - &rhs), expected, bits);
            assert_value(&(lhs - &rhs), expected, bits);
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
        boxed_sub_forms_64,
        boxed_sub_assign_64,
        boxed_sub_operators_64,
        L64,
        L64
    );
    prove_forms!(
        boxed_sub_forms_128,
        boxed_sub_assign_128,
        boxed_sub_operators_128,
        2 * L64,
        2 * L64
    );
    prove_forms!(
        boxed_sub_forms_wider_rhs,
        boxed_sub_assign_wider_rhs,
        boxed_sub_operators_wider_rhs,
        L64,
        2 * L64
    );
    prove_forms!(
        boxed_sub_forms_shorter_rhs,
        boxed_sub_assign_shorter_rhs,
        boxed_sub_operators_shorter_rhs,
        2 * L64,
        L64
    );
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{BoxedUint, CheckedSub, Limb, UintRef, Wrapping};
    use crate::Resize;

    #[test]
    fn borrowing_sub_no_borrow() {
        let (res, carry) = BoxedUint::one().borrowing_sub(BoxedUint::one(), Limb::ZERO);
        assert_eq!(res, BoxedUint::zero());
        assert_eq!(carry, Limb::ZERO);

        let (res, carry) = BoxedUint::one().borrowing_sub(BoxedUint::zero(), Limb::ZERO);
        assert_eq!(res, BoxedUint::one());
        assert_eq!(carry, Limb::ZERO);
    }

    #[test]
    fn borrowing_sub_with_borrow() {
        let (res, borrow) = BoxedUint::zero().borrowing_sub(BoxedUint::one(), Limb::ZERO);
        assert_eq!(res, BoxedUint::max(Limb::BITS));
        assert_eq!(borrow, Limb::MAX);
    }

    #[test]
    fn sub_with_u32_rhs() {
        let a = BoxedUint::from(0x100000000u64);
        let b = a - u32::MAX;
        assert_eq!(b, BoxedUint::one());
    }

    #[test]
    fn checked_sub_ok() {
        let result = BoxedUint::one().checked_sub(&BoxedUint::one());
        assert_eq!(result.unwrap(), BoxedUint::zero());
    }

    #[test]
    fn checked_sub_overflow() {
        let result = BoxedUint::zero().checked_sub(&BoxedUint::one());
        assert!(!bool::from(result.is_some()));
    }

    #[test]
    fn sub_assign() {
        let mut h = BoxedUint::one().resize(1024);
        h -= BoxedUint::one();
    }

    #[test]
    fn wrapping_sub() {
        let ret = BoxedUint::one().wrapping_sub(Limb::ONE);
        assert!(ret.is_zero_vartime());

        let mut ret = Wrapping(BoxedUint::zero_with_precision(2 * Limb::BITS));
        ret -= Wrapping(Limb::ONE);
        assert_eq!(ret.0, BoxedUint::max(2 * Limb::BITS));
    }

    #[test]
    fn sub_uintref() {
        let a = BoxedUint::from(1234567890u64);
        let b = UintRef::new(&[Limb(456), Limb(0)]);
        assert_eq!(
            a.borrowing_sub(b, Limb::ZERO).0,
            BoxedUint::from(1234567434u64)
        );
    }
}
