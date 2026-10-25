/// Constructing a compact representation of a [`Uint`]
use crate::{Limb, Uint};

impl<const LIMBS: usize> Uint<LIMBS> {
    /// Construct a [Uint] containing the bits in `self` in the range `[idx, idx + length)`.
    ///
    /// Assumes `length ≤ Uint::<SECTION_LIMBS>::BITS` and `idx + length ≤ Self::BITS`.
    ///
    /// Executes in time variable in `length` only.
    #[inline(always)]
    #[allow(clippy::cast_possible_truncation)]
    pub(super) const fn section_vartime_length<const SECTION_LIMBS: usize>(
        &self,
        idx: u32,
        length: u32,
    ) -> Uint<SECTION_LIMBS> {
        debug_assert!(length <= Uint::<SECTION_LIMBS>::BITS);
        debug_assert!(idx + length <= Self::BITS);

        if LIMBS > SECTION_LIMBS {
            let (shr_limbs, shr_bits) = (idx >> Limb::LOG2_BITS, idx & (Limb::BITS - 1));
            // shift into the lower SECTION_LIMBS+1 limbs
            let buf = self.bounded_shr_by_limbs(shr_limbs, LIMBS as u32);
            // shift the lower SECTION_LIMBS limbs by the remaining bits and carry the high bits
            buf.resize::<SECTION_LIMBS>()
                .shr_limb_with_carry(
                    shr_bits,
                    buf.limbs[SECTION_LIMBS].unbounded_shl(Limb::BITS - shr_bits),
                )
                .0
                .restrict_bits(length)
        } else {
            self.shr(idx)
                .resize::<SECTION_LIMBS>()
                .restrict_bits(length)
        }
    }

    /// Construct a [Uint] containing the bits in `self` in the range `[idx, idx + length)`.
    ///
    /// Assumes `length ≤ Uint::<SECTION_LIMBS>::BITS` and `idx + length ≤ Self::BITS`.
    ///
    /// Executes in time variable in `idx` and `length`.
    #[inline(always)]
    pub(super) const fn section_vartime<const SECTION_LIMBS: usize>(
        &self,
        idx: u32,
        length: u32,
    ) -> Uint<SECTION_LIMBS> {
        debug_assert!(length <= Uint::<SECTION_LIMBS>::BITS);
        debug_assert!(idx + length <= Self::BITS);

        self.shr_vartime(idx)
            .resize::<SECTION_LIMBS>()
            .restrict_bits(length)
    }

    /// Compact `self` to a form containing the concatenation of its bit ranges `[0, K)`
    /// and `[n-K, n)`.
    ///
    /// Assumes `K ≤ Uint::<SUMMARY_LIMBS>::BITS`, `n ≤ Self::BITS` and `n ≥ 2K`.
    #[inline(always)]
    pub(super) const fn compact<const K: u32, const SUMMARY_LIMBS: usize>(
        &self,
        n: u32,
    ) -> Uint<SUMMARY_LIMBS> {
        debug_assert!(K <= Uint::<SUMMARY_LIMBS>::BITS);
        debug_assert!(n <= Self::BITS);
        debug_assert!(n >= 2 * K);

        // safe to vartime; this function is vartime in length only, which is a public constant
        let hi = self.section_vartime_length(n - K, K);
        // safe to vartime; this function is vartime in idx and length only, which are both public
        // constants
        let lo = self.section_vartime(0, K);
        // safe to vartime; shl_vartime is variable in the value of shift only. Since this shift
        // is a public constant, the constant time property of this algorithm is not impacted.
        hi.shl_vartime(K).bitxor(&lo)
    }

    /// Vartime equivalent of [`Self::compact`].
    #[inline(always)]
    pub(crate) const fn compact_vartime<const K: u32, const SUMMARY_LIMBS: usize>(
        &self,
        n: u32,
    ) -> Uint<SUMMARY_LIMBS> {
        debug_assert!(K <= Uint::<SUMMARY_LIMBS>::BITS);
        debug_assert!(n <= Self::BITS);
        debug_assert!(n >= 2 * K);

        // safe to vartime; this function is vartime in length only, which is a public constant
        let hi = self.section_vartime(n - K, K);
        // safe to vartime; this function is vartime in idx and length only, which are both public
        // constants
        let lo = self.section_vartime(0, K);
        // safe to vartime; shl_vartime is variable in the value of shift only. Since this shift
        // is a public constant, the constant time property of this algorithm is not impacted.
        hi.shl_vartime(K).bitxor(&lo)
    }
}

#[cfg(test)]
mod tests {
    use crate::{U128, U256};

    #[test]
    fn test_compact() {
        let val =
            U256::from_be_hex("CFCF1535CEBE19BBF289933AB8645189397450A32BFEC57579FB7EB14E27D101");
        let target = U128::from_be_hex("BBF289933AB8645179FB7EB14E27D101");

        let compact = val.compact::<64, { U128::LIMBS }>(200);
        assert_eq!(compact, target);
    }

    #[test]
    fn test_compact_vartime() {
        let val =
            U256::from_be_hex("1971BC6285D8CBA9640AA3B3B9C01EF4186D1EBE9A17393A9E43586E0EBAED5B");
        let target = U128::from_be_hex("A9640AA3B3B9C01E9E43586E0EBAED5B");

        let compact = val.compact_vartime::<64, { U128::LIMBS }>(200);
        assert_eq!(compact, target);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{U64, U128};

    fn bits(x: u128, idx: u32, length: u32) -> u128 {
        let mask = 1u128.checked_shl(length).map_or(u128::MAX, |bit| bit - 1);
        x.checked_shr(idx).unwrap_or(0) & mask
    }

    /// Both section extractors return bits `[idx, idx + length)`, through a narrower (`U64`) and
    /// an equal-width (`U128`) section type. `idx == BITS` with `length == 0` is excluded: it is
    /// allowed by the documented precondition but panics (no caller passes it).
    #[kani::proof]
    #[kani::unwind(5)]
    fn section_correct() {
        let (x, idx, length): (u128, u32, u32) = kani::any();
        kani::assume(idx < U128::BITS && u64::from(idx) + u64::from(length) <= 128);
        let ux = U128::from_u128(x);

        if length <= U64::BITS {
            let expected = bits(x, idx, length) as u64;
            let s: U64 = ux.section_vartime_length(idx, length);
            assert_eq!(u64::from(s), expected);
            let s: U64 = ux.section_vartime(idx, length);
            assert_eq!(u64::from(s), expected);
        }
        let s: U128 = ux.section_vartime_length(idx, length);
        assert_eq!(u128::from(s), bits(x, idx, length));
        let s: U128 = ux.section_vartime(idx, length);
        assert_eq!(u128::from(s), bits(x, idx, length));
    }

    /// `compact` and `compact_vartime` concatenate bits `[n - K, n)` above bits `[0, K)`, for
    /// every `n` with `2K <= n <= BITS`.
    #[kani::proof]
    #[kani::unwind(5)]
    fn compact_correct() {
        const K: u32 = 31;
        let (x, n): (u128, u32) = kani::any();
        kani::assume(2 * K <= n && n <= U128::BITS);
        let ux = U128::from_u128(x);

        let expected = ((bits(x, n - K, K) << K) | bits(x, 0, K)) as u64;
        let c: U64 = ux.compact::<K, { U64::LIMBS }>(n);
        assert_eq!(u64::from(c), expected);
        let c: U64 = ux.compact_vartime::<K, { U64::LIMBS }>(n);
        assert_eq!(u64::from(c), expected);
    }
}
