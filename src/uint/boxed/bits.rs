//! Bit manipulation functions.

use crate::{BitOps, BoxedUint, Choice, Limb, bitlen};

impl BoxedUint {
    /// Get the value of the bit at position `index`, as a truthy or falsy `Choice`.
    /// Returns the falsy value for indices out of range.
    #[must_use]
    pub fn bit(&self, index: u32) -> Choice {
        self.as_uint_ref().bit(index)
    }

    /// Returns `true` if the bit at position `index` is set, `false` otherwise.
    ///
    /// # Remarks
    /// This operation is variable time with respect to `index` only.
    #[inline(always)]
    #[must_use]
    pub const fn bit_vartime(&self, index: u32) -> bool {
        self.as_uint_ref().bit_vartime(index)
    }

    /// Calculate the number of bits needed to represent this number, i.e. the index of the highest
    /// set bit.
    ///
    /// Use [`BoxedUint::bits_precision`] to get the total capacity of this integer.
    #[must_use]
    pub fn bits(&self) -> u32 {
        self.bits_precision() - self.leading_zeros()
    }

    /// Calculate the number of bits needed to represent this number in variable-time with respect
    /// to `self`.
    #[must_use]
    pub fn bits_vartime(&self) -> u32 {
        self.as_uint_ref().bits_vartime()
    }

    /// Calculate the number of leading zeros in the binary representation of this number.
    #[must_use]
    pub const fn leading_zeros(&self) -> u32 {
        self.as_uint_ref().leading_zeros()
    }

    /// Get the precision of this [`BoxedUint`] in bits.
    #[inline(always)]
    #[must_use]
    pub fn bits_precision(&self) -> u32 {
        bitlen::from_limbs(self.limbs.len())
    }

    /// Calculate the number of trailing zeros in the binary representation of this number.
    #[must_use]
    pub fn trailing_zeros(&self) -> u32 {
        self.as_uint_ref().trailing_zeros()
    }

    /// Calculate the number of trailing ones in the binary representation of this number.
    #[must_use]
    pub fn trailing_ones(&self) -> u32 {
        self.as_uint_ref().trailing_ones()
    }

    /// Calculate the number of trailing zeros in the binary representation of this number in
    /// variable-time with respect to `self`.
    #[must_use]
    pub fn trailing_zeros_vartime(&self) -> u32 {
        self.as_uint_ref().trailing_zeros_vartime()
    }

    /// Calculate the number of trailing ones in the binary representation of this number,
    /// variable time in `self`.
    #[must_use]
    pub fn trailing_ones_vartime(&self) -> u32 {
        self.as_uint_ref().trailing_ones_vartime()
    }

    /// Sets the bit at `index` to 0 or 1 depending on the value of `bit_value`.
    pub(crate) fn set_bit(&mut self, index: u32, bit_value: Choice) {
        self.as_mut_uint_ref().set_bit(index, bit_value);
    }

    pub(crate) fn set_bit_vartime(&mut self, index: u32, bit_value: bool) {
        self.as_mut_uint_ref().set_bit_vartime(index, bit_value);
    }

    /// Clear any bits at or above a given bit position.
    pub(crate) const fn restrict_bits(&mut self, len: u32) {
        self.as_mut_uint_ref().restrict_bits(len);
    }
}

impl BitOps for BoxedUint {
    fn bits_precision(&self) -> u32 {
        self.bits_precision()
    }

    fn bytes_precision(&self) -> usize {
        self.nlimbs() * Limb::BYTES
    }

    fn leading_zeros(&self) -> u32 {
        self.leading_zeros()
    }

    fn bits(&self) -> u32 {
        self.bits()
    }

    fn bit(&self, index: u32) -> Choice {
        self.bit(index)
    }

    fn set_bit(&mut self, index: u32, bit_value: Choice) {
        self.set_bit(index, bit_value);
    }

    fn trailing_zeros(&self) -> u32 {
        self.trailing_zeros()
    }

    fn trailing_ones(&self) -> u32 {
        self.trailing_ones()
    }

    fn bit_vartime(&self, index: u32) -> bool {
        self.bit_vartime(index)
    }

    fn bits_vartime(&self) -> u32 {
        self.bits_vartime()
    }

    fn set_bit_vartime(&mut self, index: u32, bit_value: bool) {
        self.set_bit_vartime(index, bit_value);
    }

    fn trailing_zeros_vartime(&self) -> u32 {
        self.trailing_zeros_vartime()
    }

    fn trailing_ones_vartime(&self) -> u32 {
        self.trailing_ones_vartime()
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{Choice, Resize};

    /// Bit access and bit counts agree with the 128-bit binary representation.
    #[kani::proof]
    #[kani::unwind(5)]
    fn boxed_bit_queries_correct() {
        let a = any_boxed::<{ 2 * L64 }>();
        let n = value(&a);
        let index: u32 = kani::any();
        let expected = index < 128 && (n >> index) & 1 != 0;
        assert_eq!(a.bit(index).to_bool(), expected);
        assert_eq!(a.bit_vartime(index), expected);
        assert_eq!(a.leading_zeros(), n.leading_zeros());
        assert_eq!(a.bits(), 128 - n.leading_zeros());
        assert_eq!(a.bits_vartime(), 128 - n.leading_zeros());
        assert_eq!(a.trailing_zeros(), n.trailing_zeros());
        assert_eq!(a.trailing_zeros_vartime(), n.trailing_zeros());
        assert_eq!(a.trailing_ones(), n.trailing_ones());
        assert_eq!(a.trailing_ones_vartime(), n.trailing_ones());
    }

    #[kani::proof]
    #[kani::unwind(5)]
    fn boxed_set_bit_correct() {
        let mut a = any_boxed::<{ 2 * L64 }>();
        let n = value(&a);
        let index: u32 = kani::any();
        let bit: bool = kani::any();
        let expected = if index >= 128 {
            n
        } else if bit {
            n | (1u128 << index)
        } else {
            n & !(1u128 << index)
        };
        let mut b = a.clone();
        a.set_bit(index, if bit { Choice::TRUE } else { Choice::FALSE });
        assert_eq!(value(&a), expected);
        assert_eq!(a.bits_precision(), 128);
        // The variable-time setter requires an in-range index.
        if index < 128 {
            b.set_bit_vartime(index, bit);
            assert_eq!(value(&b), expected);
        }
    }

    /// Restriction keeps exactly the low `len` bits, including zero and out-of-range lengths.
    #[kani::proof]
    #[kani::unwind(5)]
    fn boxed_restrict_bits_correct() {
        let mut a = any_boxed::<{ 2 * L64 }>();
        let n = value(&a);
        let len: u32 = kani::any();
        let expected = if len >= 128 {
            n
        } else {
            n & ((1u128 << len) - 1)
        };
        a.restrict_bits(len);
        assert_eq!(value(&a), expected);
        assert_eq!(a.bits_precision(), 128);
    }

    #[kani::proof]
    #[kani::unwind(5)]
    fn boxed_resize_correct() {
        let a = any_boxed::<{ 2 * L64 }>();
        let n = value(&a);
        let truncated = (&a).resize_unchecked(64);
        assert_eq!(
            (value(&truncated), truncated.bits_precision()),
            (n & u128::from(u64::MAX), 64)
        );
        let owned = a.clone().resize_unchecked(64);
        assert_eq!(
            (value(&owned), owned.bits_precision()),
            (n & u128::from(u64::MAX), 64)
        );
        assert_eq!(
            (&a).try_resize(64).map(|r| value(&r)),
            if n <= u128::from(u64::MAX) {
                Some(n)
            } else {
                None
            }
        );
        assert_eq!(
            a.try_resize(64).map(|r| value(&r)),
            if n <= u128::from(u64::MAX) {
                Some(n)
            } else {
                None
            }
        );
        let grown = truncated.resize_unchecked(128);
        assert_eq!(
            (value(&grown), grown.bits_precision()),
            (n & u128::from(u64::MAX), 128)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::BoxedUint;
    use crate::Choice;
    use hex_literal::hex;

    fn uint_with_bits_at(positions: &[u32]) -> BoxedUint {
        let mut result = BoxedUint::zero_with_precision(256);
        for &pos in positions {
            result |= BoxedUint::one_with_precision(256).shl_vartime(pos).unwrap();
        }
        result
    }

    #[test]
    fn bit_vartime() {
        let u = uint_with_bits_at(&[16, 48, 112, 127, 255]);
        assert!(!u.bit_vartime(0));
        assert!(!u.bit_vartime(1));
        assert!(u.bit_vartime(16));
        assert!(u.bit_vartime(127));
        assert!(u.bit_vartime(255));
        assert!(!u.bit_vartime(256));
        assert!(!u.bit_vartime(260));
    }

    #[test]
    fn bits() {
        assert_eq!(0, BoxedUint::zero().bits());
        assert_eq!(128, BoxedUint::max(128).bits());

        let n1 = BoxedUint::from_be_slice(&hex!("000000000029ffffffffffffffffffff"), 128).unwrap();
        assert_eq!(86, n1.bits());

        let n2 = BoxedUint::from_be_slice(&hex!("00000000004000000000000000000000"), 128).unwrap();
        assert_eq!(87, n2.bits());
    }

    #[test]
    fn set_bit() {
        let mut u = uint_with_bits_at(&[16, 79, 150]);
        u.set_bit(127, Choice::TRUE);
        assert_eq!(u, uint_with_bits_at(&[16, 79, 127, 150]));

        let mut u = uint_with_bits_at(&[16, 79, 150]);
        u.set_bit(150, Choice::TRUE);
        assert_eq!(u, uint_with_bits_at(&[16, 79, 150]));

        let mut u = uint_with_bits_at(&[16, 79, 150]);
        u.set_bit(127, Choice::FALSE);
        assert_eq!(u, uint_with_bits_at(&[16, 79, 150]));

        let mut u = uint_with_bits_at(&[16, 79, 150]);
        u.set_bit(150, Choice::FALSE);
        assert_eq!(u, uint_with_bits_at(&[16, 79]));
    }

    #[test]
    fn trailing_ones() {
        let u = !uint_with_bits_at(&[16, 79, 150]);
        assert_eq!(u.trailing_ones(), 16);

        let u = !uint_with_bits_at(&[79, 150]);
        assert_eq!(u.trailing_ones(), 79);

        let u = !uint_with_bits_at(&[150, 207]);
        assert_eq!(u.trailing_ones(), 150);

        let u = !uint_with_bits_at(&[0, 150, 207]);
        assert_eq!(u.trailing_ones(), 0);

        let u = !BoxedUint::zero_with_precision(256);
        assert_eq!(u.trailing_ones(), 256);
    }

    #[test]
    fn trailing_ones_vartime() {
        let u = !uint_with_bits_at(&[16, 79, 150]);
        assert_eq!(u.trailing_ones_vartime(), 16);

        let u = !uint_with_bits_at(&[79, 150]);
        assert_eq!(u.trailing_ones_vartime(), 79);

        let u = !uint_with_bits_at(&[150, 207]);
        assert_eq!(u.trailing_ones_vartime(), 150);

        let u = !uint_with_bits_at(&[0, 150, 207]);
        assert_eq!(u.trailing_ones_vartime(), 0);

        let u = !BoxedUint::zero_with_precision(256);
        assert_eq!(u.trailing_ones_vartime(), 256);
    }
}
