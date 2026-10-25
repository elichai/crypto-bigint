//! Bit calculations.
//!
//! These provide a single place to handle bits-to-bytes conversions and the `u32`/`usize`
//! discrepancy that can only be solved by panics or casting.

use crate::Limb;

/// Calculate a `u32` representing a number of bits from the given number of bytes as a `usize`.
#[cfg_attr(not(feature = "alloc"), allow(dead_code))]
#[inline(always)]
pub(crate) const fn from_bytes(nbytes: usize) -> u32 {
    mul(nbytes, 8)
}

/// Calculate a `u32` representing a number of bits in an integer with the given number of limbs.
#[inline(always)]
pub(crate) const fn from_limbs(nlimbs: usize) -> u32 {
    mul(nlimbs, Limb::BITS)
}

/// Calculate a `usize` representing the number of bytes required to store the given number of bits
/// represented as a `u32` (i.e. rounding up).
#[cfg_attr(not(feature = "alloc"), allow(dead_code))]
#[inline(always)]
pub(crate) const fn to_bytes(nbits: u32) -> usize {
    u32_to_usize(div_ceil_pow2(nbits, 3))
}

/// Calculate a `usize` representing the number of limbs required to store the given number of bits
/// represented as a `u32` (i.e. rounding up).
#[inline(always)]
pub(crate) const fn to_limbs(nbits: u32) -> usize {
    u32_to_usize(to_limbs_u32(nbits))
}

/// Calculate the number of limbs required to store the given number of bits as a `u32`.
#[inline(always)]
pub(crate) const fn to_limbs_u32(nbits: u32) -> u32 {
    div_ceil_pow2(nbits, Limb::LOG2_BITS)
}

/// Divide by a power of two, rounding up, without a remainder-dependent branch.
#[inline(always)]
const fn div_ceil_pow2(n: u32, log2: u32) -> u32 {
    let mask = (1 << log2) - 1;
    (n >> log2).wrapping_add(((n & mask).wrapping_add(mask)) >> log2)
}

#[inline(always)]
const fn u32_to_usize(n: u32) -> usize {
    const {
        // Ensure cast from `u32` to `usize` won't overflow
        assert!(usize::BITS >= u32::BITS, "usize too small");
    }

    n as usize
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "assertion ensures panic instead of truncation"
)]
#[inline(always)]
const fn usize_to_u32(n: usize) -> u32 {
    debug_assert!(n < u32_to_usize(u32::MAX), "u32 overflow");
    n as u32
}

#[inline(always)]
const fn mul(size: usize, n: u32) -> u32 {
    if cfg!(debug_assertions) {
        usize_to_u32(size).checked_mul(n).expect("u32 overflow")
    } else {
        usize_to_u32(size) * n
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn from_bytes() {
        assert_eq!(super::from_bytes(0), 0);
        assert_eq!(super::from_bytes(1), 8);
        assert_eq!(super::from_bytes(2), 16);
    }

    #[test]
    fn to_bytes() {
        assert_eq!(super::to_bytes(0), 0);
        assert_eq!(super::to_bytes(1), 1);
        assert_eq!(super::to_bytes(8), 1);
        assert_eq!(super::to_bytes(9), 2);
        assert_eq!(super::to_bytes(16), 2);
        assert_eq!(super::to_bytes(17), 3);
    }

    #[test]
    fn to_limbs() {
        use crate::Limb;

        assert_eq!(super::to_limbs(0), 0);
        assert_eq!(super::to_limbs(1), 1);
        assert_eq!(super::to_limbs(Limb::BITS), 1);
        assert_eq!(super::to_limbs(Limb::BITS + 1), 2);
        assert_eq!(
            super::to_limbs(u32::MAX),
            u32::MAX.div_ceil(Limb::BITS) as usize
        );
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::Limb;

    /// Exact for every limb count whose bit length fits in a `u32`.
    #[kani::proof]
    fn from_limbs_correct() {
        let nlimbs: usize = kani::any();
        let nbits = nlimbs as u128 * u128::from(Limb::BITS);
        kani::assume(nbits <= u128::from(u32::MAX));
        assert_eq!(u128::from(super::from_limbs(nlimbs)), nbits);
    }

    /// Exact for every byte count whose bit length fits in a `u32`.
    #[kani::proof]
    fn from_bytes_correct() {
        let nbytes: usize = kani::any();
        let nbits = nbytes as u128 * 8;
        kani::assume(nbits <= u128::from(u32::MAX));
        assert_eq!(u128::from(super::from_bytes(nbytes)), nbits);
    }

    /// Rounds up for every `u32` input; the result is the smallest count that holds `nbits`.
    #[kani::proof]
    fn to_limbs_and_bytes_round_up() {
        let nbits: u32 = kani::any();
        let bits = u64::from(nbits);

        let nlimbs = super::to_limbs(nbits) as u64;
        assert_eq!(u64::from(super::to_limbs_u32(nbits)), nlimbs);
        assert!(nlimbs * u64::from(Limb::BITS) >= bits);
        assert!(nlimbs == 0 || (nlimbs - 1) * u64::from(Limb::BITS) < bits);

        let nbytes = super::to_bytes(nbits) as u64;
        assert!(nbytes * 8 >= bits);
        assert!(nbytes == 0 || (nbytes - 1) * 8 < bits);
    }

    /// All power-of-two divisors for which the helper is defined, including `2^31`.
    #[kani::proof]
    fn div_ceil_pow2_correct() {
        let (n, log2): (u32, u32) = kani::any();
        kani::assume(log2 < 32);
        let divisor = 1u64 << log2;
        assert_eq!(
            u64::from(super::div_ceil_pow2(n, log2)),
            u64::from(n).div_ceil(divisor)
        );
    }
}
