//! `From`-like conversions for [`BoxedUint`].

use crate::{BoxedUint, Limb, Odd, U64, U128, Uint, UintRef, Word};
use alloc::{boxed::Box, vec::Vec};
use core::mem;

impl From<u8> for BoxedUint {
    fn from(n: u8) -> Self {
        vec![Limb::from(n); 1].into()
    }
}

impl From<u16> for BoxedUint {
    fn from(n: u16) -> Self {
        vec![Limb::from(n); 1].into()
    }
}

impl From<u32> for BoxedUint {
    fn from(n: u32) -> Self {
        vec![Limb::from(n); 1].into()
    }
}

impl From<u64> for BoxedUint {
    fn from(n: u64) -> Self {
        U64::from(n).into()
    }
}

impl From<u128> for BoxedUint {
    fn from(n: u128) -> Self {
        U128::from(n).into()
    }
}

impl From<Limb> for BoxedUint {
    fn from(limb: Limb) -> Self {
        vec![limb; 1].into()
    }
}

impl From<&[Limb]> for BoxedUint {
    fn from(limbs: &[Limb]) -> BoxedUint {
        Self {
            limbs: limbs.into(),
        }
    }
}

impl From<Box<[Limb]>> for BoxedUint {
    fn from(limbs: Box<[Limb]>) -> BoxedUint {
        Vec::from(limbs).into()
    }
}

impl From<Vec<Limb>> for BoxedUint {
    fn from(mut limbs: Vec<Limb>) -> BoxedUint {
        if limbs.is_empty() {
            limbs.push(Limb::ZERO);
        }

        Self {
            limbs: limbs.into_boxed_slice(),
        }
    }
}

impl From<Vec<Word>> for BoxedUint {
    fn from(mut words: Vec<Word>) -> BoxedUint {
        // SAFETY: `Limb` is a `repr(transparent)` newtype for `Word`
        #[allow(unsafe_code)]
        unsafe {
            let ptr = words.as_mut_ptr().cast::<Limb>();
            let len = words.len();
            let capacity = words.capacity();
            mem::forget(words);
            Vec::<Limb>::from_raw_parts(ptr, len, capacity)
        }
        .into()
    }
}

impl<const LIMBS: usize> From<Uint<LIMBS>> for BoxedUint {
    #[inline]
    fn from(uint: Uint<LIMBS>) -> BoxedUint {
        Self::from(&uint)
    }
}

impl<const LIMBS: usize> From<&Uint<LIMBS>> for BoxedUint {
    #[inline]
    fn from(uint: &Uint<LIMBS>) -> BoxedUint {
        Vec::from(uint.to_limbs()).into()
    }
}

impl<const LIMBS: usize> From<Odd<Uint<LIMBS>>> for BoxedUint {
    #[inline]
    fn from(uint: Odd<Uint<LIMBS>>) -> BoxedUint {
        Self::from(uint.as_ref())
    }
}

impl<const LIMBS: usize> From<&Odd<Uint<LIMBS>>> for BoxedUint {
    #[inline]
    fn from(uint: &Odd<Uint<LIMBS>>) -> BoxedUint {
        Self::from(uint.as_ref())
    }
}

impl<const LIMBS: usize> From<Odd<Uint<LIMBS>>> for Odd<BoxedUint> {
    #[inline]
    fn from(uint: Odd<Uint<LIMBS>>) -> Odd<BoxedUint> {
        Odd::new_unchecked(BoxedUint::from(uint.as_ref()))
    }
}

impl<const LIMBS: usize> From<&Odd<Uint<LIMBS>>> for Odd<BoxedUint> {
    #[inline]
    fn from(uint: &Odd<Uint<LIMBS>>) -> Odd<BoxedUint> {
        Odd::new_unchecked(BoxedUint::from(uint.as_ref()))
    }
}

impl From<&UintRef> for BoxedUint {
    fn from(uint_ref: &UintRef) -> BoxedUint {
        debug_assert!(uint_ref.nlimbs() >= 1, "empty `UintRef`");
        BoxedUint {
            limbs: uint_ref.as_limbs().into(),
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use crate::uint::boxed::kani_util::value;

    fn check_value(result: BoxedUint, expected: u128, bits: u32) {
        assert_eq!((value(&result), result.bits_precision()), (expected, bits));
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn boxed_from_scalars() {
        let n: u128 = kani::any();
        check_value(BoxedUint::from(n as u8), u128::from(n as u8), Word::BITS);
        check_value(BoxedUint::from(n as u16), u128::from(n as u16), Word::BITS);
        check_value(BoxedUint::from(n as u32), u128::from(n as u32), Word::BITS);
        check_value(BoxedUint::from(n as u64), u128::from(n as u64), 64);
        check_value(BoxedUint::from(n), n, 128);
        check_value(
            BoxedUint::from(Limb(n as Word)),
            u128::from(n as Word),
            Word::BITS,
        );
    }

    fn check_words(result: BoxedUint, words: &[Word]) {
        let len = words.len().max(1);
        assert_eq!(result.nlimbs(), len);
        assert_eq!(result.bits_precision(), len as u32 * Word::BITS);
        for (i, limb) in result.as_limbs().iter().enumerate() {
            assert_eq!(limb.0, words.get(i).copied().unwrap_or(0));
        }
    }

    fn word_vec_capacity<const N: usize, const SPARE: usize>() {
        let words: [Word; N] = kani::any();
        let mut input = Vec::with_capacity(N + SPARE);
        input.extend_from_slice(&words);
        check_words(BoxedUint::from(input), &words);
    }

    fn word_vec<const N: usize>() {
        word_vec_capacity::<N, 0>();
        word_vec_capacity::<N, 1>();
        word_vec_capacity::<N, 2>();
    }

    /// Exercise the unsafe ownership cast at lengths 0..4, with 0..2 spare capacity.
    // Symbolic lengths or capacities timed out; concrete calls retain all 15 size combinations.
    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_from_word_vec() {
        word_vec::<0>();
        word_vec::<1>();
        word_vec::<2>();
        word_vec::<3>();
        word_vec::<4>();
    }

    fn owned_limbs<const N: usize>() {
        let words: [Word; N] = kani::any();
        let limbs = words.map(Limb);
        check_words(BoxedUint::from(limbs.to_vec()), &words);
        check_words(BoxedUint::from(limbs.to_vec().into_boxed_slice()), &words);
    }

    /// Owned limb containers preserve lengths 1..4 and normalize empty input to one zero limb.
    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_from_owned_limb_containers() {
        owned_limbs::<0>();
        owned_limbs::<1>();
        owned_limbs::<2>();
        owned_limbs::<3>();
        owned_limbs::<4>();
    }

    /// Borrowed conversions preserve all 128 bits and their allocated precision.
    #[kani::proof]
    #[kani::unwind(6)]
    fn boxed_from_uint_and_views() {
        let n: u128 = kani::any();
        let input = U128::from_u128(n);
        check_value(BoxedUint::from(input), n, 128);
        check_value(BoxedUint::from(&input), n, 128);
        check_value(BoxedUint::from(input.as_uint_ref()), n, 128);
        check_value(BoxedUint::from(&input.as_limbs()[..]), n, 128);
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn boxed_from_odd_uint() {
        let n = kani::any::<u128>() | 1;
        let input = U128::from_u128(n).to_odd().unwrap();
        check_value(BoxedUint::from(input), n, 128);
        check_value(BoxedUint::from(&input), n, 128);
        check_value(Odd::<BoxedUint>::from(input).get(), n, 128);
        check_value(Odd::<BoxedUint>::from(&input).get(), n, 128);
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::{BoxedUint, Limb, Odd, U256};

    #[test]
    fn from_limb_slice() {
        let n = BoxedUint::from(&[Limb(42), Limb::MAX][..]);
        assert_eq!(n.as_limbs(), &[Limb(42), Limb::MAX]);
    }

    #[test]
    fn from_boxed_limb_slice() {
        let n = BoxedUint::from(Vec::<Limb>::new().into_boxed_slice());
        assert_eq!(n.as_limbs(), &[Limb::ZERO]);

        let n = BoxedUint::from(vec![Limb(42), Limb::MAX].into_boxed_slice());
        assert_eq!(n.as_limbs(), &[Limb(42), Limb::MAX]);
    }

    #[test]
    fn from_vec() {
        let n = BoxedUint::from(Vec::<Limb>::new());
        assert_eq!(n.as_limbs(), &[Limb::ZERO]);

        let n = BoxedUint::from(vec![Limb(42), Limb::MAX]);
        assert_eq!(n.as_limbs(), &[Limb(42), Limb::MAX]);
    }

    #[test]
    fn from_odd_uint() {
        let odd_u = U256::MAX.to_odd().unwrap();
        let n = BoxedUint::from(&odd_u);
        assert_eq!(n.as_limbs(), odd_u.as_ref().as_limbs());
        let n = BoxedUint::from(odd_u);
        assert_eq!(n.as_limbs(), odd_u.as_ref().as_limbs());
        let n = Odd::<BoxedUint>::from(&odd_u);
        assert_eq!(n.as_ref().as_limbs(), odd_u.as_ref().as_limbs());
        let n = Odd::<BoxedUint>::from(odd_u);
        assert_eq!(n.as_ref().as_limbs(), odd_u.as_ref().as_limbs());
    }
}
