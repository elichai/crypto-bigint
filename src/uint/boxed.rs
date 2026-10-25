//! Heap-allocated big unsigned integers.

mod add;
mod add_mod;
mod bit_and;
mod bit_not;
mod bit_or;
mod bit_xor;
mod bits;
mod cmp;
mod ct;
pub(crate) mod div;
pub(crate) mod encoding;
mod from;
mod gcd;
mod invert_mod;
mod lcm;
mod mul;
mod mul_mod;
mod neg;
mod neg_mod;
mod pow;
mod pow_mod;
mod shl;
mod shr;
mod sqrt;
mod sub;
mod sub_mod;

#[cfg(feature = "rand_core")]
mod rand;

use crate::{
    Choice, CtAssign, CtEq, CtOption, Integer, Limb, NonZero, Odd, One, Resize, UintRef, Unsigned,
    UnsignedWithMontyForm, Word, Zero, bitlen, modular::BoxedMontyForm, traits::sealed::Sealed,
};
use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    borrow::{Borrow, BorrowMut},
    fmt,
    iter::repeat,
};

#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

/// Fixed-precision heap-allocated big unsigned integer.
///
/// Alternative to the stack-allocated [`Uint`][`crate::Uint`] but with a
/// fixed precision chosen at runtime instead of compile time.
///
/// Unlike many other heap-allocated big integer libraries, this type is not
/// arbitrary precision and will wrap at its fixed-precision rather than
/// automatically growing.
#[derive(Clone)]
pub struct BoxedUint {
    /// Boxed slice containing limbs.
    ///
    /// Stored from least significant to most significant.
    pub(crate) limbs: Box<[Limb]>,
}

impl BoxedUint {
    /// Get the value `0` represented as succinctly as possible.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            limbs: vec![Limb::ZERO; 1].into(),
        }
    }

    /// Get the value `0` with the given number of bits of precision.
    ///
    /// `at_least_bits_precision` is rounded up to a multiple of [`Limb::BITS`].
    #[must_use]
    pub fn zero_with_precision(at_least_bits_precision: u32) -> Self {
        vec![Limb::ZERO; bitlen::to_limbs(at_least_bits_precision)].into()
    }

    /// Get the value `1`, represented as succinctly as possible.
    #[must_use]
    pub fn one() -> Self {
        Self {
            limbs: vec![Limb::ONE; 1].into(),
        }
    }

    /// Get the value `1` with the given number of bits of precision.
    ///
    /// `at_least_bits_precision` is rounded up to a multiple of [`Limb::BITS`].
    #[must_use]
    pub fn one_with_precision(at_least_bits_precision: u32) -> Self {
        let mut ret = Self::zero_with_precision(at_least_bits_precision);
        ret.limbs[0] = Limb::ONE;
        ret
    }

    /// Is this [`BoxedUint`] equal to zero?
    #[must_use]
    pub fn is_zero(&self) -> Choice {
        self.limbs
            .iter()
            .fold(Choice::TRUE, |acc, limb| acc & limb.is_zero())
    }

    /// Is this [`BoxedUint`] *NOT* equal to zero?
    #[inline]
    #[must_use]
    pub fn is_nonzero(&self) -> Choice {
        !self.is_zero()
    }

    /// Is this [`BoxedUint`] equal to one?
    #[must_use]
    pub fn is_one(&self) -> Choice {
        let mut iter = self.limbs.iter();
        let choice = iter.next().copied().unwrap_or(Limb::ZERO).ct_eq(&Limb::ONE);
        iter.fold(choice, |acc, limb| acc & limb.is_zero())
    }

    /// Get the maximum value for a `BoxedUint` created with `at_least_bits_precision`
    /// precision bits requested.
    ///
    /// That is, returns the value `2^self.bits_precision() - 1`.
    #[must_use]
    pub fn max(at_least_bits_precision: u32) -> Self {
        vec![Limb::MAX; bitlen::to_limbs(at_least_bits_precision)].into()
    }

    /// Create a [`BoxedUint`] from an array of [`Word`]s (i.e. word-sized unsigned
    /// integers).
    #[inline]
    pub fn from_words(words: impl IntoIterator<Item = Word>) -> Self {
        Self {
            limbs: words.into_iter().map(Into::into).collect(),
        }
    }

    /// Create a [`BoxedUint`] from an array of [`Word`]s (i.e. word-sized unsigned
    /// integers), specifying the precision of the result. Any words above the given
    /// precision will be dropped.
    #[inline]
    pub fn from_words_with_precision(
        words: impl IntoIterator<Item = Word>,
        at_least_bits_precision: u32,
    ) -> Self {
        let size = bitlen::to_limbs(at_least_bits_precision);
        Self {
            limbs: words
                .into_iter()
                .map(Into::into)
                .chain(repeat(Limb::ZERO))
                .take(size)
                .collect(),
        }
    }

    /// Create a boxed slice of [`Word`]s (i.e. word-sized unsigned integers) from
    /// a [`BoxedUint`].
    #[inline]
    pub fn to_words(&self) -> Box<[Word]> {
        self.limbs.iter().copied().map(Into::into).collect()
    }

    /// Borrow the inner limbs as a slice of [`Word`]s.
    #[must_use]
    pub fn as_words(&self) -> &[Word] {
        self.as_uint_ref().as_words()
    }

    /// Borrow the inner limbs as a mutable slice of [`Word`]s.
    pub fn as_mut_words(&mut self) -> &mut [Word] {
        self.as_mut_uint_ref().as_mut_words()
    }

    /// Borrow the inner limbs as a mutable slice of [`Word`]s.
    #[deprecated(since = "0.7.0", note = "please use `as_mut_words` instead")]
    pub fn as_words_mut(&mut self) -> &mut [Word] {
        self.as_mut_words()
    }

    /// Borrow the limbs of this [`BoxedUint`].
    #[must_use]
    pub fn as_limbs(&self) -> &[Limb] {
        self.limbs.as_ref()
    }

    /// Borrow the limbs of this [`BoxedUint`] mutably.
    pub fn as_mut_limbs(&mut self) -> &mut [Limb] {
        self.limbs.as_mut()
    }

    /// Borrow the limbs of this [`BoxedUint`] mutably.
    #[deprecated(since = "0.7.0", note = "please use `as_mut_limbs` instead")]
    pub fn as_limbs_mut(&mut self) -> &mut [Limb] {
        self.as_mut_limbs()
    }

    /// Convert this [`BoxedUint`] into its inner limbs.
    #[must_use]
    pub fn to_limbs(&self) -> Box<[Limb]> {
        self.limbs.clone()
    }

    /// Convert this [`BoxedUint`] into its inner limbs.
    #[must_use]
    pub fn into_limbs(self) -> Box<[Limb]> {
        self.limbs
    }

    /// Borrow the limbs of this [`BoxedUint`] as a [`UintRef`].
    #[inline]
    #[must_use]
    pub const fn as_uint_ref(&self) -> &UintRef {
        UintRef::new(&self.limbs)
    }

    /// Mutably borrow the limbs of this [`BoxedUint`] as a [`UintRef`].
    #[inline]
    #[must_use]
    pub const fn as_mut_uint_ref(&mut self) -> &mut UintRef {
        UintRef::new_mut(&mut self.limbs)
    }

    /// Get the number of limbs in this [`BoxedUint`].
    #[must_use]
    pub fn nlimbs(&self) -> usize {
        self.limbs.len()
    }

    /// Construct a [`NonZero`] reference, returning [`None`] in the event `self` is `0`.
    #[must_use]
    pub fn as_nz_vartime(&self) -> Option<&NonZero<Self>> {
        if self.is_zero_vartime() {
            None
        } else {
            Some(NonZero::new_ref_unchecked(self))
        }
    }

    /// Convert to a [`NonZero<BoxedUint>`].
    ///
    /// Returns some if the original value is non-zero, and none otherwise.
    #[must_use]
    pub fn to_nz(&self) -> CtOption<NonZero<Self>> {
        self.clone().into_nz()
    }

    /// Convert to a [`NonZero<BoxedUint>`], defaulting to one.
    ///
    /// Returns a pair consisting of a [`NonZero<BoxedUint>`], and a [`Choice`]
    /// indicating whether the original value was non-zero (and preserved).
    #[inline(always)]
    #[must_use]
    pub(crate) fn to_nz_or_one(&self) -> (NonZero<Self>, Choice) {
        let is_zero = self.is_zero();
        let mut nz = self.clone();
        nz.as_mut_uint_ref().conditional_set_one(is_zero);
        (NonZero::new_unchecked(nz), !is_zero)
    }

    /// Convert to an [`Odd<BoxedUint>`], defaulting to one.
    ///
    /// Returns a pair consisting of an [`Odd<BoxedUint>`], and a [`Choice`]
    /// indicating whether the original value was odd (and preserved).
    #[inline(always)]
    #[must_use]
    pub(crate) fn to_odd_or_one(&self) -> (Odd<Self>, Choice) {
        let is_odd = self.is_odd();
        let mut odd = self.clone();
        odd.as_mut_uint_ref().conditional_set_one(is_odd.not());
        (Odd::new_unchecked(odd), is_odd)
    }

    /// Construct an [`Odd`] reference, returning [`None`] in the event `self` is even.
    #[must_use]
    pub fn as_odd_vartime(&self) -> Option<&Odd<Self>> {
        if !self.is_odd().to_bool() {
            None
        } else {
            Some(Odd::new_ref_unchecked(self))
        }
    }

    /// Convert to an [`Odd<BoxedUint>`].
    ///
    /// Returns some if the original value is odd, and none otherwise.
    #[must_use]
    pub fn to_odd(&self) -> CtOption<Odd<Self>> {
        self.clone().into_odd()
    }

    /// Convert to a [`NonZero<BoxedUint>`].
    ///
    /// Returns some if the original value is non-zero, and none otherwise.
    #[must_use]
    pub fn into_nz(mut self) -> CtOption<NonZero<Self>> {
        let is_nz = self.is_nonzero();

        // Ensure the `NonZero` we construct is actually non-zero, even if the `CtOption` is none
        self.limbs[0].ct_assign(&Limb::ONE, !is_nz);
        CtOption::new(NonZero::new_unchecked(self), is_nz)
    }

    /// Convert to an [`Odd<BoxedUint>`].
    ///
    /// Returns some if the original value is odd, and none otherwise.
    #[must_use]
    pub fn into_odd(mut self) -> CtOption<Odd<Self>> {
        let is_odd = self.is_odd();

        // Ensure the `Odd` we construct is actually odd, even if the `CtOption` is none
        self.limbs[0].ct_assign(&Limb::ONE, !is_odd);
        CtOption::new(Odd::new_unchecked(self.clone()), is_odd)
    }

    /// Widen this type's precision to the given number of bits.
    ///
    /// # Panics
    /// - if `at_least_bits_precision` is smaller than the current precision.
    #[must_use]
    #[deprecated(since = "0.7.0", note = "please use `resize` instead")]
    pub fn widen(&self, at_least_bits_precision: u32) -> BoxedUint {
        assert!(at_least_bits_precision >= self.bits_precision());

        let mut ret = BoxedUint::zero_with_precision(at_least_bits_precision);
        ret.limbs[..self.nlimbs()].copy_from_slice(&self.limbs);
        ret
    }

    /// Shortens this type's precision to the given number of bits.
    ///
    /// # Panics
    /// - if `at_least_bits_precision` is larger than the current precision.
    #[must_use]
    #[deprecated(since = "0.7.0", note = "please use `resize` instead")]
    pub fn shorten(&self, at_least_bits_precision: u32) -> BoxedUint {
        assert!(at_least_bits_precision <= self.bits_precision());
        let mut ret = BoxedUint::zero_with_precision(at_least_bits_precision);
        let nlimbs = ret.nlimbs();
        ret.limbs.copy_from_slice(&self.limbs[..nlimbs]);
        ret
    }

    /// Iterate over the limbs of the inputs, applying the given function, and
    /// constructing a result from the returned values.
    #[inline]
    fn map_limbs<F>(lhs: &Self, rhs: &Self, f: F) -> Self
    where
        F: Fn(Limb, Limb) -> Limb,
    {
        let nlimbs = cmp::max(lhs.nlimbs(), rhs.nlimbs());
        let mut limbs = Vec::with_capacity(nlimbs);

        for i in 0..nlimbs {
            let &a = lhs.limbs.get(i).unwrap_or(&Limb::ZERO);
            let &b = rhs.limbs.get(i).unwrap_or(&Limb::ZERO);
            limbs.push(f(a, b));
        }

        limbs.into()
    }

    /// Resize this value in place without checking whether the current value fits.
    ///
    /// If `at_least_bits_precision` is below the current precision, high limbs
    /// are truncated. If it is above the current precision, new high limbs are
    /// zero.
    pub(crate) fn resize_in_place_unchecked(&mut self, at_least_bits_precision: u32) {
        let new_len = bitlen::to_limbs(at_least_bits_precision);

        if new_len != self.limbs.len() {
            let mut limbs = core::mem::take(&mut self.limbs).into_vec();
            limbs.resize(new_len, Limb::ZERO);
            *self = Self::from(limbs);
        }
    }

    /// Returns `true` if the integer's bit size is smaller or equal to `bits`.
    pub(crate) fn is_within_bits(&self, bits: u32) -> bool {
        bits >= self.bits_precision() || bits >= self.bits()
    }
}

impl Resize for BoxedUint {
    type Output = BoxedUint;

    fn resize_unchecked(mut self, at_least_bits_precision: u32) -> Self::Output {
        self.resize_in_place_unchecked(at_least_bits_precision);
        self
    }

    fn try_resize(self, at_least_bits_precision: u32) -> Option<BoxedUint> {
        if self.is_within_bits(at_least_bits_precision) {
            Some(self.resize_unchecked(at_least_bits_precision))
        } else {
            None
        }
    }
}

impl Resize for &BoxedUint {
    type Output = BoxedUint;

    fn resize_unchecked(self, at_least_bits_precision: u32) -> Self::Output {
        let mut ret = BoxedUint::zero_with_precision(at_least_bits_precision);
        let num_limbs_to_copy = core::cmp::min(ret.limbs.len(), self.limbs.len());
        ret.limbs[..num_limbs_to_copy].copy_from_slice(&self.limbs[..num_limbs_to_copy]);
        ret
    }

    fn try_resize(self, at_least_bits_precision: u32) -> Option<BoxedUint> {
        if self.is_within_bits(at_least_bits_precision) {
            Some(self.resize_unchecked(at_least_bits_precision))
        } else {
            None
        }
    }
}

impl Resize for NonZero<BoxedUint> {
    type Output = Self;

    fn resize_unchecked(self, at_least_bits_precision: u32) -> Self::Output {
        NonZero::new_unchecked(self.get().resize_unchecked(at_least_bits_precision))
    }

    fn try_resize(self, at_least_bits_precision: u32) -> Option<Self::Output> {
        self.get()
            .try_resize(at_least_bits_precision)
            .map(NonZero::new_unchecked)
    }
}

impl Resize for &NonZero<BoxedUint> {
    type Output = NonZero<BoxedUint>;

    fn resize_unchecked(self, at_least_bits_precision: u32) -> Self::Output {
        NonZero::new_unchecked(self.as_ref().resize_unchecked(at_least_bits_precision))
    }

    fn try_resize(self, at_least_bits_precision: u32) -> Option<Self::Output> {
        self.as_ref()
            .try_resize(at_least_bits_precision)
            .map(NonZero::new_unchecked)
    }
}

impl AsRef<[Word]> for BoxedUint {
    fn as_ref(&self) -> &[Word] {
        self.as_words()
    }
}

impl AsMut<[Word]> for BoxedUint {
    fn as_mut(&mut self) -> &mut [Word] {
        self.as_mut_words()
    }
}

impl AsRef<[Limb]> for BoxedUint {
    fn as_ref(&self) -> &[Limb] {
        self.as_limbs()
    }
}

impl AsMut<[Limb]> for BoxedUint {
    fn as_mut(&mut self) -> &mut [Limb] {
        self.as_mut_limbs()
    }
}

impl AsRef<UintRef> for BoxedUint {
    fn as_ref(&self) -> &UintRef {
        self.as_uint_ref()
    }
}

impl AsMut<UintRef> for BoxedUint {
    fn as_mut(&mut self) -> &mut UintRef {
        self.as_mut_uint_ref()
    }
}

impl Borrow<UintRef> for BoxedUint {
    fn borrow(&self) -> &UintRef {
        self.as_uint_ref()
    }
}

impl BorrowMut<UintRef> for BoxedUint {
    fn borrow_mut(&mut self) -> &mut UintRef {
        self.as_mut_uint_ref()
    }
}

impl Default for BoxedUint {
    fn default() -> Self {
        Self::zero()
    }
}

impl Integer for BoxedUint {
    fn as_limbs(&self) -> &[Limb] {
        &self.limbs
    }

    fn as_mut_limbs(&mut self) -> &mut [Limb] {
        &mut self.limbs
    }

    fn nlimbs(&self) -> usize {
        self.nlimbs()
    }
}

impl Sealed for BoxedUint {}

impl Unsigned for BoxedUint {
    fn as_uint_ref(&self) -> &UintRef {
        self.as_uint_ref()
    }

    fn as_mut_uint_ref(&mut self) -> &mut UintRef {
        self.as_mut_uint_ref()
    }

    fn from_limb_like(limb: Limb, other: &Self) -> Self {
        let mut ret = Self::zero_with_precision(other.bits_precision());
        ret.limbs[0] = limb;
        ret
    }
}

impl UnsignedWithMontyForm for BoxedUint {
    type MontyForm = BoxedMontyForm;
}

impl Zero for BoxedUint {
    fn zero() -> Self {
        Self::zero()
    }

    fn is_zero(&self) -> Choice {
        self.is_zero()
    }

    fn set_zero(&mut self) {
        self.limbs.as_mut().fill(Limb::ZERO);
    }
}

impl One for BoxedUint {
    fn one() -> Self {
        Self::one()
    }

    fn one_like(other: &Self) -> Self {
        let mut ret = other.clone();
        ret.set_one();
        ret
    }

    fn is_one(&self) -> Choice {
        self.is_one()
    }

    fn set_one(&mut self) {
        self.limbs.as_mut().fill(Limb::ZERO);
        self.limbs[0] = Limb::ONE;
    }
}

impl num_traits::Zero for BoxedUint {
    fn zero() -> Self {
        Self::zero()
    }

    fn is_zero(&self) -> bool {
        self.is_zero().into()
    }

    fn set_zero(&mut self) {
        Zero::set_zero(self);
    }
}

impl num_traits::One for BoxedUint {
    fn one() -> Self {
        Self::one()
    }

    fn is_one(&self) -> bool {
        self.is_one().into()
    }

    fn set_one(&mut self) {
        One::set_one(self);
    }
}

#[cfg(feature = "zeroize")]
impl Zeroize for BoxedUint {
    fn zeroize(&mut self) {
        self.limbs.zeroize();
    }
}

impl fmt::Debug for BoxedUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BoxedUint(0x{:X})", self.as_uint_ref())
    }
}

impl fmt::Display for BoxedUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(self, f)
    }
}

impl fmt::Binary for BoxedUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(self.as_uint_ref(), f)
    }
}

impl fmt::LowerHex for BoxedUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self.as_uint_ref(), f)
    }
}

impl fmt::UpperHex for BoxedUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(self.as_uint_ref(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::BoxedUint;
    use crate::Word;
    use alloc::vec::Vec;

    #[test]
    fn from_word_vec() {
        let words: &[Word] = &[0, 1, 2, 3];
        let uint = BoxedUint::from(Vec::from(words));
        assert_eq!(uint.nlimbs(), 4);
        assert_eq!(uint.as_words(), words);
    }

    #[test]
    fn fmt_lower_hex() {
        let n = BoxedUint::from_be_hex("AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD", 128).unwrap();
        assert_eq!(format!("{n:x}"), "aaaaaaaabbbbbbbbccccccccdddddddd");
        assert_eq!(format!("{n:#x}"), "0xaaaaaaaabbbbbbbbccccccccdddddddd");
    }

    #[test]
    fn fmt_upper_hex() {
        let n = BoxedUint::from_be_hex("aaaaaaaabbbbbbbbccccccccdddddddd", 128).unwrap();
        assert_eq!(format!("{n:X}"), "AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");
        assert_eq!(format!("{n:#X}"), "0xAAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");
    }

    #[test]
    fn fmt_binary() {
        let n = BoxedUint::from_be_hex("aaaaaaaabbbbbbbbccccccccdddddddd", 128).unwrap();
        assert_eq!(
            format!("{n:b}"),
            "10101010101010101010101010101010101110111011101110111011101110111100110011001100110011001100110011011101110111011101110111011101"
        );
        assert_eq!(
            format!("{n:#b}"),
            "0b10101010101010101010101010101010101110111011101110111011101110111100110011001100110011001100110011011101110111011101110111011101"
        );
    }

    #[test]
    fn test_unsigned() {
        crate::traits::tests::test_unsigned(
            BoxedUint::zero_with_precision(128),
            BoxedUint::max(128),
        );
    }

    #[test]
    fn test_unsigned_monty_form() {
        crate::traits::tests::test_unsigned_monty_form::<BoxedUint>();
    }
}

/// Helpers for Kani proofs over `BoxedUint`s of at most 128 bits.
#[cfg(kani)]
pub(crate) mod kani_util {
    use super::BoxedUint;
    use crate::{Limb, Word};

    /// Number of limbs in 64 bits.
    pub(crate) const L64: usize = (u64::BITS / Limb::BITS) as usize;

    /// An arbitrary value with exactly `N` limbs. The limb count is concrete: symbolic `Vec`
    /// lengths make Kani's heap model blow up.
    pub(crate) fn any_boxed<const N: usize>() -> BoxedUint {
        BoxedUint::from_words(kani::any::<[Word; N]>())
    }

    /// The value of `x`, which must have at most 128 bits of precision.
    pub(crate) fn value(x: &BoxedUint) -> u128 {
        assert!(x.bits_precision() <= u128::BITS);
        let mut acc = 0;
        for (i, limb) in x.as_limbs().iter().enumerate() {
            acc |= u128::from(limb.0) << (i as u32 * Limb::BITS);
        }
        acc
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::kani_util::{L64, any_boxed, value};
    use crate::{BoxedUint, CheckedSub, CtOption, Limb, NonZero, Odd, One, Resize, Word, Zero};

    fn check_nonzero(result: CtOption<NonZero<BoxedUint>>, n: u128) {
        assert_eq!(result.is_some().to_bool(), n != 0);
        let inner = result.as_inner_unchecked().as_ref();
        assert_eq!(inner.bits_precision(), 128);
        // Even an absent CtOption must contain a valid wrapper, as its constructors promise.
        assert_ne!(value(inner), 0);
        if n != 0 {
            assert_eq!(value(inner), n);
        }
    }

    fn check_odd(result: CtOption<Odd<BoxedUint>>, n: u128) {
        assert_eq!(result.is_some().to_bool(), n & 1 == 1);
        let inner = result.as_inner_unchecked().as_ref();
        assert_eq!(inner.bits_precision(), 128);
        assert_eq!(value(inner) & 1, 1);
        if n & 1 == 1 {
            assert_eq!(value(inner), n);
        }
    }

    /// Every u128: Boxed conversion presence, successful values, and absent payload invariants.
    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_fallible_wrappers_native() {
        let input = any_boxed::<{ 2 * L64 }>();
        let n = value(&input);
        check_nonzero(input.to_nz(), n);
        check_nonzero(input.clone().into_nz(), n);
        check_odd(input.to_odd(), n);
        check_odd(input.clone().into_odd(), n);
        let borrowed = input.as_nz_vartime();
        assert_eq!(borrowed.is_some(), n != 0);
        if let Some(nonzero) = borrowed {
            assert_eq!(value(nonzero.as_ref()), n);
            assert_eq!(nonzero.as_ref().bits_precision(), 128);
        }
        let borrowed = input.as_odd_vartime();
        assert_eq!(borrowed.is_some(), n & 1 == 1);
        if let Some(odd) = borrowed {
            assert_eq!(value(odd.as_ref()), n);
            assert_eq!(odd.as_ref().bits_precision(), 128);
        }
    }

    /// Internal defaulting conversions return exactly one when the wrapper predicate fails.
    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_defaulting_wrappers_native() {
        let input = any_boxed::<{ 2 * L64 }>();
        let n = value(&input);
        let (nonzero, present) = input.to_nz_or_one();
        assert_eq!(present.to_bool(), n != 0);
        assert_eq!(value(nonzero.as_ref()), if n == 0 { 1 } else { n });
        assert_eq!(nonzero.as_ref().bits_precision(), 128);
        let (odd, present) = input.to_odd_or_one();
        assert_eq!(present.to_bool(), n & 1 == 1);
        assert_eq!(value(odd.as_ref()), if n & 1 == 1 { n } else { 1 });
        assert_eq!(odd.as_ref().bits_precision(), 128);
    }

    /// Identity constructors and mutation preserve arbitrary u128 storage precision.
    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_identity_mutation_native() {
        let mut input = any_boxed::<{ 2 * L64 }>();
        let one = BoxedUint::one_like(&input);
        assert_eq!((value(&one), one.bits_precision()), (1, 128));
        assert_eq!(input.is_one().to_bool(), value(&input) == 1);
        num_traits::One::set_one(&mut input);
        assert_eq!((value(&input), input.bits_precision()), (1, 128));
        assert!(One::is_one(&input).to_bool());
        assert!(num_traits::One::is_one(&input));
        num_traits::Zero::set_zero(&mut input);
        assert_eq!((value(&input), input.bits_precision()), (0, 128));
        assert!(Zero::is_zero(&input).to_bool());
        assert!(num_traits::Zero::is_zero(&input));
    }

    fn resized_precision(bits: u32) -> u32 {
        bits.div_ceil(Word::BITS).max(1) * Word::BITS
    }

    fn check_precision_constructors<const BITS: u32>() {
        let precision = resized_precision(BITS);
        let zero = BoxedUint::zero_with_precision(BITS);
        assert_eq!(zero.bits_precision(), precision);
        for limb in zero.as_limbs() {
            assert_eq!(limb.0, 0);
        }
        let one = BoxedUint::one_with_precision(BITS);
        assert_eq!(one.bits_precision(), precision);
        assert_eq!(one.as_limbs()[0].0, 1);
        for limb in &one.as_limbs()[1..] {
            assert_eq!(limb.0, 0);
        }
        if BITS > 0 {
            let maximum = BoxedUint::max(BITS);
            assert_eq!(maximum.bits_precision(), precision);
            for limb in maximum.as_limbs() {
                assert_eq!(limb.0, Word::MAX);
            }
        }
    }

    /// These boundary requests are bounded coverage; max(0)'s doc mismatch is a separate regression.
    // Symbolic requests and an exhaustive loop over all 129 requests timed out.
    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_precision_constructor_boundaries() {
        check_precision_constructors::<0>();
        check_precision_constructors::<1>();
        check_precision_constructors::<{ Word::BITS - 1 }>();
        check_precision_constructors::<{ Word::BITS }>();
        check_precision_constructors::<{ Word::BITS + 1 }>();
        check_precision_constructors::<64>();
        check_precision_constructors::<127>();
        check_precision_constructors::<128>();
    }

    fn check_unchecked_resize<const BITS: u32>() {
        let input = any_boxed::<{ 2 * L64 }>();
        let precision = resized_precision(BITS);
        let mask = u128::MAX >> (u128::BITS - precision);
        // A zero request discards all words before the empty vector normalizes to one zero limb.
        let expected = if BITS == 0 { 0 } else { value(&input) & mask };
        let mut direct = input.clone();
        direct.resize_in_place_unchecked(BITS);
        assert_eq!(
            (value(&direct), direct.bits_precision()),
            (expected, precision)
        );
        let result = input.resize_unchecked(BITS);
        assert_eq!(
            (value(&result), result.bits_precision()),
            (expected, precision)
        );
    }

    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_resize_unchecked_boundaries() {
        check_unchecked_resize::<0>();
        check_unchecked_resize::<63>();
        check_unchecked_resize::<64>();
        check_unchecked_resize::<65>();
        check_unchecked_resize::<127>();
        check_unchecked_resize::<128>();
    }

    fn check_checked_resize<const BITS: u32>() {
        let input = any_boxed::<{ 2 * L64 }>();
        let n = value(&input);
        let mask = if BITS == 0 {
            0
        } else {
            u128::MAX >> (u128::BITS - BITS)
        };
        let fits = n <= mask;
        let result = input.clone().try_resize(BITS);
        assert_eq!(result.is_some(), fits);
        if fits {
            let result = result.unwrap();
            assert_eq!(
                (value(&result), result.bits_precision()),
                (n, resized_precision(BITS))
            );
            let result = input.resize(BITS);
            assert_eq!(
                (value(&result), result.bits_precision()),
                (n, resized_precision(BITS))
            );
        }
    }

    #[kani::proof]
    #[kani::unwind(8)]
    fn boxed_resize_checked_boundaries() {
        check_checked_resize::<0>();
        check_checked_resize::<63>();
        check_checked_resize::<64>();
        check_checked_resize::<65>();
        check_checked_resize::<127>();
        check_checked_resize::<128>();
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn boxed_is_within_bits_native() {
        let input = any_boxed::<{ 2 * L64 }>();
        let bits: u32 = kani::any();
        let expected = if bits >= u128::BITS {
            true
        } else if bits == 0 {
            value(&input) == 0
        } else {
            value(&input) <= u128::MAX >> (u128::BITS - bits)
        };
        assert_eq!(input.is_within_bits(bits), expected);
    }

    /// `|=` with a wider rhs (fixed in 3f7dede) widens `self` and ORs the zero-extended values.
    #[kani::proof]
    #[kani::unwind(5)]
    fn boxed_bitor_assign_wider_rhs() {
        let (mut x, wide) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let expected = value(&x) | value(&wide);
        x |= &wide;
        assert_eq!((value(&x), x.bits_precision()), (expected, 128));
    }

    /// Binary `|`, `&` and `^` between a 64-bit and a 128-bit value act on the zero-extended
    /// values and produce the wider precision.
    #[kani::proof]
    #[kani::unwind(5)]
    fn boxed_bit_ops_mixed_width() {
        let (narrow, wide) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let (vn, vw) = (value(&narrow), value(&wide));

        let res = narrow.bitor(&wide);
        assert_eq!((value(&res), res.bits_precision()), (vn | vw, 128));
        let res = wide.bitand(&narrow);
        assert_eq!((value(&res), res.bits_precision()), (vn & vw, 128));
        let res = narrow.bitxor(&wide);
        assert_eq!((value(&res), res.bits_precision()), (vn ^ vw, 128));
    }

    /// Adding a 128-bit rhs to a 64-bit `self`: `carrying_add` widens to 128 bits, while
    /// `overflowing_add` and `wrapping_add` keep 64 bits, and `overflowing_add` also reports a
    /// rhs that does not fit.
    #[kani::proof]
    #[kani::unwind(5)]
    fn boxed_add_wider_rhs() {
        let (narrow, wide) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let (vn, vw) = (value(&narrow), value(&wide));

        let (res, carry) = narrow.carrying_add(&wide, Limb::ZERO);
        let (sum, overflow) = vn.overflowing_add(vw);
        assert_eq!((value(&res), res.bits_precision()), (sum, 128));
        assert_eq!(carry.0, Word::from(overflow));

        let sum64 = vn + (vw & u128::from(u64::MAX));
        let (res, overflow) = narrow.overflowing_add(&wide);
        assert_eq!(
            (value(&res), res.bits_precision()),
            (sum64 as u64 as u128, 64)
        );
        assert_eq!(overflow.to_bool(), sum64 >> 64 != 0 || vw >> 64 != 0);
        let res = narrow.wrapping_add(&wide);
        assert_eq!(
            (value(&res), res.bits_precision()),
            (sum64 as u64 as u128, 64)
        );
    }

    /// Subtracting a 128-bit rhs from a 64-bit `self`: `borrowing_sub` widens to 128 bits, while
    /// `underflowing_sub`, `wrapping_sub` and `checked_sub` keep 64 bits and report underflow
    /// exactly when `rhs > self`.
    #[kani::proof]
    #[kani::unwind(5)]
    fn boxed_sub_wider_rhs() {
        let (narrow, wide) = (any_boxed::<L64>(), any_boxed::<{ 2 * L64 }>());
        let (vn, vw) = (value(&narrow), value(&wide));

        let (res, borrow) = narrow.borrowing_sub(&wide, Limb::ZERO);
        assert_eq!(
            (value(&res), res.bits_precision()),
            (vn.wrapping_sub(vw), 128)
        );
        assert_eq!(borrow.0, if vw > vn { Word::MAX } else { 0 });

        let diff64 = u128::from((vn as u64).wrapping_sub(vw as u64));
        let (res, underflow) = narrow.underflowing_sub(&wide);
        assert_eq!((value(&res), res.bits_precision()), (diff64, 64));
        assert_eq!(underflow.to_bool(), vw > vn);
        let res = narrow.wrapping_sub(&wide);
        assert_eq!((value(&res), res.bits_precision()), (diff64, 64));
        let res = Option::<BoxedUint>::from(narrow.checked_sub(&wide));
        assert_eq!(res.map(|r| value(&r)), vn.checked_sub(vw));
    }
}
