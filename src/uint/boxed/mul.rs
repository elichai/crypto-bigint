//! [`BoxedUint`] multiplication operations.

use crate::{
    BoxedUint, CheckedMul, Choice, ConcatenatingMul, ConcatenatingSquare, CtOption, Limb, Mul,
    MulAssign, UintRef, Wrapping, WrappingMul,
};

mod helper;
pub(crate) use helper::BoxedMultiplier;

impl BoxedUint {
    /// Multiply `self` by `rhs`.
    ///
    /// Returns a widened output with a limb count equal to the sums of the input limb counts.
    #[must_use]
    #[deprecated(since = "0.7.0", note = "please use `concatenating_mul`")]
    pub fn mul(&self, rhs: impl AsRef<UintRef>) -> Self {
        self.concatenating_mul(rhs)
    }

    /// Perform wrapping multiplication, wrapping to the width of `self`.
    #[must_use]
    pub fn wrapping_mul(&self, rhs: impl AsRef<UintRef>) -> Self {
        self.wrapping_mul_carry(rhs.as_ref(), self.nlimbs()).0
    }

    /// Multiply `self` by `rhs`, wrapping to the width of `self`.
    /// Returns `CtOption::None` if the result overflowed the precision of `self`.
    #[must_use]
    pub fn checked_mul(&self, rhs: impl AsRef<UintRef>) -> CtOption<Self> {
        let (res, overflow) = self.overflowing_mul(rhs.as_ref(), self.nlimbs());
        CtOption::new(res, overflow.not())
    }

    /// Perform saturating multiplication, returning `MAX` on overflow.
    #[must_use]
    pub fn saturating_mul(&self, rhs: impl AsRef<UintRef>) -> Self {
        let (mut res, overflow) = self.overflowing_mul(rhs.as_ref(), self.nlimbs());
        res.as_mut_uint_ref().conditional_set_max(overflow);
        res
    }

    /// Multiply `self` by `rhs`, wrapping to the width of `self`.
    /// Returns a `Choice` indicating if the result overflowed the precision of `self`.
    #[inline(always)]
    #[must_use]
    pub(crate) fn overflowing_mul(&self, rhs: &UintRef, size: usize) -> (Self, Choice) {
        let mut limbs = vec![Limb::ZERO; size];
        let overflow = self
            .as_uint_ref()
            .overflowing_mul(rhs, UintRef::new_mut(&mut limbs));
        (limbs.into(), overflow)
    }

    /// Multiply `self` by `rhs`, wrapping to the width `size`.
    /// Returns a pair of the wrapped product and a Limb representing the carry.
    #[inline(always)]
    fn wrapping_mul_carry(&self, rhs: &UintRef, size: usize) -> (Self, Limb) {
        let mut limbs = vec![Limb::ZERO; size];
        let carry = self
            .as_uint_ref()
            .wrapping_mul(rhs, UintRef::new_mut(limbs.as_mut_slice()));
        (limbs.into(), carry)
    }

    /// Multiply `self` by itself.
    #[must_use]
    #[deprecated(since = "0.7.0", note = "please use `concatenating_square`")]
    pub fn square(&self) -> Self {
        self.concatenating_square()
    }

    /// Multiply `self` by itself, wrapping to the width of `self`.
    #[must_use]
    pub fn wrapping_square(&self) -> Self {
        self.wrapping_square_carry(self.nlimbs()).0
    }

    /// Multiply `self` by itself, wrapping to the width of `self`.
    /// Returns `CtOption::None` if the result overflowed the precision of `self`.
    #[must_use]
    pub fn checked_square(&self) -> CtOption<Self> {
        let (res, overflow) = self.overflowing_square(self.nlimbs());
        CtOption::new(res, overflow.not())
    }

    /// Perform saturating squaring, returning `MAX` on overflow.
    #[must_use]
    pub fn saturating_square(&self) -> Self {
        let (mut res, overflow) = self.overflowing_square(self.nlimbs());
        res.as_mut_uint_ref().conditional_set_max(overflow);
        res
    }

    /// Multiply `self` by itself, wrapping to the width of `self`.
    /// Returns a `Choice` indicating if the result overflowed the precision of `self`.
    #[inline(always)]
    #[must_use]
    pub(crate) fn overflowing_square(&self, size: usize) -> (Self, Choice) {
        let mut limbs = vec![Limb::ZERO; size];
        let overflow = self
            .as_uint_ref()
            .overflowing_square(UintRef::new_mut(&mut limbs));
        (limbs.into(), overflow)
    }

    /// Multiply `self` by itself, wrapping to the width `size`.
    /// Returns a pair of the wrapped product and a Limb representing the carry.
    #[inline(always)]
    #[must_use]
    fn wrapping_square_carry(&self, size: usize) -> (Self, Limb) {
        let mut limbs = vec![Limb::ZERO; size];
        let carry = self
            .as_uint_ref()
            .wrapping_square(UintRef::new_mut(limbs.as_mut_slice()));
        (limbs.into(), carry)
    }
}

impl<Rhs: AsRef<UintRef>> CheckedMul<Rhs> for BoxedUint {
    fn checked_mul(&self, rhs: &Rhs) -> CtOption<Self> {
        self.checked_mul(rhs)
    }
}

impl<Rhs: AsRef<UintRef>> Mul<Rhs> for BoxedUint {
    type Output = BoxedUint;

    #[track_caller]
    fn mul(self, rhs: Rhs) -> Self {
        Mul::mul(&self, rhs)
    }
}

impl<Rhs: AsRef<UintRef>> Mul<Rhs> for &BoxedUint {
    type Output = BoxedUint;

    #[track_caller]
    fn mul(self, rhs: Rhs) -> Self::Output {
        self.checked_mul(rhs)
            .expect("attempted to multiply with overflow")
    }
}

impl<Rhs: AsRef<UintRef>> MulAssign<Rhs> for BoxedUint {
    fn mul_assign(&mut self, rhs: Rhs) {
        *self = self
            .checked_mul(rhs)
            .expect("attempted to multiply with overflow");
    }
}

impl<Rhs: AsRef<UintRef>> MulAssign<Rhs> for Wrapping<BoxedUint> {
    fn mul_assign(&mut self, other: Rhs) {
        self.0 = self.0.wrapping_mul(other);
    }
}

impl<Rhs: AsRef<UintRef>> ConcatenatingMul<Rhs> for BoxedUint {
    type Output = Self;

    #[inline]
    fn concatenating_mul(&self, rhs: Rhs) -> Self {
        let rhs = rhs.as_ref();
        self.wrapping_mul_carry(rhs, self.nlimbs() + rhs.nlimbs()).0
    }
}

impl ConcatenatingSquare for BoxedUint {
    type Output = Self;

    #[inline]
    fn concatenating_square(&self) -> Self {
        self.wrapping_square_carry(self.nlimbs() * 2).0
    }
}

impl WrappingMul for BoxedUint {
    fn wrapping_mul(&self, v: &Self) -> Self {
        self.wrapping_mul(v)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BoxedUint, CheckedMul, ConcatenatingMul, ConcatenatingSquare, Limb, Resize, UintRef,
        WrappingMul,
    };

    #[test]
    fn mul_zero_and_one() {
        assert!(bool::from(
            BoxedUint::zero()
                .concatenating_mul(BoxedUint::zero())
                .is_zero()
        ));
        assert!(bool::from(
            BoxedUint::zero()
                .concatenating_mul(BoxedUint::one())
                .is_zero()
        ));
        assert!(bool::from(
            BoxedUint::one()
                .concatenating_mul(BoxedUint::zero())
                .is_zero()
        ));
        assert_eq!(
            BoxedUint::one().concatenating_mul(BoxedUint::one()),
            BoxedUint::one()
        );
        assert_eq!(BoxedUint::zero().concatenating_square(), BoxedUint::zero());
        assert_eq!(BoxedUint::one().concatenating_square(), BoxedUint::one());
    }

    #[test]
    fn mul_primes() {
        let primes: &[u32] = &[3, 5, 17, 257, 65537];

        for &a_int in primes {
            for &b_int in primes {
                let actual = BoxedUint::from(a_int).concatenating_mul(BoxedUint::from(b_int));
                let expected = BoxedUint::from(u64::from(a_int) * u64::from(b_int));
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn mul_trait() {
        let expect = BoxedUint::one() * BoxedUint::from(2u8);
        assert_eq!(expect, BoxedUint::one() * &BoxedUint::from(2u8));
    }

    #[should_panic]
    #[test]
    fn mul_trait_panic() {
        let _ = BoxedUint::max(64) * BoxedUint::max(64);
    }

    #[cfg(feature = "rand_core")]
    #[test]
    fn mul_cmp() {
        use crate::{RandomBits, Resize};
        use rand_core::SeedableRng;
        let mut rng = chacha20::ChaCha8Rng::seed_from_u64(1);
        let bits = if cfg!(miri) { 512 } else { 4096 };
        let rounds = if cfg!(miri) { 10 } else { 50 };

        for i in 0..rounds {
            let a = BoxedUint::random_bits(&mut rng, bits);
            assert_eq!(
                a.concatenating_mul(&a),
                a.concatenating_square(),
                "a={a}, i={i}"
            );
            assert_eq!(a.wrapping_mul(&a), a.wrapping_square(), "a={a}, i={i}");
            assert_eq!(a.saturating_mul(&a), a.saturating_square(), "a={a}, i={i}");
        }

        for i in 0..rounds {
            let a = BoxedUint::random_bits(&mut rng, bits);
            let b = BoxedUint::random_bits(&mut rng, bits + 64);
            let expect = a.concatenating_mul(&b);
            assert_eq!(
                ConcatenatingMul::concatenating_mul(&b, &a),
                expect,
                "a={a}, b={b}, i={i}"
            );
            assert_eq!(
                WrappingMul::wrapping_mul(&a, &b),
                expect.clone().resize_unchecked(a.bits_precision()),
                "a={a}, b={b}, i={i}"
            );
            assert_eq!(
                WrappingMul::wrapping_mul(&b, &a),
                expect.clone().resize_unchecked(b.bits_precision()),
                "a={a}, b={b}, i={i}"
            );
        }
    }

    #[test]
    fn checked_mul_cmp() {
        let n = BoxedUint::max(64)
            .resize_unchecked(256)
            .wrapping_add(BoxedUint::one());
        let n2 = n.checked_square();
        assert!(n2.is_some().to_bool());
        let n2_c = CheckedMul::checked_mul(&n, &n);
        assert_eq!(n2.clone().into_option(), n2_c.into_option());
        let n2 = n2.unwrap();

        let n4 = n2.checked_square();
        assert!(n4.is_none().to_bool());
        let n4_c = CheckedMul::checked_mul(&n2, &n2);
        assert!(n4_c.is_none().to_bool());

        let z = BoxedUint::zero_with_precision(256);
        let z2 = z.checked_square();
        assert!(z2.is_some().to_bool());
        let z2_c = CheckedMul::checked_mul(&z, &z);
        assert_eq!(z2.into_option(), z2_c.into_option());

        let m = BoxedUint::max(256);
        assert!(m.checked_square().is_none().to_bool());
        assert!(CheckedMul::checked_mul(&m, &m).is_none().to_bool());
    }

    #[test]
    fn mul_uintref() {
        let a = BoxedUint::from(1234567890u64);
        let b = UintRef::new(&[Limb(456), Limb(0)]);
        assert_eq!(a * b, BoxedUint::from(562962957840u64));
    }

    #[test]
    fn overflow_check() {
        for bits in (64..4096).step_by(2 * Limb::BITS as usize) {
            let mut b = BoxedUint::max(bits);
            b.restrict_bits(bits >> 1);
            assert!(b.checked_mul(&b).is_some().to_bool());
            assert!(b.checked_square().is_some().to_bool());
            b.wrapping_add_assign(Limb::ONE);
            assert!(b.checked_mul(&b).is_none().to_bool());
            assert!(b.checked_square().is_none().to_bool());
            assert!(b.saturating_mul(&b) == BoxedUint::max(bits));
            assert!(b.saturating_square() == BoxedUint::max(bits));
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{BoxedUint, ConcatenatingMul, ConcatenatingSquare, Limb, Wrapping};

    fn assert_value(result: &BoxedUint, expected: u128, bits: u32) {
        assert_eq!(value(result), expected);
        assert_eq!(result.bits_precision(), bits);
    }

    fn native_product(a: u64, b: u64) -> u128 {
        if Limb::BITS == 32 {
            let (a_lo, a_hi) = (u64::from(a as u32), a >> 32);
            let (b_lo, b_hi) = (u64::from(b as u32), b >> 32);
            // Exact radix-2^32 multiplication, carrying after each native partial product.
            let p00 = a_lo * b_lo;
            let p01 = a_lo * b_hi + (p00 >> 32);
            let p10 = a_hi * b_lo + u64::from(p01 as u32);
            let p11 = a_hi * b_hi + (p01 >> 32) + (p10 >> 32);
            u128::from(p00 as u32) | (u128::from(p10 as u32) << 32) | (u128::from(p11) << 64)
        } else {
            u128::from(a) * u128::from(b)
        }
    }

    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(6)]
    fn boxed_mul_wide_64() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<L64>());
        let expected = native_product(value(&a) as u64, value(&b) as u64);
        let (result, carry) = a.wrapping_mul_carry(b.as_uint_ref(), 2 * L64);
        assert_value(&result, expected, 128);
        assert_eq!(carry, Limb::ZERO);
        assert_value(&a.concatenating_mul(&b), expected, 128);
    }

    #[kani::proof]
    #[cfg_attr(cpubits = "32", kani::solver(cadical))]
    #[cfg_attr(not(cpubits = "32"), kani::solver(z3))]
    #[kani::unwind(6)]
    fn boxed_mul_wrapping_64() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<L64>());
        let expected = native_product(value(&a) as u64, value(&b) as u64) as u64;
        assert_value(&a.wrapping_mul(&b), u128::from(expected), 64);
        let mut assigned = Wrapping(a);
        assigned *= &b;
        assert_value(&assigned.0, u128::from(expected), 64);
    }

    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(6)]
    fn boxed_mul_checked_saturating_64() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<L64>());
        let expected = native_product(value(&a) as u64, value(&b) as u64);
        let fits = expected <= u128::from(u64::MAX);
        let checked = a.checked_mul(&b);
        assert_eq!(checked.is_some().to_bool(), fits);
        if fits {
            assert_value(&checked.unwrap(), expected, 64);
        }
        assert_value(
            &a.saturating_mul(&b),
            if fits { expected } else { u128::from(u64::MAX) },
            64,
        );
    }

    #[kani::proof]
    #[cfg_attr(cpubits = "32", kani::solver(cadical))]
    #[cfg_attr(not(cpubits = "32"), kani::solver(z3))]
    #[kani::unwind(6)]
    fn boxed_mul_overflowing_64() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<L64>());
        let expected = native_product(value(&a) as u64, value(&b) as u64);
        let (result, overflow) = a.overflowing_mul(b.as_uint_ref(), L64);
        assert_value(&result, u128::from(expected as u64), 64);
        assert_eq!(overflow.to_bool(), expected >> 64 != 0);
    }

    fn native_square(a: u64) -> u128 {
        if Limb::BITS == 32 {
            let (lo, hi) = (u64::from(a as u32), a >> 32);
            u128::from(lo * lo) + (u128::from(lo * hi) << 33) + (u128::from(hi * hi) << 64)
        } else {
            u128::from(a) * u128::from(a)
        }
    }

    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(6)]
    fn boxed_square_wide_64() {
        let a = any_boxed::<L64>();
        let expected = native_square(value(&a) as u64);
        let (result, carry) = a.wrapping_square_carry(2 * L64);
        assert_value(&result, expected, 128);
        assert_eq!(carry, Limb::ZERO);
        assert_value(&a.concatenating_square(), expected, 128);
    }

    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(6)]
    fn boxed_square_wrapping_overflowing_64() {
        let a = any_boxed::<L64>();
        let expected = native_square(value(&a) as u64);
        let (result, overflow) = a.overflowing_square(L64);
        assert_value(&result, u128::from(expected as u64), 64);
        assert_eq!(overflow.to_bool(), expected >> 64 != 0);
        assert_value(&a.wrapping_square(), u128::from(expected as u64), 64);
    }

    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(6)]
    fn boxed_square_checked_saturating_64() {
        let a = any_boxed::<L64>();
        let expected = native_square(value(&a) as u64);
        let fits = expected <= u128::from(u64::MAX);
        let checked = a.checked_square();
        assert_eq!(checked.is_some().to_bool(), fits);
        if fits {
            assert_value(&checked.unwrap(), expected, 64);
        }
        assert_value(
            &a.saturating_square(),
            if fits { expected } else { u128::from(u64::MAX) },
            64,
        );
    }

    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(6)]
    fn boxed_mul_operators_64() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<L64>());
        let expected = native_product(value(&a) as u64, value(&b) as u64);
        kani::assume(expected <= u128::from(u64::MAX));
        assert_value(&(&a * &b), expected, 64);
        let mut assigned = a;
        assigned *= b;
        assert_value(&assigned, expected, 64);
    }

    /// A wider rhs can contribute overflow above the retained output, including when lhs is zero.
    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(6)]
    fn boxed_mul_wider_rhs() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let (va, vb) = (value(&a), value(&b));
        let low_product = native_product(va as u64, vb as u64);
        let overflow = low_product >> 64 != 0 || (va != 0 && vb >> 64 != 0);
        let (result, actual_overflow) = a.overflowing_mul(b.as_uint_ref(), L64);
        assert_value(&result, u128::from(low_product as u64), 64);
        assert_eq!(actual_overflow.to_bool(), overflow);
    }

    /// A single retained word returns the complete next word as carry, including nonzero carries.
    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(6)]
    fn boxed_mul_carry_word() {
        let (a, b) = (any_boxed::<1>(), any_boxed::<1>());
        let expected = u128::from(a.as_words()[0]) * u128::from(b.as_words()[0]);
        let (result, carry) = a.wrapping_mul_carry(b.as_uint_ref(), 1);
        assert_value(&result, expected & u128::from(crate::Word::MAX), Limb::BITS);
        assert_eq!(u128::from(carry.0), expected >> Limb::BITS);
    }
}
