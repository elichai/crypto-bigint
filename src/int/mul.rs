//! [`Int`] multiplication operations.

use crate::{
    Checked, CheckedMul, Choice, Concat, CtOption, Int, Mul, MulAssign, Uint, WrappingMul,
};

impl<const LIMBS: usize> Int<LIMBS> {
    /// Compute "wide" multiplication as a 3-tuple `(lo, hi, negate)`.
    /// The `(lo, hi)` components contain the _magnitude of the product_, with sizes
    /// corresponding to the sizes of the operands; `negate` indicates whether the result should be
    /// negated when converted from [`Uint`] to [`Int`].
    ///
    /// Note: even if `negate` is truthy, the magnitude might be zero!
    #[deprecated(since = "0.7.0", note = "please use `widening_mul` instead")]
    #[must_use]
    pub const fn split_mul<const RHS_LIMBS: usize>(
        &self,
        rhs: &Int<RHS_LIMBS>,
    ) -> (Uint<{ LIMBS }>, Uint<{ RHS_LIMBS }>, Choice) {
        self.widening_mul(rhs)
    }

    /// Compute "wide" multiplication as a 3-tuple `(lo, hi, negate)`.
    /// The `(lo, hi)` components contain the _magnitude of the product_, with sizes
    /// corresponding to the sizes of the operands; `negate` indicates whether the result should be
    /// negated when converted from [`Uint`] to [`Int`].
    ///
    /// Note: even if `negate` is truthy, the magnitude might be zero!
    #[inline]
    #[must_use]
    pub const fn widening_mul<const RHS_LIMBS: usize>(
        &self,
        rhs: &Int<RHS_LIMBS>,
    ) -> (Uint<{ LIMBS }>, Uint<{ RHS_LIMBS }>, Choice) {
        // Step 1: split operands into their signs and magnitudes.
        let (lhs_abs, lhs_sgn) = self.abs_sign();
        let (rhs_abs, rhs_sgn) = rhs.abs_sign();

        // Step 2: multiply the magnitudes
        let (lo, hi) = lhs_abs.widening_mul(&rhs_abs);

        // Step 3. Determine if the result should be negated.
        // This should be done if and only if lhs and rhs have opposing signs.
        // Note: if either operand is zero, the resulting magnitude will also be zero. Negating
        // zero, however, still yields zero, so having a truthy `negate` in that scenario is OK.
        let negate = lhs_sgn.xor(rhs_sgn);

        (lo, hi, negate)
    }

    /// Multiply `self` by `rhs`, returning a concatenated "wide" result.
    #[inline]
    #[must_use]
    pub const fn concatenating_mul<const RHS_LIMBS: usize, const WIDE_LIMBS: usize>(
        &self,
        rhs: &Int<RHS_LIMBS>,
    ) -> Int<WIDE_LIMBS>
    where
        Uint<LIMBS>: Concat<RHS_LIMBS, Output = Uint<WIDE_LIMBS>>,
    {
        let (lhs_abs, lhs_sign) = self.abs_sign();
        let (rhs_abs, rhs_sign) = rhs.abs_sign();
        let product_abs = lhs_abs.concatenating_mul(&rhs_abs);
        let product_sign = lhs_sign.xor(rhs_sign);

        // always fits
        Int::from_bits(product_abs.wrapping_neg_if(product_sign))
    }

    /// Multiply `self` by `rhs`, returning a `CtOption` which is `is_some` only if
    /// overflow did not occur.
    #[must_use]
    pub const fn checked_mul<const RHS_LIMBS: usize>(
        &self,
        rhs: &Int<RHS_LIMBS>,
    ) -> CtOption<Self> {
        let (abs_lhs, lhs_sgn) = self.abs_sign();
        let (abs_rhs, rhs_sgn) = rhs.abs_sign();
        let maybe_res = abs_lhs.checked_mul(&abs_rhs);
        Self::new_from_abs_opt_sign(maybe_res, lhs_sgn.xor(rhs_sgn))
    }

    /// Multiply `self` by `rhs`, saturating at the numeric bounds instead of overflowing.
    #[must_use]
    pub const fn saturating_mul<const RHS_LIMBS: usize>(&self, rhs: &Int<RHS_LIMBS>) -> Self {
        let (abs_lhs, lhs_sgn) = self.abs_sign();
        let (abs_rhs, rhs_sgn) = rhs.abs_sign();
        let maybe_res = abs_lhs.checked_mul(&abs_rhs);
        let is_neg = lhs_sgn.xor(rhs_sgn);
        let bound = Self::select(&Self::MAX, &Self::MIN, is_neg);
        ctutils::unwrap_or!(
            Self::new_from_abs_opt_sign(maybe_res, is_neg),
            bound,
            Self::select
        )
    }

    /// Multiply `self` by `rhs`, wrapping the result in case of overflow.
    /// This is equivalent to `(self * rhs) % (Uint::<LIMBS>::MAX + 1)`.
    #[inline]
    #[must_use]
    pub const fn wrapping_mul<const RHS_LIMBS: usize>(&self, rhs: &Int<RHS_LIMBS>) -> Self {
        if RHS_LIMBS >= LIMBS {
            Self(self.0.wrapping_mul(&rhs.0))
        } else {
            let (abs_rhs, rhs_sgn) = rhs.abs_sign();
            Self(self.0.wrapping_mul(&abs_rhs).wrapping_neg_if(rhs_sgn))
        }
    }
}

/// Squaring operations.
impl<const LIMBS: usize> Int<LIMBS> {
    /// Square self, returning a concatenated "wide" result.
    #[must_use]
    pub fn concatenating_square<const WIDE_LIMBS: usize>(&self) -> Uint<WIDE_LIMBS>
    where
        Uint<LIMBS>: Concat<LIMBS, Output = Uint<WIDE_LIMBS>>,
    {
        self.abs().concatenating_square()
    }

    /// Square self, checking that the result fits in the original [`Uint`] size.
    #[must_use]
    pub fn checked_square(&self) -> CtOption<Uint<LIMBS>> {
        self.abs().checked_square()
    }

    /// Perform wrapping square, discarding overflow.
    #[must_use]
    pub const fn wrapping_square(&self) -> Uint<LIMBS> {
        self.abs().wrapping_square()
    }

    /// Perform saturating squaring, returning `MAX` on overflow.
    #[must_use]
    pub const fn saturating_square(&self) -> Uint<LIMBS> {
        self.abs().saturating_square()
    }
}

impl<const LIMBS: usize, const RHS_LIMBS: usize> CheckedMul<Int<RHS_LIMBS>> for Int<LIMBS> {
    #[inline]
    fn checked_mul(&self, rhs: &Int<RHS_LIMBS>) -> CtOption<Self> {
        self.checked_mul(rhs)
    }
}

impl<const LIMBS: usize> WrappingMul for Int<LIMBS> {
    fn wrapping_mul(&self, v: &Self) -> Self {
        self.wrapping_mul(v)
    }
}

impl<const LIMBS: usize, const RHS_LIMBS: usize> Mul<Int<RHS_LIMBS>> for Int<LIMBS> {
    type Output = Int<LIMBS>;

    fn mul(self, rhs: Int<RHS_LIMBS>) -> Self {
        self.mul(&rhs)
    }
}

impl<const LIMBS: usize, const RHS_LIMBS: usize> Mul<&Int<RHS_LIMBS>> for Int<LIMBS> {
    type Output = Int<LIMBS>;

    fn mul(self, rhs: &Int<RHS_LIMBS>) -> Self {
        (&self).mul(rhs)
    }
}

impl<const LIMBS: usize, const RHS_LIMBS: usize> Mul<Int<RHS_LIMBS>> for &Int<LIMBS> {
    type Output = Int<LIMBS>;

    fn mul(self, rhs: Int<RHS_LIMBS>) -> Self::Output {
        self.mul(&rhs)
    }
}

impl<const LIMBS: usize, const RHS_LIMBS: usize> Mul<&Int<RHS_LIMBS>> for &Int<LIMBS> {
    type Output = Int<LIMBS>;

    fn mul(self, rhs: &Int<RHS_LIMBS>) -> Self::Output {
        self.checked_mul(rhs)
            .expect("attempted to multiply with overflow")
    }
}

impl<const LIMBS: usize, const RHS_LIMBS: usize> MulAssign<Int<RHS_LIMBS>> for Int<LIMBS> {
    fn mul_assign(&mut self, rhs: Int<RHS_LIMBS>) {
        *self = self.mul(&rhs);
    }
}

impl<const LIMBS: usize, const RHS_LIMBS: usize> MulAssign<&Int<RHS_LIMBS>> for Int<LIMBS> {
    fn mul_assign(&mut self, rhs: &Int<RHS_LIMBS>) {
        *self = self.mul(rhs);
    }
}

impl<const LIMBS: usize> MulAssign<Checked<Int<LIMBS>>> for Checked<Int<LIMBS>> {
    fn mul_assign(&mut self, other: Checked<Int<LIMBS>>) {
        *self = *self * other;
    }
}

impl<const LIMBS: usize> MulAssign<&Checked<Int<LIMBS>>> for Checked<Int<LIMBS>> {
    fn mul_assign(&mut self, other: &Checked<Int<LIMBS>>) {
        *self = *self * other;
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_wrap)]
mod tests {
    use crate::{I64, I128, I256, Int, U64, U128, U256};

    #[test]
    #[allow(clippy::init_numbered_fields)]
    fn test_checked_mul() {
        let min_plus_one = Int {
            0: I128::MIN.0.wrapping_add(&I128::ONE.0),
        };

        // lhs = min

        let result = I128::MIN.checked_mul(&I128::MIN);
        assert!(bool::from(result.is_none()));

        let result = I128::MIN.checked_mul(&I128::MINUS_ONE);
        assert!(bool::from(result.is_none()));

        let result = I128::MIN.checked_mul(&I128::ZERO);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::MIN.checked_mul(&I128::ONE);
        assert_eq!(result.unwrap(), I128::MIN);

        let result = I128::MIN.checked_mul(&I128::MAX);
        assert!(bool::from(result.is_none()));

        // lhs = -1

        let result = I128::MINUS_ONE.checked_mul(&I128::MIN);
        assert!(bool::from(result.is_none()));

        let result = I128::MINUS_ONE.checked_mul(&I128::MINUS_ONE);
        assert_eq!(result.unwrap(), I128::ONE);

        let result = I128::MINUS_ONE.checked_mul(&I128::ZERO);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::MINUS_ONE.checked_mul(&I128::ONE);
        assert_eq!(result.unwrap(), I128::MINUS_ONE);

        let result = I128::MINUS_ONE.checked_mul(&I128::MAX);
        assert_eq!(result.unwrap(), min_plus_one);

        // lhs = 0

        let result = I128::ZERO.checked_mul(&I128::MIN);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::ZERO.checked_mul(&I128::MINUS_ONE);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::ZERO.checked_mul(&I128::ZERO);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::ZERO.checked_mul(&I128::ONE);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::ZERO.checked_mul(&I128::MAX);
        assert_eq!(result.unwrap(), I128::ZERO);

        // lhs = 1

        let result = I128::ONE.checked_mul(&I128::MIN);
        assert_eq!(result.unwrap(), I128::MIN);

        let result = I128::ONE.checked_mul(&I128::MINUS_ONE);
        assert_eq!(result.unwrap(), I128::MINUS_ONE);

        let result = I128::ONE.checked_mul(&I128::ZERO);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::ONE.checked_mul(&I128::ONE);
        assert_eq!(result.unwrap(), I128::ONE);

        let result = I128::ONE.checked_mul(&I128::MAX);
        assert_eq!(result.unwrap(), I128::MAX);

        // lhs = max

        let result = I128::MAX.checked_mul(&I128::MIN);
        assert!(bool::from(result.is_none()));

        let result = I128::MAX.checked_mul(&I128::MINUS_ONE);
        assert_eq!(result.unwrap(), min_plus_one);

        let result = I128::MAX.checked_mul(&I128::ZERO);
        assert_eq!(result.unwrap(), I128::ZERO);

        let result = I128::MAX.checked_mul(&I128::ONE);
        assert_eq!(result.unwrap(), I128::MAX);

        let result = I128::MAX.checked_mul(&I128::MAX);
        assert!(bool::from(result.is_none()));
    }

    #[test]
    fn test_wrapping_mul() {
        // wrapping
        let a = 0xFFFFFFFB7B63198EF870DF1F90D9BD9Eu128 as i128;
        let b = 0xF20C29FA87B356AA3B4C05C4F9C24B4Au128 as i128;
        let z = 0xAA700D354D6CF4EE881F8FF8093A19ACu128 as i128;
        assert_eq!(a.wrapping_mul(b), z);
        assert_eq!(
            I128::from_i128(a).wrapping_mul(&I128::from_i128(b)),
            I128::from_i128(z)
        );

        // no wrapping
        let c = -12345i64;
        assert_eq!(
            I128::from_i128(a).wrapping_mul(&I128::from_i64(c)),
            I128::from_i128(a.wrapping_mul(i128::from(c)))
        );

        // overflow into MSB
        let a = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128 as i128;
        assert!(!a.is_negative() && a.wrapping_mul(a).is_negative());

        // core case
        assert_eq!(i8::MAX.wrapping_mul(2), -2);
        assert_eq!(i64::MAX.wrapping_mul(2), -2);
        assert_eq!(
            I128::MAX.wrapping_mul(&I128::from_i64(2i64)),
            I128::from_i64(-2i64)
        );

        let x = -197044252290277702i64;
        let y = -2631691865753118366;
        let z = -2988283350644101836;
        assert_eq!(x.wrapping_mul(y), z);
        assert_eq!(
            I64::from_i64(x).wrapping_mul(&I64::from_i64(y)),
            I64::from_i64(z)
        );

        let x = -86027672844719838068326470675019902915i128;
        let y = -21188806580823612823777395451044967239i128;
        let z = 11054120842379932838712398402517374997i128;
        assert_eq!(x.wrapping_mul(y), z);
        assert_eq!(
            I128::from_i128(x).wrapping_mul(&I128::from_i128(y)),
            I128::from_i128(z)
        );
    }

    #[test]
    fn test_wrapping_mul_mixed() {
        let a = U64::from_u64(0x0011223344556677);
        let b = U128::from_u128(0x8899aabbccddeeff_8899aabbccddeeff);
        let expected = a.as_int().concatenating_mul(b.as_int());
        assert_eq!(a.as_int().wrapping_mul(b.as_int()), expected.resize());
        assert_eq!(b.as_int().wrapping_mul(a.as_int()), expected.resize());
        assert_eq!(
            a.as_int().wrapping_neg().wrapping_mul(b.as_int()),
            expected.wrapping_neg().resize()
        );
        assert_eq!(
            a.as_int().wrapping_mul(&b.as_int().wrapping_neg()),
            expected.wrapping_neg().resize()
        );
        assert_eq!(
            b.as_int().wrapping_neg().wrapping_mul(a.as_int()),
            expected.wrapping_neg().resize()
        );
        assert_eq!(
            b.as_int().wrapping_mul(&a.as_int().wrapping_neg()),
            expected.wrapping_neg().resize()
        );
        assert_eq!(
            a.as_int()
                .wrapping_neg()
                .wrapping_mul(&b.as_int().wrapping_neg()),
            expected.resize()
        );
        assert_eq!(
            b.as_int()
                .wrapping_neg()
                .wrapping_mul(&a.as_int().wrapping_neg()),
            expected.resize()
        );
    }

    #[test]
    fn test_saturating_mul() {
        // wrapping
        let a = 0xFFFFFFFB7B63198EF870DF1F90D9BD9Eu128 as i128;
        let b = 0xF20C29FA87B356AA3B4C05C4F9C24B4Au128 as i128;
        assert_eq!(a.saturating_mul(b), i128::MAX);
        assert_eq!(
            I128::from_i128(a).saturating_mul(&I128::from_i128(b)),
            I128::MAX
        );

        // no wrapping
        let c = -12345i64;
        assert_eq!(
            I128::from_i128(a).saturating_mul(&I128::from_i64(c)),
            I128::from_i128(a.saturating_mul(i128::from(c)))
        );

        // core case
        assert_eq!(i8::MAX.saturating_mul(2), i8::MAX);
        assert_eq!(i8::MAX.saturating_mul(-2), i8::MIN);
        assert_eq!(i64::MAX.saturating_mul(2), i64::MAX);
        assert_eq!(i64::MAX.saturating_mul(-2), i64::MIN);
        assert_eq!(I128::MAX.saturating_mul(&I128::from_i64(2i64)), I128::MAX);
        assert_eq!(I128::MAX.saturating_mul(&I128::from_i64(-2i64)), I128::MIN);

        let x = -197044252290277702i64;
        let y = -2631691865753118366;
        assert_eq!(x.saturating_mul(y), i64::MAX);
        assert_eq!(I64::from_i64(x).saturating_mul(&I64::from_i64(y)), I64::MAX);

        let x = -86027672844719838068326470675019902915i128;
        let y = 21188806580823612823777395451044967239i128;
        assert_eq!(x.saturating_mul(y), i128::MIN);
        assert_eq!(x.saturating_mul(-y), i128::MAX);
        assert_eq!(
            I128::from_i128(x).saturating_mul(&I128::from_i128(y)),
            I128::MIN
        );
        assert_eq!(
            I128::from_i128(x).saturating_mul(&I128::from_i128(-y)),
            I128::MAX
        );
    }

    #[test]
    fn test_concatenating_mul() {
        assert_eq!(
            I128::MIN.concatenating_mul(&I128::MIN),
            I256::from_be_hex("4000000000000000000000000000000000000000000000000000000000000000")
        );
        assert_eq!(
            I128::MIN.concatenating_mul(&I128::MINUS_ONE),
            I256::from_be_hex("0000000000000000000000000000000080000000000000000000000000000000")
        );
        assert_eq!(I128::MIN.concatenating_mul(&I128::ZERO), I256::ZERO);
        assert_eq!(
            I128::MIN.concatenating_mul(&I128::ONE),
            I256::from_be_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF80000000000000000000000000000000")
        );
        assert_eq!(
            I128::MIN.concatenating_mul(&I128::MAX),
            I256::from_be_hex("C000000000000000000000000000000080000000000000000000000000000000")
        );

        assert_eq!(
            I128::MINUS_ONE.concatenating_mul(&I128::MIN),
            I256::from_be_hex("0000000000000000000000000000000080000000000000000000000000000000")
        );
        assert_eq!(
            I128::MINUS_ONE.concatenating_mul(&I128::MINUS_ONE),
            I256::ONE
        );
        assert_eq!(I128::MINUS_ONE.concatenating_mul(&I128::ZERO), I256::ZERO);
        assert_eq!(
            I128::MINUS_ONE.concatenating_mul(&I128::ONE),
            I256::MINUS_ONE
        );
        assert_eq!(
            I128::MINUS_ONE.concatenating_mul(&I128::MAX),
            I256::from_be_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF80000000000000000000000000000001")
        );

        assert_eq!(I128::ZERO.concatenating_mul(&I128::MIN), I256::ZERO);
        assert_eq!(I128::ZERO.concatenating_mul(&I128::MINUS_ONE), I256::ZERO);
        assert_eq!(I128::ZERO.concatenating_mul(&I128::ZERO), I256::ZERO);
        assert_eq!(I128::ZERO.concatenating_mul(&I128::ONE), I256::ZERO);
        assert_eq!(I128::ZERO.concatenating_mul(&I128::MAX), I256::ZERO);

        assert_eq!(
            I128::ONE.concatenating_mul(&I128::MIN),
            I256::from_be_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF80000000000000000000000000000000")
        );
        assert_eq!(
            I128::ONE.concatenating_mul(&I128::MINUS_ONE),
            I256::MINUS_ONE
        );
        assert_eq!(I128::ONE.concatenating_mul(&I128::ZERO), I256::ZERO);
        assert_eq!(I128::ONE.concatenating_mul(&I128::ONE), I256::ONE);
        assert_eq!(
            I128::ONE.concatenating_mul(&I128::MAX),
            I256::from_be_hex("000000000000000000000000000000007FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
        );

        assert_eq!(
            I128::MAX.concatenating_mul(&I128::MIN),
            I256::from_be_hex("C000000000000000000000000000000080000000000000000000000000000000")
        );
        assert_eq!(
            I128::MAX.concatenating_mul(&I128::MINUS_ONE),
            I256::from_be_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF80000000000000000000000000000001")
        );
        assert_eq!(I128::MAX.concatenating_mul(&I128::ZERO), I256::ZERO);
        assert_eq!(
            I128::MAX.concatenating_mul(&I128::ONE),
            I256::from_be_hex("000000000000000000000000000000007FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
        );
        assert_eq!(
            I128::MAX.concatenating_mul(&I128::MAX),
            I256::from_be_hex("3FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000000000000000000000000001")
        );
    }

    #[test]
    fn test_concatenating_square() {
        let res = I128::from_i64(i64::MIN).concatenating_square();
        assert_eq!(
            res,
            U256::from_be_hex("0000000000000000000000000000000040000000000000000000000000000000")
        );

        let x: I128 = I128::MINUS_ONE << 64;
        let res = x.concatenating_square();
        assert_eq!(
            res,
            U256::from_be_hex("0000000000000000000000000000000100000000000000000000000000000000")
        );
    }

    #[test]
    fn test_checked_square() {
        let res = I128::from_i64(i64::MIN).checked_square();
        assert!(res.is_some().to_bool());
        assert_eq!(
            res.unwrap(),
            U128::from_be_hex("40000000000000000000000000000000")
        );

        let x: I128 = I128::MINUS_ONE << 64;
        let res = x.checked_square();
        assert!(res.is_none().to_bool());
    }

    #[test]
    fn test_wrapping_square() {
        let res = I128::from_i64(i64::MIN).wrapping_square();
        assert_eq!(res, U128::from_be_hex("40000000000000000000000000000000"));

        let x: I128 = I128::MINUS_ONE << 64;
        let res = x.wrapping_square();
        assert_eq!(res, U128::ZERO);

        let x: I128 = I128::from_i64(i64::MAX);
        let res = x.wrapping_square();
        assert_eq!(res, U128::from_be_hex("3FFFFFFFFFFFFFFF0000000000000001"));
    }

    #[test]
    fn test_saturating_square() {
        assert_eq!(
            I128::from_i64(i64::MIN).saturating_square(),
            U128::from_be_hex("40000000000000000000000000000000")
        );
        let x: I128 = I128::MINUS_ONE << 64;
        assert_eq!(x.saturating_square(), U128::MAX);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{I64, I128, Limb, U64, U128};

    fn native_product(a: u64, b: u64) -> u128 {
        if Limb::BITS == 32 {
            let (a_lo, a_hi) = (u64::from(a as u32), a >> 32);
            let (b_lo, b_hi) = (u64::from(b as u32), b >> 32);
            // Expand (a_lo + 2^32·a_hi)(b_lo + 2^32·b_hi); the full product fits u128.
            // Native 32×32 products share the implementation's multiplier circuits.
            u128::from(a_lo * b_lo)
                + ((u128::from(a_lo * b_hi) + u128::from(a_hi * b_lo)) << 32)
                + (u128::from(a_hi * b_hi) << 64)
        } else {
            u128::from(a) * u128::from(b)
        }
    }

    /// 64-bit signed multiplication agrees with `i128`, including `MIN` operands. The reference
    /// multiplies the magnitudes as `u128` (the same product the code forms) and then applies the
    /// sign, so the solver does not have to prove two multiplier circuits equivalent.
    // CaDiCaL completes the 32-bit overflow circuit that exceeds Z3's time budget.
    #[kani::proof]
    #[cfg_attr(cpubits = "32", kani::solver(cadical))]
    #[cfg_attr(not(cpubits = "32"), kani::solver(z3))]
    #[kani::unwind(5)]
    fn int_mul_i64_correct() {
        let (a, b): (i64, i64) = kani::any();
        let (ia, ib) = (I64::from_i64(a), I64::from_i64(b));

        // Check magnitudes separately so the native product shares the implementation's inputs.
        let (abs_a, abs_b) = (u64::from(ia.abs()), u64::from(ib.abs()));
        assert_eq!((abs_a, abs_b), (a.unsigned_abs(), b.unsigned_abs()));
        let magnitude = native_product(abs_a, abs_b);
        let negative = (a < 0) != (b < 0);
        let product = if negative {
            (magnitude as i128).wrapping_neg()
        } else {
            magnitude as i128
        };
        let fits = i64::try_from(product).ok();

        let wide: I128 = ia.concatenating_mul(&ib);
        assert_eq!(i128::from(wide), product);
        let (lo, hi, negate) = ia.widening_mul(&ib);
        assert_eq!(
            u128::from(u64::from(lo)) | (u128::from(u64::from(hi)) << 64),
            magnitude
        );
        assert_eq!(negate.to_bool(), negative);
        assert_eq!(
            Option::<I64>::from(ia.checked_mul(&ib)).map(i64::from),
            fits
        );
        let saturated = fits.unwrap_or(if negative { i64::MIN } else { i64::MAX });
        assert_eq!(i64::from(ia.saturating_mul(&ib)), saturated);
    }

    /// Mixed multiplication covers all `i64 × u64` inputs on 64-bit words and `i32 × u32`
    /// inputs on 32-bit words, including signed `MIN` and unsigned `MAX` in each domain.
    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(5)]
    fn int_mul_unsigned_i64_correct() {
        // shortcut: wider mixed inputs exceed 120s on 32-bit words; widen after solver improvements.
        let (a, b) = if Limb::BITS == 32 {
            let (a, b): (i32, u32) = kani::any();
            (i64::from(a), u64::from(b))
        } else {
            kani::any::<(i64, u64)>()
        };
        let (ia, ub) = (I64::from_i64(a), U64::from_u64(b));
        let abs_a = u64::from(ia.abs());
        assert_eq!(abs_a, a.unsigned_abs());
        let magnitude = native_product(abs_a, b);
        let product = if a < 0 {
            (magnitude as i128).wrapping_neg()
        } else {
            magnitude as i128
        };
        let fits = i64::try_from(product).ok();

        let wide: I128 = ia.concatenating_mul_unsigned(&ub);
        assert_eq!(i128::from(wide), product);
        let wide: I128 = ub.concatenating_mul_signed(&ia);
        assert_eq!(i128::from(wide), product);
        let (lo, hi, negate) = ia.widening_mul_unsigned(&ub);
        assert_eq!(
            (u64::from(lo), u64::from(hi)),
            (magnitude as u64, (magnitude >> 64) as u64)
        );
        assert_eq!(negate.to_bool(), a < 0);
        let (lo, hi, negate) = ub.widening_mul_signed(&ia);
        assert_eq!(
            (u64::from(lo), u64::from(hi)),
            (magnitude as u64, (magnitude >> 64) as u64)
        );
        assert_eq!(negate.to_bool(), a < 0);
        assert_eq!(
            Option::<I64>::from(ia.checked_mul_unsigned(&ub)).map(i64::from),
            fits
        );
        assert_eq!(
            Option::<I64>::from(ub.checked_mul_signed(&ia)).map(i64::from),
            fits
        );
    }

    /// Wrapping uses the operands' bits directly; a native wrapping oracle shares that circuit.
    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(5)]
    fn int_wrapping_mul_i64_correct() {
        let (a, b): (i64, i64) = kani::any();
        assert_eq!(
            i64::from(I64::from_i64(a).wrapping_mul(&I64::from_i64(b))),
            native_product(a as u64, b as u64) as i64
        );
    }

    /// Signed squares return unsigned values, so the checked/saturating bound is `u64::MAX`.
    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(5)]
    fn int_square_i64_correct() {
        let a: i64 = kani::any();
        let ia = I64::from_i64(a);
        let magnitude = u64::from(ia.abs());
        assert_eq!(magnitude, a.unsigned_abs());
        // For |a| = lo + 2^32·hi, a² = lo² + 2^33·lo·hi + 2^64·hi² < 2^128.
        // This exact native identity lets 32-bit words share the limb multipliers.
        let square = if Limb::BITS == 32 {
            let (lo, hi) = (u64::from(magnitude as u32), magnitude >> 32);
            u128::from(lo * lo) + (u128::from(lo * hi) << 33) + (u128::from(hi * hi) << 64)
        } else {
            u128::from(magnitude) * u128::from(magnitude)
        };
        let fits = u64::try_from(square).ok();

        let wide: U128 = ia.concatenating_square();
        assert_eq!(u128::from(wide), square);
        assert_eq!(u64::from(ia.wrapping_square()), square as u64);
        assert_eq!(
            Option::<U64>::from(ia.checked_square()).map(u64::from),
            fits
        );
        assert_eq!(u64::from(ia.saturating_square()), fits.unwrap_or(u64::MAX));
    }
}
