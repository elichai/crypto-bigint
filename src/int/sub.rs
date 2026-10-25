//! [`Int`] subtraction operations.

use crate::{Checked, CheckedSub, Choice, CtOption, Int, Sub, SubAssign, Wrapping, WrappingSub};

impl<const LIMBS: usize> Int<LIMBS> {
    /// Perform subtraction, returning the result along with a [`Choice`] which `is_true`
    /// only if the operation underflowed.
    #[inline]
    #[must_use]
    pub const fn underflowing_sub(&self, rhs: &Self) -> (Self, Choice) {
        // Step 1. subtract operands
        let res = Self(self.0.wrapping_sub(&rhs.0));

        // Step 2. check whether underflow happened.
        // Note:
        // - underflow can only happen when the inputs have opposing signs, and
        // - underflow occurs if and only if the result and the lhs have opposing signs.
        //
        // We can thus express the overflow flag as: (self.msb != rhs.msb) & (self.msb != res.msb)
        let self_msb = self.is_negative();
        let underflow = self_msb
            .ne(rhs.is_negative())
            .and(self_msb.ne(res.is_negative()));

        // Step 3. Construct result
        (res, underflow)
    }

    /// Perform wrapping subtraction, discarding underflow and wrapping around the boundary of the
    /// type.
    #[inline]
    #[must_use]
    pub const fn wrapping_sub(&self, rhs: &Self) -> Self {
        self.underflowing_sub(rhs).0
    }
}

impl<const LIMBS: usize> CheckedSub for Int<LIMBS> {
    fn checked_sub(&self, rhs: &Self) -> CtOption<Self> {
        let (res, underflow) = Self::underflowing_sub(self, rhs);
        CtOption::new(res, underflow.not())
    }
}

impl<const LIMBS: usize> Sub for Int<LIMBS> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.sub(&rhs)
    }
}

impl<const LIMBS: usize> Sub<&Int<LIMBS>> for Int<LIMBS> {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self {
        self.checked_sub(rhs)
            .expect("attempted to subtract with underflow")
    }
}

impl<const LIMBS: usize> SubAssign<Int<LIMBS>> for Int<LIMBS> {
    fn sub_assign(&mut self, rhs: Int<LIMBS>) {
        *self = self.sub(&rhs);
    }
}

impl<const LIMBS: usize> SubAssign<&Int<LIMBS>> for Int<LIMBS> {
    fn sub_assign(&mut self, rhs: &Int<LIMBS>) {
        *self = self.sub(rhs);
    }
}

impl<const LIMBS: usize> SubAssign for Wrapping<Int<LIMBS>> {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl<const LIMBS: usize> SubAssign<&Wrapping<Int<LIMBS>>> for Wrapping<Int<LIMBS>> {
    fn sub_assign(&mut self, other: &Self) {
        *self = *self - other;
    }
}

impl<const LIMBS: usize> SubAssign for Checked<Int<LIMBS>> {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl<const LIMBS: usize> SubAssign<&Checked<Int<LIMBS>>> for Checked<Int<LIMBS>> {
    fn sub_assign(&mut self, other: &Self) {
        *self = *self - other;
    }
}

impl<const LIMBS: usize> WrappingSub for Int<LIMBS> {
    fn wrapping_sub(&self, v: &Self) -> Self {
        Self(self.0.wrapping_sub(&v.0))
    }
}

#[cfg(test)]
#[allow(clippy::init_numbered_fields)]
mod tests {
    use crate::{CheckedSub, I128, Int, U128};

    #[test]
    fn checked_sub() {
        let min_plus_one = Int {
            0: I128::MIN.0.wrapping_add(&I128::ONE.0),
        };
        let max_minus_one = Int {
            0: I128::MAX.0.wrapping_sub(&I128::ONE.0),
        };
        let two = Int {
            0: U128::from(2u32),
        };
        let min_plus_two = Int {
            0: I128::MIN.0.wrapping_add(&two.0),
        };

        // lhs = MIN

        let result = I128::MIN.checked_sub(&I128::MIN);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::MIN.checked_sub(&I128::MINUS_ONE);
        assert_eq!(result.unwrap(), min_plus_one);

        let result = I128::MIN.checked_sub(&I128::ZERO);
        assert_eq!(result.unwrap(), I128::MIN);

        let result = I128::MIN.checked_sub(&I128::ONE);
        assert!(bool::from(result.is_none()));

        let result = I128::MIN.checked_sub(&I128::MAX);
        assert!(bool::from(result.is_none()));

        // lhs = -1

        let result = I128::MINUS_ONE.checked_sub(&I128::MIN);
        assert_eq!(result.unwrap(), I128::MAX);

        let result = I128::MINUS_ONE.checked_sub(&I128::MINUS_ONE);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::MINUS_ONE.checked_sub(&I128::ZERO);
        assert_eq!(result.unwrap(), I128::MINUS_ONE);

        let result = I128::MINUS_ONE.checked_sub(&I128::ONE);
        assert_eq!(result.unwrap(), two.wrapping_neg());

        let result = I128::MINUS_ONE.checked_sub(&I128::MAX);
        assert_eq!(result.unwrap(), I128::MIN);

        // lhs = 0

        let result = I128::ZERO.checked_sub(&I128::MIN);
        assert!(bool::from(result.is_none()));

        let result = I128::ZERO.checked_sub(&I128::MINUS_ONE);
        assert_eq!(result.unwrap(), I128::ONE);

        let result = I128::ZERO.checked_sub(&I128::ZERO);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::ZERO.checked_sub(&I128::ONE);
        assert_eq!(result.unwrap(), I128::MINUS_ONE);

        let result = I128::ZERO.checked_sub(&I128::MAX);
        assert_eq!(result.unwrap(), min_plus_one);

        // lhs = 1

        let result = I128::ONE.checked_sub(&I128::MIN);
        assert!(bool::from(result.is_none()));

        let result = I128::ONE.checked_sub(&I128::MINUS_ONE);
        assert_eq!(result.unwrap(), two);

        let result = I128::ONE.checked_sub(&I128::ZERO);
        assert_eq!(result.unwrap(), I128::ONE);

        let result = I128::ONE.checked_sub(&I128::ONE);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::ONE.checked_sub(&I128::MAX);
        assert_eq!(result.unwrap(), min_plus_two);

        // lhs = MAX

        let result = I128::MAX.checked_sub(&I128::MIN);
        assert!(bool::from(result.is_none()));

        let result = I128::MAX.checked_sub(&I128::MINUS_ONE);
        assert!(bool::from(result.is_none()));

        let result = I128::MAX.checked_sub(&I128::ZERO);
        assert_eq!(result.unwrap(), I128::MAX);

        let result = I128::MAX.checked_sub(&I128::ONE);
        assert_eq!(result.unwrap(), max_minus_one);

        let result = I128::MAX.checked_sub(&I128::MAX);
        assert_eq!(result.unwrap(), I128::ZERO);
    }

    #[test]
    fn wrapping_sub() {
        let min_plus_one = Int {
            0: I128::MIN.0.wrapping_add(&I128::ONE.0),
        };
        let two = Int {
            0: U128::from(2u32),
        };
        let max_minus_one = Int {
            0: I128::MAX.0.wrapping_sub(&I128::ONE.0),
        };

        // + sub -
        let result = I128::ONE.wrapping_sub(&I128::MIN);
        assert_eq!(result, min_plus_one);

        // 0 sub -
        let result = I128::ZERO.wrapping_sub(&I128::MIN);
        assert_eq!(result, I128::MIN);

        // - sub +
        let result = I128::MIN.wrapping_sub(&two);
        assert_eq!(result, max_minus_one);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{Checked, CheckedSub, I128, Wrapping};

    #[kani::proof]
    fn int_sub_correct() {
        let (a, b): (i128, i128) = kani::any();
        let (ia, ib) = (I128::from_i128(a), I128::from_i128(b));

        let (diff, underflow) = ia.underflowing_sub(&ib);
        assert_eq!(
            (i128::from(diff), underflow.to_bool()),
            a.overflowing_sub(b)
        );
        assert_eq!(i128::from(ia.wrapping_sub(&ib)), a.wrapping_sub(b));
        assert_eq!(
            Option::<I128>::from(ia.checked_sub(&ib)).map(i128::from),
            a.checked_sub(b)
        );
    }

    #[kani::proof]
    #[kani::unwind(5)]
    fn int_sub_wrappers_native() {
        let (a, b): (i128, i128) = kani::any();
        let (ia, ib) = (I128::from_i128(a), I128::from_i128(b));
        let mut wrapped = Wrapping(ia);
        wrapped -= Wrapping(ib);
        assert_eq!(i128::from(wrapped.0), a.wrapping_sub(b));
        wrapped = Wrapping(ia);
        wrapped -= &Wrapping(ib);
        assert_eq!(i128::from(wrapped.0), a.wrapping_sub(b));
        let mut checked = Checked::new(ia);
        checked -= Checked::new(ib);
        assert_eq!(
            Option::<I128>::from(checked).map(i128::from),
            a.checked_sub(b)
        );
        checked = Checked::new(ia);
        checked -= &Checked::new(ib);
        assert_eq!(
            Option::<I128>::from(checked).map(i128::from),
            a.checked_sub(b)
        );
    }

    #[kani::proof]
    #[kani::unwind(5)]
    fn int_sub_operators_native() {
        let (a, b): (i128, i128) = kani::any();
        let Some(expected) = a.checked_sub(b) else {
            return;
        };
        let (ia, ib) = (I128::from_i128(a), I128::from_i128(b));
        assert_eq!(i128::from(ia - ib), expected);
        assert_eq!(i128::from(ia - &ib), expected);
        let mut assigned = ia;
        assigned -= ib;
        assert_eq!(i128::from(assigned), expected);
        assigned = ia;
        assigned -= &ib;
        assert_eq!(i128::from(assigned), expected);
    }
}
