use super::UintRef;
use crate::{Choice, Limb};

impl UintRef {
    /// Perform an in-place carrying add of a limb, returning the carried limb value.
    #[inline]
    pub const fn add_assign_limb(&mut self, mut rhs: Limb) -> Limb {
        let mut i = 0;
        while i < self.limbs.len() {
            (self.limbs[i], rhs) = self.limbs[i].overflowing_add(rhs);
            i += 1;
        }
        rhs
    }

    /// Perform an in-place carrying add of another [`UintRef`], returning the carried limb value.
    ///
    /// # Panics
    /// If `self` is shorter than `rhs`.
    #[inline]
    #[track_caller]
    pub const fn carrying_add_assign(&mut self, rhs: &Self, carry: Limb) -> Limb {
        self.carrying_add_assign_slice(&rhs.limbs, carry)
    }

    /// Perform an in-place carrying add of another limb slice, returning the carried limb value.
    ///
    /// # Panics
    /// If `self` is shorter than `rhs`.
    #[inline]
    #[track_caller]
    pub const fn carrying_add_assign_slice(&mut self, rhs: &[Limb], mut carry: Limb) -> Limb {
        assert!(
            self.limbs.len() >= rhs.len(),
            "length mismatch in carrying_add_assign_slice"
        );
        let mut i = 0;
        while i < rhs.len() {
            (self.limbs[i], carry) = self.limbs[i].carrying_add(rhs[i], carry);
            i += 1;
        }
        while i < self.limbs.len() {
            (self.limbs[i], carry) = self.limbs[i].overflowing_add(carry);
            i += 1;
        }
        carry
    }

    /// Perform an in-place carrying add of another limb slice, returning the carried limb value.
    ///
    /// # Panics
    /// If `self` is shorter than `rhs`.
    #[inline]
    #[track_caller]
    pub const fn conditional_add_assign(
        &mut self,
        rhs: &Self,
        carry: Limb,
        choice: Choice,
    ) -> Limb {
        self.conditional_add_assign_slice(rhs.as_limbs(), carry, choice)
    }

    /// Perform an in-place carrying add of another limb slice, returning the carried limb value.
    ///
    /// # Panics
    /// If `self` is shorter than `rhs`.
    #[inline]
    #[track_caller]
    pub const fn conditional_add_assign_slice(
        &mut self,
        rhs: &[Limb],
        mut carry: Limb,
        choice: Choice,
    ) -> Limb {
        assert!(
            self.limbs.len() >= rhs.len(),
            "length mismatch in conditional_add_assign_slice"
        );
        let mut i = 0;
        while i < rhs.len() {
            (self.limbs[i], carry) =
                self.limbs[i].carrying_add(Limb::select(Limb::ZERO, rhs[i], choice), carry);
            i += 1;
        }
        while i < self.limbs.len() {
            (self.limbs[i], carry) = self.limbs[i].overflowing_add(carry);
            i += 1;
        }
        carry
    }

    /// Perform an in-place carrying add of another [`UintRef`] multiplied by
    /// a limb, returning the carried limb value.
    ///
    /// # Panics
    /// If `self` is shorter than `rhs`.
    #[inline]
    #[track_caller]
    pub const fn carrying_add_assign_mul_limb(
        &mut self,
        rhs: &Self,
        rhs_mul: Limb,
        mut carry: Limb,
    ) -> Limb {
        assert!(
            self.limbs.len() >= rhs.limbs.len(),
            "length mismatch in carrying_add_assign_mul_limb"
        );
        let mut i = 0;
        while i < rhs.limbs.len() {
            (self.limbs[i], carry) = rhs.limbs[i].carrying_mul_add(rhs_mul, self.limbs[i], carry);
            i += 1;
        }
        self.trailing_mut(i).add_assign_limb(carry)
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{Choice, Limb, UintRef, WideWord, Word};

    /// Includes empty slices, arbitrary incoming carry, and untouched backing limbs.
    #[kani::proof]
    #[kani::unwind(6)]
    fn uint_ref_carrying_add_mixed_lengths() {
        let original: [Word; 4] = kani::any();
        let rhs: [Word; 4] = kani::any();
        let (len, rhs_len): (usize, usize) = kani::any();
        kani::assume(rhs_len <= len && len <= 4);
        let incoming: Word = kani::any();
        let enabled: bool = kani::any();
        let mut plain = original.map(Limb);
        let mut conditional = plain;
        let rhs_limbs = rhs.map(Limb);
        let carry = UintRef::new_mut(&mut plain[..len])
            .carrying_add_assign(UintRef::new(&rhs_limbs[..rhs_len]), Limb(incoming));
        let conditional_carry = UintRef::new_mut(&mut conditional[..len])
            .conditional_add_assign_slice(
                &rhs_limbs[..rhs_len],
                Limb(incoming),
                Choice::from_u8_lsb(enabled as u8),
            );
        let mut expected_carry = incoming as WideWord;
        let mut expected_conditional_carry = incoming as WideWord;
        for i in 0..4 {
            if i < len {
                let rhs_word = if i < rhs_len { rhs[i] } else { 0 };
                let sum = original[i] as WideWord + rhs_word as WideWord + expected_carry;
                let conditional_sum = original[i] as WideWord
                    + if enabled { rhs_word as WideWord } else { 0 }
                    + expected_conditional_carry;
                assert_eq!(plain[i].0, sum as Word);
                assert_eq!(conditional[i].0, conditional_sum as Word);
                expected_carry = sum >> Word::BITS;
                expected_conditional_carry = conditional_sum >> Word::BITS;
            } else {
                assert_eq!(plain[i].0, original[i]);
                assert_eq!(conditional[i].0, original[i]);
            }
        }
        assert_eq!(carry.0 as WideWord, expected_carry);
        assert_eq!(conditional_carry.0 as WideWord, expected_conditional_carry);
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn uint_ref_add_assign_limb_mixed_lengths() {
        let original: [Word; 4] = kani::any();
        let rhs: Word = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= 4);
        let mut limbs = original.map(Limb);
        let carry = UintRef::new_mut(&mut limbs[..len]).add_assign_limb(Limb(rhs));
        let mut expected_carry = rhs as WideWord;
        for i in 0..4 {
            if i < len {
                let sum = original[i] as WideWord + expected_carry;
                assert_eq!(limbs[i].0, sum as Word);
                expected_carry = sum >> Word::BITS;
            } else {
                assert_eq!(limbs[i].0, original[i]);
            }
        }
        assert_eq!(carry.0 as WideWord, expected_carry);
    }

    /// A full-word multiplier, addend, and incoming carry, with propagation into a spare limb.
    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(4)]
    fn uint_ref_carrying_add_mul_limb_two_words() {
        let original: [Word; 2] = kani::any();
        let (rhs, multiplier, incoming): (Word, Word, Word) = kani::any();
        let mut out = [Limb(original[0]), Limb(original[1])];
        let carry = UintRef::new_mut(&mut out).carrying_add_assign_mul_limb(
            UintRef::new(&[Limb(rhs)]),
            Limb(multiplier),
            Limb(incoming),
        );
        let low = original[0] as WideWord
            + rhs as WideWord * multiplier as WideWord
            + incoming as WideWord;
        let high = original[1] as WideWord + (low >> Word::BITS);
        assert_eq!(out[0].0, low as Word);
        assert_eq!(out[1].0, high as Word);
        assert_eq!(carry.0, (high >> Word::BITS) as Word);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Choice, Limb, UintRef, Unsigned};

    #[test]
    fn carrying_add_assign_mixed() {
        let mut a = [Limb::MAX];
        let carry =
            UintRef::new_mut(&mut a).carrying_add_assign(Limb::ONE.as_uint_ref(), Limb::ZERO);
        assert_eq!((a, carry), ([Limb::ZERO], Limb::ONE));

        let mut a = [Limb::MAX];
        let carry = UintRef::new_mut(&mut a).conditional_add_assign(
            Limb::ONE.as_uint_ref(),
            Limb::ZERO,
            Choice::FALSE,
        );
        assert_eq!((a, carry), ([Limb::MAX], Limb::ZERO));

        let mut a = [Limb::MAX];
        let carry = UintRef::new_mut(&mut a).conditional_add_assign(
            Limb::ONE.as_uint_ref(),
            Limb::ZERO,
            Choice::TRUE,
        );
        assert_eq!((a, carry), ([Limb::ZERO], Limb::ONE));

        let mut a = [Limb::MAX, Limb::MAX];
        let carry =
            UintRef::new_mut(&mut a).carrying_add_assign(Limb::ZERO.as_uint_ref(), Limb::ONE);
        assert_eq!((a, carry), ([Limb::ZERO, Limb::ZERO], Limb::ONE));

        let mut a = [Limb::MAX, Limb::MAX];
        let carry = UintRef::new_mut(&mut a).conditional_add_assign(
            Limb::MAX.as_uint_ref(),
            Limb::ONE,
            Choice::TRUE,
        );
        assert_eq!((a, carry), ([Limb::MAX, Limb::ZERO], Limb::ONE));
    }
}
