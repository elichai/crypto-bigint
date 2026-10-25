//! Implementation of Bernstein-Yang modular inversion and GCD algorithm (a.k.a. safegcd)
//! as described in: <https://eprint.iacr.org/2019/266>.
//!
//! Adapted from the Apache 2.0+MIT licensed implementation originally from:
//! <https://github.com/taikoxyz/halo2curves/pull/2>
//! <https://github.com/privacy-scaling-explorations/halo2curves/pull/83>
//!
//! Copyright (c) 2023 Privacy Scaling Explorations Team

// TODO(tarcieri): optimized implementation for 32-bit platforms (#380)

#[cfg(feature = "alloc")]
pub(crate) mod boxed;

use crate::{Choice, CtOption, I64, Int, Limb, Odd, U64, Uint, bitlen, primitives::u32_min};
use core::fmt;

const GCD_BATCH_SIZE: u32 = 62;

/// Modular multiplicative inverter based on the Bernstein-Yang method.
///
/// The inverter can be created for a specified modulus M and adjusting parameter A to compute
/// the adjusted multiplicative inverses of positive integers, i.e. for computing
/// (1 / x) * A (mod M) for a positive integer x.
///
/// The adjusting parameter allows computing the multiplicative inverses in the case of using the
/// Montgomery representation for the input or the expected output. If R is the Montgomery
/// factor, the multiplicative inverses in the appropriate representation can be computed
/// provided that the value of A is chosen as follows:
/// - A = 1, if both the input and the expected output are in the standard form
/// - A = R^2 mod M, if both the input and the expected output are in the Montgomery form
/// - A = R mod M, if either the input or the expected output is in the Montgomery form,
///   but not both of them
///
/// The public methods of this type receive and return unsigned big integers as arrays of 64-bit
/// chunks, the ordering of which is little-endian. Both the modulus and the integer to be
/// inverted should not exceed 2 ^ (62 * L - 64).
///
/// For better understanding the implementation, the following resources are recommended:
/// - D. Bernstein, B.-Y. Yang, "Fast constant-time gcd computation and modular inversion",
///   <https://gcd.cr.yp.to/safegcd-20190413.pdf>
/// - P. Wuille, "The safegcd implementation in libsecp256k1 explained",
///   <https://github.com/bitcoin-core/secp256k1/blob/master/doc/safegcd_implementation.md>
#[derive(Clone, Debug)]
pub(crate) struct SafeGcdInverter<const LIMBS: usize> {
    /// Modulus
    pub(super) modulus: Odd<Uint<LIMBS>>,

    /// Multiplicative inverse of the modulus modulo 2^62
    inverse: u64,

    /// Adjusting parameter (see toplevel documentation).
    adjuster: Uint<LIMBS>,
}

/// Type of the Bernstein-Yang transition matrix multiplied by 2^62
type Matrix = [[i64; 2]; 2];

impl<const LIMBS: usize> SafeGcdInverter<LIMBS> {
    /// Creates the inverter for specified modulus and adjusting parameter.
    #[cfg(test)]
    pub(crate) const fn new(modulus: &Odd<Uint<LIMBS>>, adjuster: &Uint<LIMBS>) -> Self {
        Self::new_with_inverse(
            modulus,
            U64::from_u64(modulus.as_uint_ref().invert_mod_u64()),
            adjuster,
        )
    }

    #[inline]
    pub(crate) const fn new_with_inverse(
        modulus: &Odd<Uint<LIMBS>>,
        inverse: U64,
        adjuster: &Uint<LIMBS>,
    ) -> Self {
        Self {
            modulus: *modulus,
            inverse: inverse.as_uint_ref().lowest_u64(),
            adjuster: *adjuster,
        }
    }

    /// Returns either the adjusted modular multiplicative inverse for the argument or `None`
    /// depending on invertibility of the argument, i.e. its coprimality with the modulus.
    pub const fn invert(&self, value: &Uint<LIMBS>) -> CtOption<Uint<LIMBS>> {
        invert_odd_mod_precomp::<LIMBS, false>(value, &self.modulus, self.inverse, &self.adjuster)
    }

    /// Returns either the adjusted modular multiplicative inverse for the argument or `None`
    /// depending on invertibility of the argument, i.e. its coprimality with the modulus.
    ///
    /// This version is variable-time with respect to `value`.
    pub const fn invert_vartime(&self, value: &Uint<LIMBS>) -> CtOption<Uint<LIMBS>> {
        invert_odd_mod_precomp::<LIMBS, true>(value, &self.modulus, self.inverse, &self.adjuster)
    }
}

#[inline]
pub const fn invert_odd_mod<const LIMBS: usize, const VARTIME: bool>(
    a: &Uint<LIMBS>,
    m: &Odd<Uint<LIMBS>>,
) -> CtOption<Uint<LIMBS>> {
    let mi = m.as_uint_ref().invert_mod_u64();
    invert_odd_mod_precomp::<LIMBS, VARTIME>(a, m, mi, &Uint::ONE)
}

/// Calculate the multiplicative inverse of `a` modulo `m`.
const fn invert_odd_mod_precomp<const LIMBS: usize, const VARTIME: bool>(
    a: &Uint<LIMBS>,
    m: &Odd<Uint<LIMBS>>,
    mi: u64,
    e: &Uint<LIMBS>,
) -> CtOption<Uint<LIMBS>> {
    let a_nonzero = a.is_nonzero();
    let (mut f, mut g) = (SignedInt::from_uint(*m.as_ref()), SignedInt::from_uint(*a));
    let (mut d, mut e) = (SignedInt::<LIMBS>::ZERO, SignedInt::from_uint(*e));
    let mut steps = iterations(Uint::<LIMBS>::BITS);
    let mut delta = 1;
    let mut t;

    while steps > 0 {
        if VARTIME && g.is_zero_vartime() {
            break;
        }
        let batch = u32_min(steps, GCD_BATCH_SIZE);
        (delta, t) = jump::<VARTIME>(f.lowest(), g.lowest(), delta, batch);
        (f, g) = update_fg(&f, &g, t, batch);
        (d, e) = update_de(&d, &e, m.as_ref(), mi, t, batch);
        steps -= batch;
    }

    let d = d.norm(f.is_negative(), m.as_ref());
    CtOption::new(d, Uint::eq(&f.magnitude, &Uint::ONE).and(a_nonzero))
}

/// Calculate the greatest common denominator of odd `f`, and `g`.
pub const fn gcd_odd<const LIMBS: usize, const VARTIME: bool>(
    f: &Odd<Uint<LIMBS>>,
    g: &Uint<LIMBS>,
) -> Odd<Uint<LIMBS>> {
    let (mut f, mut g) = (SignedInt::from_uint(*f.as_ref()), SignedInt::from_uint(*g));
    let mut steps = iterations(Uint::<LIMBS>::BITS);
    let mut delta = 1;
    let mut t;

    while steps > 0 {
        if VARTIME && g.is_zero_vartime() {
            break;
        }
        let batch = u32_min(steps, GCD_BATCH_SIZE);
        (delta, t) = jump::<VARTIME>(f.lowest(), g.lowest(), delta, batch);
        (f, g) = update_fg(&f, &g, t, batch);
        steps -= batch;
    }

    f.magnitude().to_odd().expect_copied("odd by construction")
}

/// Perform `batch` steps of the gcd reduction process on signed tail values `f` and `g`.
#[inline]
const fn jump<const VARTIME: bool>(
    mut f: i64,
    mut g: i64,
    mut delta: i64,
    mut batch: u32,
) -> (i64, Matrix) {
    debug_assert!(f & 1 == 1, "f must be odd");
    let mut t = [[1i64, 0], [0, 1]];
    while batch > 0 {
        (f, g, delta, t) = if VARTIME {
            jump_step_vartime(f, g, delta, t)
        } else {
            jump_step(f, g, delta, t)
        };
        batch -= 1;
    }
    (delta, t)
}

/// Perform one step of the gcd reduction in constant time.
/// This follows the half-delta variant of safegcd-bounds which reduces the round count.
/// <https://github.com/sipa/safegcd-bounds>
#[inline(always)]
#[allow(clippy::cast_sign_loss)]
const fn jump_step(
    mut f: i64,
    mut g: i64,
    mut delta: i64,
    mut t: Matrix,
) -> (i64, i64, i64, Matrix) {
    let d_gtz = Choice::from_u64_nz((delta & !(delta >> 63)) as u64);
    let g_odd = Choice::from_u64_lsb((g & 1) as u64);
    let g_adj = g_odd.select_i64(0, f);
    let swap = d_gtz.and(g_odd);
    delta = swap.select_i64(2i64.wrapping_add(delta), 2i64.wrapping_sub(delta));
    f = swap.select_i64(f, g);
    g = swap.select_i64(g.wrapping_add(g_adj), g.wrapping_sub(g_adj)) >> 1;
    t = [
        [
            swap.select_i64(t[0][0], t[1][0]) << 1,
            swap.select_i64(t[0][1], t[1][1]) << 1,
        ],
        [
            t[1][0].wrapping_add(g_odd.select_i64(0, d_gtz.select_i64(t[0][0], -t[0][0]))),
            t[1][1].wrapping_add(g_odd.select_i64(0, d_gtz.select_i64(t[0][1], -t[0][1]))),
        ],
    ];
    (f, g, delta, t)
}

/// Perform one step of the gcd reduction in variable time.
#[inline(always)]
const fn jump_step_vartime(
    mut f: i64,
    mut g: i64,
    mut delta: i64,
    mut t: Matrix,
) -> (i64, i64, i64, Matrix) {
    if (g & 1) != 0 {
        (f, g, delta, t) = if delta > 0 {
            (
                g,
                g.wrapping_sub(f),
                2i64.wrapping_sub(delta),
                [
                    t[1],
                    [t[1][0].wrapping_sub(t[0][0]), t[1][1].wrapping_sub(t[0][1])],
                ],
            )
        } else {
            (
                f,
                g.wrapping_add(f),
                2i64.wrapping_add(delta),
                [
                    t[0],
                    [t[1][0].wrapping_add(t[0][0]), t[1][1].wrapping_add(t[0][1])],
                ],
            )
        };
    } else {
        delta = 2i64.wrapping_add(delta);
    }
    g >>= 1;
    t[0][0] <<= 1;
    t[0][1] <<= 1;
    (f, g, delta, t)
}

#[inline]
const fn update_fg<const LIMBS: usize>(
    a: &SignedInt<LIMBS>,
    b: &SignedInt<LIMBS>,
    t: Matrix,
    shift: u32,
) -> (SignedInt<LIMBS>, SignedInt<LIMBS>) {
    (
        SignedInt::lincomb_int_reduce_shift(
            a,
            b,
            &I64::from_i64(t[0][0]),
            &I64::from_i64(t[0][1]),
            shift,
        ),
        SignedInt::lincomb_int_reduce_shift(
            a,
            b,
            &I64::from_i64(t[1][0]),
            &I64::from_i64(t[1][1]),
            shift,
        ),
    )
}

#[inline]
const fn update_de<const LIMBS: usize>(
    d: &SignedInt<LIMBS>,
    e: &SignedInt<LIMBS>,
    m: &Uint<LIMBS>,
    mi: u64,
    t: Matrix,
    shift: u32,
) -> (SignedInt<LIMBS>, SignedInt<LIMBS>) {
    (
        SignedInt::lincomb_int_reduce_shift_mod(
            d,
            e,
            &Int::from_i64(t[0][0]),
            &Int::from_i64(t[0][1]),
            shift,
            m,
            U64::from_u64(mi),
        ),
        SignedInt::lincomb_int_reduce_shift_mod(
            d,
            e,
            &Int::from_i64(t[1][0]),
            &Int::from_i64(t[1][1]),
            shift,
            m,
            U64::from_u64(mi),
        ),
    )
}

/// Conditionally negate a wide Uint represented by `(lo, hi)`.
#[inline]
const fn conditional_negate_in_place_wide<const L: usize, const H: usize>(
    lo: &mut Uint<L>,
    hi: &mut Uint<H>,
    flag: Choice,
) {
    let (neg, carry) = lo.carrying_neg();
    let hi_neg = hi
        .not()
        .wrapping_add(&Uint::select(&Uint::ZERO, &Uint::ONE, carry));
    *lo = Uint::select(lo, &neg, flag);
    *hi = Uint::select(hi, &hi_neg, flag);
}

/// Right shift a wide Uint represented by `(lo, hi)` returning any remaining high bits.
#[inline]
const fn shr_in_place_wide<const L: usize, const H: usize>(
    lo: &mut Uint<L>,
    hi: &mut Uint<H>,
    shift: u32,
) {
    debug_assert!(H <= L);
    debug_assert!(shift < Uint::<H>::BITS);
    let copy = hi.shl_vartime(Uint::<H>::BITS - shift);
    *hi = hi.shr_vartime(shift);
    *lo = lo.shr_vartime(shift);
    let mut offs = bitlen::to_limbs(shift);
    lo.limbs[L - offs] = lo.limbs[L - offs].bitor(copy.limbs[H - offs]);
    loop {
        offs -= 1;
        if offs == 0 {
            break;
        }
        lo.limbs[L - offs] = copy.limbs[H - offs];
    }
}

/// Calculate the maximum number of iterations required according to
/// safegcd-bounds: <https://github.com/sipa/safegcd-bounds>
///
/// Computed in `u64` because `45907 * bits + 30179` exceeds `u32::MAX` once `bits >= 93558`.
/// The count itself (about `2.3 * bits`) only exceeds `u32::MAX` for `bits` above `1864517463`,
/// which panics rather than wrapping.
// NOTE: the division is non-constant-time, but this is used to compute the number of iterations we
// perform which is leaked in timing information
#[inline]
#[allow(clippy::integer_division_remainder_used, reason = "public parameter")]
#[allow(clippy::cast_lossless, reason = "`const fn`")]
#[allow(clippy::cast_possible_truncation, reason = "checked by the assertion")]
const fn iterations(bits: u32) -> u32 {
    let iterations = (45907 * bits as u64 + 30179) / 19929;
    assert!(
        iterations <= u32::MAX as u64,
        "precision too large for safegcd"
    );
    iterations as u32
}

/// A `Uint` which carries a separate sign in order to maintain the same range.
#[derive(Clone, Copy)]
struct SignedInt<const LIMBS: usize> {
    sign: Choice,
    magnitude: Uint<LIMBS>,
}

impl<const LIMBS: usize> SignedInt<LIMBS> {
    pub const ZERO: Self = Self::from_uint(Uint::ZERO);

    /// Construct a new `SignedInt` from a `Uint`.
    pub const fn from_uint(uint: Uint<LIMBS>) -> Self {
        Self {
            sign: Choice::FALSE,
            magnitude: uint,
        }
    }

    /// Construct a new `SignedInt` from a `Uint` and a sign flag.
    pub const fn from_uint_sign(magnitude: Uint<LIMBS>, sign: Choice) -> Self {
        Self { sign, magnitude }
    }

    /// Obtain the magnitude of the `SignedInt`, ie. its absolute value.
    pub const fn magnitude(&self) -> Uint<LIMBS> {
        self.magnitude
    }

    /// Determine if the `SignedInt` is non-zero.
    pub const fn is_nonzero(&self) -> Choice {
        self.magnitude.is_nonzero()
    }

    /// Determine if the `SignedInt` is zero in variable time.
    pub const fn is_zero_vartime(&self) -> bool {
        self.magnitude.is_zero_vartime()
    }

    /// Determine if the `SignedInt` is negative.
    /// Note: `-0` is representable in this type, so it may be necessary
    /// to check `self.is_nonzero()` as well.
    pub const fn is_negative(&self) -> Choice {
        self.sign
    }

    /// Extract the lowest 63 bits and convert to its signed representation.
    #[allow(clippy::cast_possible_wrap)]
    pub const fn lowest(&self) -> i64 {
        let mag = (self.magnitude.as_uint_ref().lowest_u64() & (u64::MAX >> 1)) as i64;
        self.sign.select_i64(mag, mag.wrapping_neg())
    }

    /// Compute the linear combination `a•b + c•d`, returning `(lo, hi, sign)`.
    #[inline]
    pub(crate) const fn lincomb_int<const RHS: usize>(
        a: &SignedInt<LIMBS>,
        b: &SignedInt<LIMBS>,
        c: &Int<RHS>,
        d: &Int<RHS>,
    ) -> (Uint<LIMBS>, Uint<RHS>, Choice) {
        let (c, c_sign) = c.abs_sign();
        let (d, d_sign) = d.abs_sign();
        // Each SignedInt • abs(Int) product leaves an empty upper bit.
        let (mut x, mut x_hi) = a.magnitude.widening_mul(&c);
        let x_neg = a.sign.xor(c_sign);
        let (mut y, mut y_hi) = b.magnitude.widening_mul(&d);
        let y_neg = b.sign.xor(d_sign);
        let odd_neg = x_neg.xor(y_neg);

        // Negate y if none or both of the multiplication results are negative.
        conditional_negate_in_place_wide(&mut y, &mut y_hi, odd_neg.not());

        let mut borrow;
        (x, borrow) = x.borrowing_sub(&y, Limb::ZERO);
        (x_hi, borrow) = x_hi.borrowing_sub(&y_hi, borrow);
        let swap = borrow.is_nonzero().and(odd_neg);

        // Negate the result if we did not negate y and there was a borrow,
        // indicating that |y| > |x|.
        conditional_negate_in_place_wide(&mut x, &mut x_hi, swap);

        let sign = x_neg.and(swap.not()).or(y_neg.and(swap));
        (x, x_hi, sign)
    }

    /// Compute the linear combination `a•b + c•d`, and shift the result
    /// `shift` bits to the right, returning a signed value in the same range
    /// as the `SignedInt` inputs.
    pub(crate) const fn lincomb_int_reduce_shift<const S: usize>(
        a: &Self,
        b: &Self,
        c: &Int<S>,
        d: &Int<S>,
        shift: u32,
    ) -> Self {
        debug_assert!(shift < Uint::<S>::BITS);
        let (mut a, mut a_hi, a_sign) = Self::lincomb_int(a, b, c, d);
        shr_in_place_wide(&mut a, &mut a_hi, shift);
        SignedInt::from_uint_sign(a, a_sign)
    }

    /// Compute the linear combination `a•b + c•d`, and shift the result
    /// `shift` bits to the right modulo `m`, returning a signed value in the
    /// same range as the `SignedInt` inputs.
    pub(crate) const fn lincomb_int_reduce_shift_mod<const S: usize>(
        a: &Self,
        b: &Self,
        c: &Int<S>,
        d: &Int<S>,
        shift: u32,
        m: &Uint<LIMBS>,
        mi: Uint<S>,
    ) -> SignedInt<LIMBS> {
        debug_assert!(shift < Uint::<S>::BITS);
        let (mut x, mut x_hi, mut x_sign) = SignedInt::lincomb_int(a, b, c, d);

        // Compute the multiple of m that will clear the low N bits of (x, x_hi).
        let mut mf = x.resize::<S>().wrapping_mul(&mi);
        mf = mf.bitand(&Uint::MAX.shr_vartime(Uint::<S>::BITS - shift));
        let (xa, xa_hi) = m.widening_mul(&mf);

        // Subtract the adjustment from (x, x_hi) potentially producing a borrow.
        let mut borrow;
        (x, borrow) = x.borrowing_sub(&xa, Limb::ZERO);
        (x_hi, borrow) = x_hi.borrowing_sub(&xa_hi, borrow);

        // Negate (x, x_hi) if the subtraction borrowed.
        let swap = borrow.is_nonzero();
        conditional_negate_in_place_wide(&mut x, &mut x_hi, swap);
        x_sign = x_sign.xor(swap);

        // Shift the result, eliminating the trailing zeros.
        shr_in_place_wide(&mut x, &mut x_hi, shift);
        debug_assert!(
            x_hi.shr1().is_nonzero().not().to_bool(),
            "overflow was larger than one bit"
        );

        // The magnitude x is now in the range [0, 2m). We conditionally subtract
        // m in order to keep the output in (-m, m).
        x = x.try_sub_with_carry(x_hi.limbs[0], m).0;

        SignedInt::from_uint_sign(x, x_sign)
    }

    /// Normalize the value to a `Uint` in the range `[0, m)`.
    const fn norm(&self, f_sign: Choice, m: &Uint<LIMBS>) -> Uint<LIMBS> {
        let swap = f_sign.xor(self.sign).and(self.is_nonzero());
        Uint::select(&self.magnitude, &m.wrapping_sub(&self.magnitude), swap)
    }

    /// Compare two `SignedInt` in constant time.
    pub const fn eq(a: &Self, b: &Self) -> Choice {
        Uint::eq(&a.magnitude, &b.magnitude).and(a.sign.eq(b.sign).or(a.is_nonzero().not()))
    }
}

impl<const LIMBS: usize> fmt::Debug for SignedInt<LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}0x{}",
            if self.sign.to_bool() { "-" } else { "+" },
            self.magnitude
        ))
    }
}

impl<const LIMBS: usize> PartialEq for SignedInt<LIMBS> {
    fn eq(&self, other: &Self) -> bool {
        Self::eq(self, other).to_bool()
    }
}

#[cfg(test)]
mod tests {
    use super::{SafeGcdInverter, iterations};
    use crate::{U128, U256, modular::safegcd::shr_in_place_wide};

    #[test]
    fn invert() {
        let g =
            U256::from_be_hex("00000000CBF9350842F498CE441FC2DC23C7BF47D3DE91C327B2157C5E4EED77");
        let modulus =
            U256::from_be_hex("FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551")
                .to_odd()
                .unwrap();
        let inverter = SafeGcdInverter::new(&modulus, &U256::ONE);
        let result = inverter.invert(&g).unwrap();
        assert_eq!(
            U256::from_be_hex("FB668F8F509790BC549B077098918604283D42901C92981062EB48BC723F617B"),
            result
        );
    }

    #[test]
    fn shr_wide() {
        let hi = U128::from_u128(0x11111111222222223333333344444444);
        let lo = U256::MAX;
        let (mut a, mut a_hi) = (lo, hi);
        shr_in_place_wide(&mut a, &mut a_hi, 16);
        assert_eq!(
            a,
            U256::from_be_hex("4444FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
        );
        assert_eq!(a_hi, U128::from_u128(0x1111111122222222333333334444));
        let (mut b, mut b_hi) = (lo, hi);
        shr_in_place_wide(&mut b, &mut b_hi, 68);
        assert_eq!(
            b,
            U256::from_be_hex("23333333344444444FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
        );
        assert_eq!(b_hi, U128::from_u128(0x111111112222222));
    }

    #[test]
    fn iterations_do_not_overflow() {
        // Values from the `safegcd-bounds` formula evaluated in arbitrary precision.
        assert_eq!(iterations(256), 591);
        assert_eq!(iterations(93_504), 215_390);
        // `45907 * bits + 30179` first exceeds `u32::MAX` here.
        assert_eq!(iterations(93_558), 215_514);
        assert_eq!(iterations(93_568), 215_537);
        assert_eq!(iterations(98_304), 226_447);
        // The largest precision whose count still fits in `u32`.
        assert_eq!(iterations(1_864_517_463), 4_294_967_294);
    }

    #[test]
    #[should_panic(expected = "precision too large for safegcd")]
    fn iterations_panics_beyond_u32() {
        iterations(1_864_517_464);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::{
        Matrix, SignedInt, conditional_negate_in_place_wide, jump_step, jump_step_vartime,
        shr_in_place_wide,
    };
    use crate::{Choice, I64, Limb, U64, U128};

    fn native_product(a: u64, b: u64) -> u128 {
        if Limb::BITS == 32 {
            let (a_lo, a_hi) = (u64::from(a as u32), a >> 32);
            let (b_lo, b_hi) = (u64::from(b as u32), b >> 32);
            // Exact a*b; native radix-2^32 carry order shares the implementation's solver circuits.
            let p00 = a_lo * b_lo;
            let p01 = a_lo * b_hi + (p00 >> 32);
            let p10 = a_hi * b_lo + u64::from(p01 as u32);
            let p11 = a_hi * b_hi + (p01 >> 32) + (p10 >> 32);
            u128::from(p00 as u32) | (u128::from(p10 as u32) << 32) | (u128::from(p11) << 64)
        } else {
            u128::from(a) * u128::from(b)
        }
    }

    /// The signed tail uses only the low 63 magnitude bits, including for negative zero.
    #[kani::proof]
    #[kani::unwind(5)]
    fn safegcd_signed_lowest_correct() {
        let (magnitude, negative): (u128, bool) = kani::any();
        let signed =
            SignedInt::from_uint_sign(U128::from_u128(magnitude), u8::from(negative).into());
        let tail = (magnitude & ((1 << 63) - 1)) as i64;
        assert_eq!(signed.lowest(), if negative { -tail } else { tail });
        assert_eq!(signed.is_nonzero().to_bool(), magnitude != 0);
        assert_eq!(signed.is_zero_vartime(), magnitude == 0);
    }

    /// Normalization applies the product of both signs modulo `m` for a reduced magnitude.
    #[kani::proof]
    #[kani::unwind(5)]
    fn safegcd_signed_norm_correct() {
        let (magnitude, modulus, negative, f_negative): (u128, u128, bool, bool) = kani::any();
        kani::assume(magnitude < modulus);
        let signed =
            SignedInt::from_uint_sign(U128::from_u128(magnitude), u8::from(negative).into());
        let result =
            u128::from(signed.norm(u8::from(f_negative).into(), &U128::from_u128(modulus)));
        let expected = if magnitude != 0 && negative != f_negative {
            modulus - magnitude
        } else {
            magnitude
        };
        assert_eq!(result, expected);
        assert!(result < modulus);
    }

    /// Full 64-bit magnitudes and signed coefficients, including `i64::MIN` and cancellation.
    /// Native sign/magnitude arithmetic avoids overflowing `i128` when both products are large.
    // CaDiCaL32 and Z3/64 fit 120s; the earlier Z3/32 oracle and CaDiCaL64 did not.
    #[kani::proof]
    #[cfg_attr(cpubits = "32", kani::solver(cadical))]
    #[cfg_attr(not(cpubits = "32"), kani::solver(z3))]
    #[kani::unwind(5)]
    fn safegcd_signed_lincomb_u64_correct() {
        let (a, b, c, d, a_negative, b_negative): (u64, u64, i64, i64, bool, bool) = kani::any();
        let sa = SignedInt::from_uint_sign(U64::from_u64(a), u8::from(a_negative).into());
        let sb = SignedInt::from_uint_sign(U64::from_u64(b), u8::from(b_negative).into());
        let (c_int, d_int) = (I64::from_i64(c), I64::from_i64(d));
        let (c_magnitude, d_magnitude) = (u64::from(c_int.abs()), u64::from(d_int.abs()));
        // Check multiplier inputs independently, then share their circuits with the implementation.
        assert_eq!(c_magnitude, c.unsigned_abs());
        assert_eq!(d_magnitude, d.unsigned_abs());
        let (lo, hi, negative) = SignedInt::lincomb_int(&sa, &sb, &c_int, &d_int);
        let x = native_product(a, c_magnitude);
        let y = native_product(b, d_magnitude);
        let x_negative = a_negative ^ (c < 0);
        let y_negative = b_negative ^ (d < 0);
        let (magnitude, expected_negative) = if x_negative == y_negative {
            (x + y, x_negative)
        } else if x >= y {
            (x - y, x_negative)
        } else {
            (y - x, y_negative)
        };
        assert_eq!(
            (u128::from(u64::from(hi)) << 64) | u128::from(u64::from(lo)),
            magnitude
        );
        // The representation permits either sign for zero.
        if magnitude != 0 {
            assert_eq!(negative.to_bool(), expected_negative);
        }
    }

    /// The upstream widened iteration calculation returns the exact floor without truncation.
    // Z3 proves this specification in under a second; CaDiCaL exceeds 120 seconds.
    #[kani::proof]
    #[kani::solver(z3)]
    fn safegcd_iterations_correct() {
        let bits: u32 = kani::any();
        let expected = (45907 * u64::from(bits) + 30179) / 19929;
        kani::assume(expected <= u64::from(u32::MAX));
        assert_eq!(u64::from(super::iterations(bits)), expected);
    }

    /// One constant-time divstep agrees with the vartime one, keeps `f` odd, and only halves even
    /// values of `g`. `t[0]` entries of `i64::MIN` are excluded: the constant-time step negates
    /// them (`jump_step_row_bounds` shows they cannot occur).
    #[kani::proof]
    fn jump_step_ct_matches_vartime() {
        let (f, g, delta): (i64, i64, i64) = kani::any();
        let t: Matrix = kani::any();
        kani::assume(f & 1 == 1);
        kani::assume(t[0][0] != i64::MIN && t[0][1] != i64::MIN);

        let ct = jump_step(f, g, delta, t);
        assert_eq!(ct, jump_step_vartime(f, g, delta, t));

        let (f2, g2, _, _) = ct;
        assert_eq!(f2 & 1, 1);
        // The halved value is exact: `2·g2` is `g`, `g + f` or `g - f` (all even when used).
        let unhalved = if g & 1 == 0 {
            g
        } else if delta > 0 {
            g.wrapping_sub(f)
        } else {
            g.wrapping_add(f)
        };
        assert_eq!(unhalved & 1, 0);
        assert_eq!(unhalved >> 1, g2);
    }

    /// A divstep consumes one low bit: congruence modulo `2^k` becomes congruence modulo
    /// `2^(k-1)`, with identical delta and matrix updates. Induction gives batch tail dependence.
    #[kani::proof]
    fn jump_step_low_bits_dependence() {
        let (f, g, f_high, g_high, delta, k): (i64, i64, u64, u64, i64, u32) = kani::any();
        let t: Matrix = kani::any();
        kani::assume(f & 1 == 1);
        kani::assume(1 <= k && k <= 63);
        kani::assume(t[0][0] != i64::MIN && t[0][1] != i64::MIN);
        let mask = (1u64 << k) - 1;
        let same_f_tail = ((f as u64 & mask) | (f_high & !mask)) as i64;
        let same_g_tail = ((g as u64 & mask) | (g_high & !mask)) as i64;
        let (f1, g1, delta1, t1) = jump_step(f, g, delta, t);
        let (f2, g2, delta2, t2) = jump_step(same_f_tail, same_g_tail, delta, t);
        assert_eq!((delta1, t1), (delta2, t2));
        assert_eq!(f1 as u64 & (mask >> 1), f2 as u64 & (mask >> 1));
        assert_eq!(g1 as u64 & (mask >> 1), g2 as u64 & (mask >> 1));
    }

    fn row_norm(row: [i64; 2]) -> u128 {
        u128::from(row[0].unsigned_abs()) + u128::from(row[1].unsigned_abs())
    }

    /// If both rows of `t` have `|t[i][0]| + |t[i][1]| <= 2^k`, one divstep keeps both rows below
    /// `2^(k+1)` without overflow. From the identity matrix, by induction, every matrix produced by
    /// a batch of at most 62 steps has rows bounded by `2^62`: no entry is `i64::MIN`, and the
    /// matrix arithmetic in `jump_step` never overflows. Linear combinations need separate bounds.
    #[kani::proof]
    fn jump_step_row_bounds() {
        let (f, g, delta, k): (i64, i64, i64, u32) = kani::any();
        let t: Matrix = kani::any();
        kani::assume(f & 1 == 1);
        kani::assume(k <= 61);
        kani::assume(row_norm(t[0]) <= 1 << k && row_norm(t[1]) <= 1 << k);

        let (_, _, _, t2) = jump_step(f, g, delta, t);
        assert!(row_norm(t2[0]) <= 1 << (k + 1));
        assert!(row_norm(t2[1]) <= 1 << (k + 1));
    }

    /// Shifting the wide value `(lo, hi)` right by `0 < shift < 64` (the batch sizes used) agrees
    /// with native arithmetic, for a 64- and a 128-bit `lo` under a 64-bit `hi`. These are the
    /// `(L, H)` shapes `(1, 1)`, `(2, 1)` with 64-bit words and `(2, 2)`, `(4, 2)` with 32-bit
    /// words. With 32-bit words, `Uint<1>` gives `L = 1 < H = 2`, which panics.
    #[kani::proof]
    #[kani::unwind(5)]
    fn shr_in_place_wide_correct() {
        let (lo, hi, shift): (u128, u64, u32) = kani::any();
        kani::assume(0 < shift && shift < 64);

        let (mut ulo, mut uhi) = (U64::from_u64(lo as u64), U64::from_u64(hi));
        shr_in_place_wide(&mut ulo, &mut uhi, shift);
        let wide = (u128::from(hi) << 64) | u128::from(lo as u64);
        assert_eq!(u64::from(ulo), (wide >> shift) as u64);
        assert_eq!(u64::from(uhi), hi >> shift);

        let (mut ulo, mut uhi) = (U128::from_u128(lo), U64::from_u64(hi));
        shr_in_place_wide(&mut ulo, &mut uhi, shift);
        let expected_lo = (lo >> shift) | (u128::from(hi) << (128 - shift));
        assert_eq!(u128::from(ulo), expected_lo);
        assert_eq!(u64::from(uhi), hi >> shift);
    }

    /// Conditionally negating the wide value `(lo, hi)` is two's complement negation of the
    /// concatenation, for 64- and 128-bit `lo` under a 64-bit `hi`.
    #[kani::proof]
    #[kani::unwind(5)]
    fn conditional_negate_in_place_wide_correct() {
        let (lo, hi, flag): (u128, u64, bool) = kani::any();
        let choice = if flag { Choice::TRUE } else { Choice::FALSE };

        let (mut ulo, mut uhi) = (U64::from_u64(lo as u64), U64::from_u64(hi));
        conditional_negate_in_place_wide(&mut ulo, &mut uhi, choice);
        let wide = (u128::from(hi) << 64) | u128::from(lo as u64);
        let expected = if flag { wide.wrapping_neg() } else { wide };
        assert_eq!(
            (u64::from(ulo), u64::from(uhi)),
            (expected as u64, (expected >> 64) as u64)
        );

        let (mut ulo, mut uhi) = (U128::from_u128(lo), U64::from_u64(hi));
        conditional_negate_in_place_wide(&mut ulo, &mut uhi, choice);
        // 192-bit negation: the low part is `-lo`, the high part `!hi + (lo == 0)`.
        let (expected_lo, expected_hi) = if flag {
            (lo.wrapping_neg(), (!hi).wrapping_add(u64::from(lo == 0)))
        } else {
            (lo, hi)
        };
        assert_eq!(
            (u128::from(ulo), u64::from(uhi)),
            (expected_lo, expected_hi)
        );
    }
}
