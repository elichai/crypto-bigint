use super::UintRef;
use crate::{Choice, Limb};

impl UintRef {
    /// Copy the contents from a [`UintRef`].
    ///
    /// # Panics
    /// If `self.nlimbs() != rhs.nlimbs()`
    #[inline(always)]
    #[track_caller]
    pub const fn copy_from(&mut self, rhs: &UintRef) {
        self.copy_from_slice(&rhs.limbs);
    }

    /// Conditionally copy the contents from a [`UintRef`].
    ///
    /// # Panics
    /// If `self.nlimbs() != rhs.nlimbs()`
    #[inline(always)]
    #[track_caller]
    pub const fn conditional_copy_from(&mut self, rhs: &UintRef, copy: Choice) {
        self.conditional_copy_from_slice(&rhs.limbs, copy);
    }

    /// Copy the contents from a limb slice.
    ///
    /// # Panics
    /// If `self.nlimbs() != limbs.len()`
    #[inline(always)]
    #[track_caller]
    pub const fn copy_from_slice(&mut self, limbs: &[Limb]) {
        // TODO core::slice::copy_from_slice should eventually be const
        assert!(self.limbs.len() == limbs.len(), "length mismatch");
        let mut i = 0;
        while i < self.limbs.len() {
            self.limbs[i] = limbs[i];
            i += 1;
        }
    }

    /// Conditionally copy the contents from a limb slice.
    ///
    /// # Panics
    /// If `self.nlimbs() != rhs.nlimbs()`
    #[inline(always)]
    #[track_caller]
    pub const fn conditional_copy_from_slice(&mut self, limbs: &[Limb], copy: Choice) {
        assert!(self.limbs.len() == limbs.len(), "length mismatch");
        let mut i = 0;
        while i < self.limbs.len() {
            self.limbs[i] = Limb::select(self.limbs[i], limbs[i], copy);
            i += 1;
        }
    }

    /// Fill the limb slice with a repeated limb value.
    #[inline(always)]
    pub const fn fill(&mut self, limb: Limb) {
        let mut i = 0;
        while i < self.limbs.len() {
            self.limbs[i] = limb;
            i += 1;
        }
    }

    /// Split the limb slice at a fixed position, producing head and tail slices.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn split_at(&self, mid: usize) -> (&Self, &Self) {
        let (a, b) = self.limbs.split_at(mid);
        (UintRef::new(a), UintRef::new(b))
    }

    /// Split the mutable limb slice at index `mid`, producing head and tail slices.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn split_at_mut(&mut self, mid: usize) -> (&mut Self, &mut Self) {
        let (a, b) = self.limbs.split_at_mut(mid);
        (UintRef::new_mut(a), UintRef::new_mut(b))
    }

    /// Access a limb slice up to a number of elements `len`.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn leading(&self, len: usize) -> &Self {
        Self::new(self.limbs.split_at(len).0)
    }

    /// Access a mutable limb slice up to a number of elements `len`.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn leading_mut(&mut self, len: usize) -> &mut Self {
        Self::new_mut(self.limbs.split_at_mut(len).0)
    }

    /// Access a limb slice starting from the index `start`.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn trailing(&self, start: usize) -> &Self {
        Self::new(self.limbs.split_at(start).1)
    }

    /// Access a mutable limb slice starting from the index `start`.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn trailing_mut(&mut self, start: usize) -> &mut Self {
        Self::new_mut(self.limbs.split_at_mut(start).1)
    }

    /// Determine if the slice has zero limbs.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.limbs.is_empty()
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::UintRef;
    use crate::{Choice, Limb, Word};

    /// Every prefix length through four limbs, including zero, and both copy choices.
    #[kani::proof]
    #[kani::unwind(6)]
    fn uintref_copy_preserves_slice_boundaries() {
        let original: [Word; 4] = kani::any();
        let source: [Word; 4] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= original.len());
        let copy: bool = kani::any();
        let source = source.map(Limb);
        let mut destination = original.map(Limb);
        UintRef::new_mut(&mut destination[..len]).conditional_copy_from(
            UintRef::new(&source[..len]),
            Choice::from_u8_lsb(copy as u8),
        );
        for i in 0..original.len() {
            assert_eq!(
                destination[i],
                if copy && i < len {
                    source[i]
                } else {
                    Limb(original[i])
                }
            );
        }
        UintRef::new_mut(&mut destination[..len]).copy_from(UintRef::new(&source[..len]));
        for i in 0..original.len() {
            assert_eq!(
                destination[i],
                if i < len {
                    source[i]
                } else {
                    Limb(original[i])
                }
            );
        }
    }

    // Slice equality can lower to memcmp over four 64-bit words (32 bytes).
    #[kani::proof]
    #[kani::unwind(34)]
    fn uintref_split_views_preserve_boundaries() {
        let original: [Word; 4] = kani::any();
        let (len, mid): (usize, usize) = kani::any();
        kani::assume(mid <= len && len <= original.len());
        let (low, high): (Word, Word) = kani::any();
        let mut limbs = original.map(Limb);
        let value = UintRef::new(&limbs[..len]);
        let (head, tail) = value.split_at(mid);
        assert_eq!(head.as_words(), &original[..mid]);
        assert_eq!(tail.as_words(), &original[mid..len]);
        assert_eq!(value.leading(mid).as_words(), head.as_words());
        assert_eq!(value.trailing(mid).as_words(), tail.as_words());
        assert_eq!(value.is_empty(), len == 0);

        let (head, tail) = UintRef::new_mut(&mut limbs[..len]).split_at_mut(mid);
        head.fill(Limb(low));
        tail.fill(Limb(high));
        for i in 0..original.len() {
            let expected = if i < mid {
                low
            } else if i < len {
                high
            } else {
                original[i]
            };
            assert_eq!(limbs[i].0, expected);
        }
    }

    /// Flattening must preserve order and write through to the original nested arrays.
    #[kani::proof]
    #[kani::unwind(34)]
    fn uintref_flattened_mut_preserves_storage() {
        let original: [[Word; 2]; 2] = kani::any();
        let replacement: Word = kani::any();
        let index: usize = kani::any();
        kani::assume(index < 4);
        let mut limbs = original.map(|row| row.map(Limb));
        let flat = UintRef::new_flattened_mut(&mut limbs);
        assert_eq!(
            flat.as_words(),
            &[
                original[0][0],
                original[0][1],
                original[1][0],
                original[1][1]
            ]
        );
        flat.as_mut_words()[index] = replacement;
        for i in 0..4 {
            let expected = if i == index {
                replacement
            } else {
                original[i / 2][i % 2]
            };
            assert_eq!(limbs[i / 2][i % 2].0, expected);
        }
        let mut empty_rows: [[Limb; 0]; 2] = [[], []];
        assert!(UintRef::new_flattened_mut(&mut empty_rows).is_empty());
        let mut no_rows: [[Limb; 2]; 0] = [];
        assert!(UintRef::new_flattened_mut(&mut no_rows).is_empty());
    }

    #[kani::proof]
    #[kani::unwind(8)]
    fn uintref_conditional_constants_and_resize() {
        let original: [Word; 4] = kani::any();
        let (len, flag): (usize, bool) = kani::any();
        kani::assume(len >= 1 && len <= original.len());
        let choice = Choice::from_u8_lsb(flag as u8);
        let mut limbs = original.map(Limb);
        let value = UintRef::new_mut(&mut limbs[..len]);
        value.conditional_set_zero(choice);
        for i in 0..len {
            assert_eq!(value.as_words()[i], if flag { 0 } else { original[i] });
        }
        value.conditional_set_one(choice);
        for i in 0..len {
            assert_eq!(
                value.as_words()[i],
                if flag {
                    Word::from(i == 0)
                } else {
                    original[i]
                }
            );
        }
        value.conditional_set_max(choice);
        let narrow = value.to_uint_resize::<2>();
        let wide = value.to_uint_resize::<6>();
        for i in 0..6 {
            let expected = if i >= len {
                0
            } else if flag {
                Word::MAX
            } else {
                original[i]
            };
            assert_eq!(wide.as_words()[i], expected);
            if i < 2 {
                assert_eq!(narrow.as_words()[i], expected);
            }
        }
        for i in len..original.len() {
            assert_eq!(limbs[i].0, original[i]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UintRef;
    use crate::{Choice, Limb};

    #[test]
    #[should_panic(expected = "length mismatch")]
    fn copy_from_slice_rejects_mismatched_lengths() {
        let mut dst = [Limb::ZERO];
        UintRef::new_mut(&mut dst).copy_from_slice(&[Limb::ONE, Limb(2)]);
    }

    #[test]
    #[should_panic(expected = "length mismatch")]
    fn conditional_copy_from_slice_rejects_mismatched_lengths() {
        let mut dst = [Limb::ZERO];
        UintRef::new_mut(&mut dst).conditional_copy_from_slice(&[Limb::ONE, Limb(2)], Choice::TRUE);
    }
}
