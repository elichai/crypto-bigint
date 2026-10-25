//! [`BoxedUint`] bitwise NOT operations.

use super::BoxedUint;
use crate::Limb;
use core::ops::Not;

impl BoxedUint {
    /// Computes bitwise `!a`.
    #[must_use]
    pub fn not(&self) -> Self {
        let mut limbs = vec![Limb::ZERO; self.nlimbs()];

        for i in 0..self.nlimbs() {
            limbs[i] = self.limbs[i].not();
        }

        limbs.into()
    }
}

impl Not for BoxedUint {
    type Output = Self;

    fn not(self) -> Self {
        BoxedUint::not(&self)
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{BoxedUint, Wrapping};

    // Separate calls keep 64/128-bit heap lengths concrete for Kani.
    fn check(result: BoxedUint, expected: (u128, u32)) {
        assert_eq!((value(&result), result.bits_precision()), expected);
    }

    fn check_not(input: BoxedUint) {
        let bits = input.bits_precision();
        let mask = u128::MAX >> (u128::BITS - bits);
        let expected = (!value(&input) & mask, bits);
        let wrapped = Wrapping(input.clone());
        check(input.not(), expected);
        check(!input, expected);
        check((!&wrapped).0, expected);
        check((!wrapped).0, expected);
    }

    /// NOT flips only the allocated 64/128 bits and preserves that precision.
    #[kani::proof]
    #[kani::unwind(9)]
    fn boxed_bitnot_preserves_precision() {
        check_not(any_boxed::<L64>());
        check_not(any_boxed::<{ 2 * L64 }>());
    }
}

#[cfg(test)]
mod tests {
    use crate::BoxedUint;

    #[test]
    fn bitnot_ok() {
        assert_eq!(
            BoxedUint::zero_with_precision(128).not(),
            BoxedUint::max(128)
        );
        assert_eq!(
            BoxedUint::max(128).not(),
            BoxedUint::zero_with_precision(128)
        );
    }
}
