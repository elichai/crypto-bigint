use crate::{Choice, WideWord, Word};

/// Computes `lhs + rhs + carry`, returning the result along with the new carry (0, 1, or 2).
#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
pub(crate) const fn carrying_add(lhs: Word, rhs: Word, carry: Word) -> (Word, Word) {
    // We could use `Word::overflowing_add()` here analogous to `overflowing_add()`,
    // but this version seems to produce a slightly better assembly.
    let a = lhs as WideWord;
    let b = rhs as WideWord;
    let carry = carry as WideWord;
    let ret = a + b + carry;
    (ret as Word, (ret >> Word::BITS) as Word)
}

/// Computes `lhs + rhs`, returning the result along with the carry (0 or 1).
#[inline(always)]
pub(crate) const fn overflowing_add(lhs: Word, rhs: Word) -> (Word, Word) {
    let (res, carry) = lhs.overflowing_add(rhs);
    (res, carry as Word)
}

/// Computes `lhs * rhs`, returning the low and the high words of the result.
#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
pub(crate) const fn widening_mul(lhs: Word, rhs: Word) -> (Word, Word) {
    let a = lhs as WideWord;
    let b = rhs as WideWord;
    let ret = a * b;
    (ret as Word, (ret >> Word::BITS) as Word)
}

/// Computes `(lhs * rhs) + addend + carry`, returning the result along with the new carry.
#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
pub(crate) const fn carrying_mul_add(
    lhs: Word,
    rhs: Word,
    addend: Word,
    carry: Word,
) -> (Word, Word) {
    let lhs = lhs as WideWord;
    let rhs = rhs as WideWord;
    let addend = addend as WideWord;
    let carry = carry as WideWord;

    // Cannot overflow:
    // lhs      * rhs      + addend   + carry
    // (2^64-1) * (2^64-1) + (2^64-1) + (2^64-1) =
    // 2^128 - 2^65 + 1 + 2^64 - 1 + 2^64 - 1 =
    // 2^128 - 2^65 + 2*2^64 - 1 =
    // 2^128 - 1 = u128::MAX
    let ret = ((lhs * rhs) + addend) + carry;
    (ret as Word, (ret >> Word::BITS) as Word)
}

/// `const fn` equivalent of `u32::max(a, b)`.
#[inline]
pub(crate) const fn u32_max(a: u32, b: u32) -> u32 {
    Choice::from_u32_lt(a, b).select_u32(a, b)
}

/// `const` equivalent of `u32::min(a, b)`.
#[inline]
pub(crate) const fn u32_min(a: u32, b: u32) -> u32 {
    Choice::from_u32_lt(a, b).select_u32(b, a)
}

/// Remainder calculation, constant time for a given divisor `d`.
/// Based on "Faster Remainder by Direct Computation: Applications to Compilers and Software Libraries"
/// by Daniel Lemire, Owen Kaser, and Nathan Kurz., Fig. 1.
#[inline(never)]
#[allow(clippy::cast_possible_truncation, reason = "needs triage")]
#[allow(clippy::integer_division_remainder_used, reason = "needs triage")]
pub(crate) const fn u32_rem(n: u32, d: u32) -> u32 {
    assert!(d > 0, "divisor must be nonzero");
    let c = u64::MAX / (d as u64) + 1;
    (((c.wrapping_mul(n as u64) as u128) * d as u128) >> 64) as u32
}

/// Compute the number of bits needed to represent `n`.
#[inline(always)]
pub(crate) const fn u32_bits(n: u32) -> u32 {
    u32::BITS - n.leading_zeros()
}

/// Return a `Choice` representing whether `a < b`.
#[allow(clippy::cast_possible_truncation)]
#[cfg(target_pointer_width = "32")]
#[inline]
pub(crate) const fn usize_lt(a: usize, b: usize) -> Choice {
    Choice::from_u32_lt(a as u32, b as u32)
}

/// Return a `Choice` representing whether `a < b`.
#[allow(clippy::cast_possible_truncation)]
#[cfg(target_pointer_width = "64")]
#[inline]
pub(crate) const fn usize_lt(a: usize, b: usize) -> Choice {
    Choice::from_u64_lt(a as u64, b as u64)
}

cpubits::cpubits! {
    32 => {
        /// Returns the multiplicative inverse of the argument modulo 2^32.
        ///
        /// For correct results, the input `value` must be odd.
        #[must_use]
        pub(crate) const fn u32_invert_odd(value: u32) -> u32 {
            debug_assert!(value & 1 == 1, "value must be odd");
            let x = value.wrapping_mul(3) ^ 2;
            let y = 1u32.wrapping_sub(x.wrapping_mul(value));
            let (x, y) = (x.wrapping_mul(y.wrapping_add(1)), y.wrapping_mul(y));
            let (x, y) = (x.wrapping_mul(y.wrapping_add(1)), y.wrapping_mul(y));
            x.wrapping_mul(y.wrapping_add(1))
        }
    }
}

/// Returns the multiplicative inverse of the argument modulo 2^64. The implementation is based
/// on Hurchalla's method for computing the multiplicative inverse modulo a power of two, and
/// is essentially an optimized Newton iteration.
///
/// For correct results, the input `value` must be odd.
///
/// For better understanding the implementation, the following paper is recommended:
/// J. Hurchalla, "An Improved Integer Multiplicative Inverse (modulo 2^w)",
/// <https://arxiv.org/abs/2204.04342>
#[must_use]
pub(crate) const fn u64_invert_odd(value: u64) -> u64 {
    debug_assert!(value & 1 == 1, "value must be odd");
    let x = value.wrapping_mul(3) ^ 2;
    let y = 1u64.wrapping_sub(x.wrapping_mul(value));
    let (x, y) = (x.wrapping_mul(y.wrapping_add(1)), y.wrapping_mul(y));
    let (x, y) = (x.wrapping_mul(y.wrapping_add(1)), y.wrapping_mul(y));
    let (x, y) = (x.wrapping_mul(y.wrapping_add(1)), y.wrapping_mul(y));
    x.wrapping_mul(y.wrapping_add(1))
}

#[cfg(test)]
mod tests {
    use super::{u32_max, u32_min, u32_rem, usize_lt};
    use crate::Word;

    #[test]
    fn carrying_mul_add_cannot_overflow() {
        let lhs = Word::MAX;
        let rhs = Word::MAX;
        let addend = Word::MAX;
        let carry_in = Word::MAX;
        let (result, carry_out) = super::carrying_mul_add(lhs, rhs, addend, carry_in);
        assert_eq!(result, Word::MAX);
        assert_eq!(carry_out, Word::MAX);
    }

    #[test]
    fn test_u32_const_min() {
        assert_eq!(u32_min(0, 5), 0);
        assert_eq!(u32_min(7, 0), 0);
        assert_eq!(u32_min(7, 5), 5);
        assert_eq!(u32_min(7, 7), 7);
    }

    #[test]
    fn test_u32_const_max() {
        assert_eq!(u32_max(0, 5), 5);
        assert_eq!(u32_max(7, 0), 7);
        assert_eq!(u32_max(7, 5), 7);
        assert_eq!(u32_max(7, 7), 7);
    }

    #[test]
    fn test_u32_const_rem() {
        assert_eq!(u32_rem(0, 5), 0);
        assert_eq!(u32_rem(4, 5), 4);
        assert_eq!(u32_rem(7, 5), 2);
        assert_eq!(u32_rem(101, 5), 1);
    }

    #[test]
    fn test_usize_const_lt() {
        assert!(usize_lt(0, 5).to_bool());
        assert!(!usize_lt(7, 0).to_bool());
        assert!(!usize_lt(7, 5).to_bool());
        assert!(!usize_lt(7, 7).to_bool());
    }

    cpubits::cpubits! {
        32 => {
            #[test]
            fn test_u32_invert_odd() {
                use super::u32_invert_odd;

                assert_eq!(u32_invert_odd(1), 1);
                assert_eq!(u32_invert_odd(5).wrapping_mul(5), 1);
                assert_eq!(u32_invert_odd(u32::MAX).wrapping_mul(u32::MAX), 1);
            }
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use crate::word::join;

    #[kani::proof]
    fn carrying_add_correct() {
        let (lhs, rhs, carry): (Word, Word, Word) = kani::any();
        let (res, carry_out) = carrying_add(lhs, rhs, carry);
        let expected = lhs as WideWord + rhs as WideWord + carry as WideWord;
        assert_eq!(join(res, carry_out), expected);
        if carry <= 1 {
            assert!(carry_out <= 1);
        }
    }

    /// Also checks the in-code argument that the wide sum cannot overflow.
    #[kani::proof]
    fn carrying_mul_add_correct() {
        let (lhs, rhs, addend, carry): (Word, Word, Word, Word) = kani::any();
        let (lo, hi) = carrying_mul_add(lhs, rhs, addend, carry);
        let expected =
            (lhs as WideWord) * (rhs as WideWord) + addend as WideWord + carry as WideWord;
        assert_eq!(join(lo, hi), expected);
    }

    #[kani::proof]
    fn u32_min_max_correct() {
        let (a, b): (u32, u32) = kani::any();
        assert_eq!(u32_min(a, b), a.min(b));
        assert_eq!(u32_max(a, b), a.max(b));
        assert_eq!(u32_bits(a), u32::BITS - a.leading_zeros());
        let (a, b): (usize, usize) = kani::any();
        assert_eq!(usize_lt(a, b).to_bool(), a < b);
    }

    /// Lemire's remainder is exact for every `n` and every multiple-of-32 divisor up to 4096.
    /// This bounds the caller precision domain; larger precisions are not covered here.
    /// Concrete divisors let CBMC fold reciprocals, but non-power-of-two cases remain expensive.
    /// `d == 1` overflows in debug builds.
    #[kani::proof]
    #[kani::unwind(130)]
    fn u32_rem_correct_for_bit_precisions() {
        let n: u32 = kani::any();
        let mut d = 32;
        while d <= 4096 {
            assert_eq!(u32_rem(n, d), n % d);
            d += 32;
        }
    }

    /// All dividends at power-of-two precisions from 32 to 4096 bits.
    #[kani::proof]
    #[kani::unwind(10)]
    fn u32_rem_power_of_two_precisions_correct() {
        let n: u32 = kani::any();
        for shift in 5..=12 {
            let d = 1 << shift;
            assert_eq!(u32_rem(n, d), n & (d - 1));
        }
    }

    /// All odd inputs, checking the low 16 bits of the inverse rather than a reduced input set.
    #[kani::proof]
    fn u64_invert_odd_low16_correct() {
        let value = kani::any::<u64>() | 1;
        let inverse = u64_invert_odd(value);
        assert_eq!((value as u16).wrapping_mul(inverse as u16), 1);
    }

    cpubits::cpubits! {
        32 => {
            /// All odd inputs to the separate 32-bit implementation, modulo 2^16.
            #[kani::proof]
            fn u32_invert_odd_low16_correct() {
                let value = kani::any::<u32>() | 1;
                assert_eq!((value as u16).wrapping_mul(u32_invert_odd(value) as u16), 1);
            }
        }
    }
}
