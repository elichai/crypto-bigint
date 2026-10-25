//! Const-friendly decoding/encoding operations for [`Int`].

use crate::{EncodedUint, Encoding, Int, Uint};

impl<const LIMBS: usize> Int<LIMBS> {
    /// Create a new [`Int`] from the provided big endian hex string.
    ///
    /// See [`Uint::from_be_hex`] for more details.
    ///
    /// # Panics
    /// - if the hex is malformed or not zero-padded accordingly for the size.
    #[must_use]
    pub const fn from_be_hex(hex: &str) -> Self {
        Self(Uint::from_be_hex(hex))
    }
}

impl<const LIMBS: usize> Encoding for Int<LIMBS> {
    type Repr = EncodedUint<LIMBS>;

    #[inline]
    fn from_be_bytes(bytes: Self::Repr) -> Self {
        Self(Uint::from_be_bytes(bytes))
    }

    #[inline]
    fn from_le_bytes(bytes: Self::Repr) -> Self {
        Self(Uint::from_le_bytes(bytes))
    }

    #[inline]
    fn to_be_bytes(&self) -> Self::Repr {
        self.0.to_be_bytes()
    }

    #[inline]
    fn to_le_bytes(&self) -> Self::Repr {
        self.0.to_le_bytes()
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{ByteOrder, Encoding, I128};

    /// Every signed bit pattern preserves its native byte representation in both byte orders.
    #[kani::proof]
    #[kani::unwind(18)]
    fn int_encoding_i128_correct() {
        let n: i128 = kani::any();
        let x = I128::from_i128(n);
        assert_eq!(x.to_be_bytes().as_ref(), &n.to_be_bytes());
        assert_eq!(x.to_le_bytes().as_ref(), &n.to_le_bytes());
        assert_eq!(I128::from_be_bytes(n.to_be_bytes().into()), x);
        assert_eq!(I128::from_le_bytes(n.to_le_bytes().into()), x);
    }

    /// The trait's byte-order dispatch decodes arbitrary bytes and re-encodes them exactly.
    #[kani::proof]
    #[kani::unwind(18)]
    fn int_encoding_byte_order_correct() {
        let (bytes, be): ([u8; 16], bool) = kani::any();
        let order = if be {
            ByteOrder::BigEndian
        } else {
            ByteOrder::LittleEndian
        };
        let expected = if be {
            i128::from_be_bytes(bytes)
        } else {
            i128::from_le_bytes(bytes)
        };
        let x = I128::from_bytes(bytes.into(), order);
        assert_eq!(i128::from(x), expected);
        assert_eq!(x.to_bytes(order).as_ref(), &bytes);
    }
}
