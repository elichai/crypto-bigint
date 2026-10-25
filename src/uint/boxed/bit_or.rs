//! [`BoxedUint`] bitwise OR operations.

use crate::{BitOr, BitOrAssign, BoxedUint, CtOption, Limb};

impl BoxedUint {
    /// Computes bitwise `a | b`.
    #[inline(always)]
    #[must_use]
    pub fn bitor(&self, rhs: &Self) -> Self {
        Self::map_limbs(self, rhs, Limb::bitor)
    }

    /// Perform wrapping bitwise `OR`.
    ///
    /// There's no way wrapping could ever happen.
    /// This function exists so that all operations are accounted for in the wrapping operations
    #[must_use]
    pub fn wrapping_or(&self, rhs: &Self) -> Self {
        self.bitor(rhs)
    }

    /// Perform checked bitwise `OR`, returning a [`CtOption`] which `is_some` always
    #[must_use]
    pub fn checked_or(&self, rhs: &Self) -> CtOption<Self> {
        CtOption::some(self.bitor(rhs))
    }
}

impl BitOr for BoxedUint {
    type Output = Self;

    fn bitor(self, rhs: Self) -> BoxedUint {
        self.bitor(&rhs)
    }
}

impl BitOr<&BoxedUint> for BoxedUint {
    type Output = BoxedUint;

    #[allow(clippy::needless_borrow)]
    fn bitor(self, rhs: &BoxedUint) -> BoxedUint {
        (&self).bitor(rhs)
    }
}

impl BitOr<BoxedUint> for &BoxedUint {
    type Output = BoxedUint;

    fn bitor(self, rhs: BoxedUint) -> BoxedUint {
        self.bitor(&rhs)
    }
}

impl BitOr<&BoxedUint> for &BoxedUint {
    type Output = BoxedUint;

    fn bitor(self, rhs: &BoxedUint) -> BoxedUint {
        self.bitor(rhs)
    }
}

impl BitOrAssign for BoxedUint {
    fn bitor_assign(&mut self, other: Self) {
        Self::bitor_assign(self, &other);
    }
}

impl BitOrAssign<&BoxedUint> for BoxedUint {
    fn bitor_assign(&mut self, other: &Self) {
        if other.limbs.len() > self.limbs.len() {
            self.resize_in_place_unchecked(other.bits_precision());
        }

        for (a, b) in self.limbs.iter_mut().zip(other.limbs.iter()) {
            *a |= *b;
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{BoxedUint, Wrapping};

    // Separate calls keep 64/128-bit heap lengths concrete for Kani.
    fn check128(result: BoxedUint, expected: u128) {
        assert_eq!((value(&result), result.bits_precision()), (expected, 128));
    }

    fn check_methods(lhs: &BoxedUint, rhs: &BoxedUint, expected: u128) {
        check128(lhs.bitor(rhs), expected);
        check128(lhs.wrapping_or(rhs), expected);
        check128(
            lhs.checked_or(rhs).into_option().expect("always Some"),
            expected,
        );
    }

    fn check_operators(lhs: &BoxedUint, rhs: &BoxedUint, expected: u128) {
        check128(lhs | rhs, expected);
        check128(lhs.clone() | rhs, expected);
        check128(lhs | rhs.clone(), expected);
        check128(lhs.clone() | rhs.clone(), expected);
        let mut assigned = lhs.clone();
        assigned |= rhs;
        check128(assigned, expected);
        let mut assigned = lhs.clone();
        assigned |= rhs.clone();
        check128(assigned, expected);
        let mut wrapped = Wrapping(lhs.clone());
        let wrapped_rhs = Wrapping(rhs.clone());
        check128((&wrapped | &wrapped_rhs).0, expected);
        wrapped |= wrapped_rhs;
        check128(wrapped.0, expected);
    }

    /// All bits of both 64/128-bit operands, including either widening direction.
    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_bitor_methods_mixed_precision() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let expected = value(&a) | value(&b);
        check_methods(&a, &b, expected);
        check_methods(&b, &a, expected);
    }

    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_bitor_operators_mixed_precision() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let expected = value(&a) | value(&b);
        check_operators(&a, &b, expected);
        check_operators(&b, &a, expected);
    }
}

#[cfg(test)]
mod tests {
    use crate::{BitOrAssign, BoxedUint, Limb};

    #[test]
    fn bitor_assign_preserves_wider_rhs_limbs() {
        let mut lhs = BoxedUint::zero_with_precision(Limb::BITS);
        let rhs = BoxedUint::max(Limb::BITS * 2);
        let expected = lhs.bitor(&rhs);

        lhs.bitor_assign(&rhs);

        assert_eq!(lhs, expected);
    }
}
