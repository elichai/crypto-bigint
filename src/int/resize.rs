use crate::{Int, Limb, Uint};

impl<const LIMBS: usize> Int<LIMBS> {
    /// Resize the representation of `self` to an [`Int<T>`].
    /// Warning: this operation may lead to loss of information.
    #[inline(always)]
    #[must_use]
    pub const fn resize<const T: usize>(&self) -> Int<T> {
        let mut limbs = [Limb::select(Limb::ZERO, Limb::MAX, self.is_negative()); T];
        let mut i = 0;
        let dim = if T < LIMBS { T } else { LIMBS };
        while i < dim {
            limbs[i] = self.0.limbs[i];
            i += 1;
        }
        *Uint { limbs }.as_int()
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{I64, I128, I256, Limb};

    #[kani::proof]
    #[kani::unwind(9)]
    fn int_resize_sign_extension_native() {
        let x: i128 = kani::any();
        let wide = I128::from_i128(x).resize::<{ I256::LIMBS }>();
        assert_eq!(i128::from(wide.resize::<{ I128::LIMBS }>()), x);
        let words = wide.as_limbs();
        let mut i = I128::LIMBS;
        while i < I256::LIMBS {
            assert_eq!(words[i], if x < 0 { Limb::MAX } else { Limb::ZERO });
            i += 1;
        }
    }

    #[kani::proof]
    #[kani::unwind(5)]
    fn int_resize_truncation_native() {
        let x: i128 = kani::any();
        let narrow: I64 = I128::from_i128(x).resize();
        assert_eq!(i64::from(narrow), x as i64);
        assert_eq!(
            i128::from(narrow.resize::<{ I128::LIMBS }>()),
            (x as i64) as i128
        );
    }
}

#[cfg(test)]
mod tests {

    use crate::{I128, I256};

    #[test]
    fn scale_down() {
        assert_eq!(I256::MIN.resize::<{ I128::LIMBS }>(), I128::ZERO);
        assert_eq!(I256::MINUS_ONE.resize::<{ I128::LIMBS }>(), I128::MINUS_ONE);
        assert_eq!(I256::ZERO.resize::<{ I128::LIMBS }>(), I128::ZERO);
        assert_eq!(I256::ONE.resize::<{ I128::LIMBS }>(), I128::ONE);
        assert_eq!(I256::MAX.resize::<{ I128::LIMBS }>(), I128::MINUS_ONE);
    }

    #[test]
    fn scale_up() {
        assert_eq!(
            I128::MIN.resize::<{ I256::LIMBS }>(),
            I256::ZERO.wrapping_sub(&I256 {
                0: I128::MIN.0.resize()
            })
        );
        assert_eq!(I128::MINUS_ONE.resize::<{ I256::LIMBS }>(), I256::MINUS_ONE);
        assert_eq!(I128::ZERO.resize::<{ I256::LIMBS }>(), I256::ZERO);
        assert_eq!(I128::ONE.resize::<{ I256::LIMBS }>(), I256::ONE);
        assert_eq!(
            I128::MAX.resize::<{ I256::LIMBS }>(),
            *I128::MAX.0.resize().as_int()
        );
    }
}
