//! Multiplication support for [`UintRef`].

use crate::{Choice, Limb, UintRef, uint::mul::karatsuba};

impl UintRef {
    /// Compute the wrapping product of `self` and `rhs`, placing the result into `out`
    /// and returning a `Choice` indicating whether overflow occurred.
    #[inline(always)]
    pub const fn overflowing_mul(&self, rhs: &UintRef, out: &mut UintRef) -> Choice {
        let carry = self.wrapping_mul(rhs, out);
        self.check_mul_overflow(rhs, carry.is_nonzero())
    }

    /// Compute the wrapping squaring of `self`, placing the result into `out`
    /// and returning a `Choice` indicating whether overflow occurred.
    #[inline(always)]
    pub const fn overflowing_square(&self, out: &mut UintRef) -> Choice {
        let carry = self.wrapping_square(out);
        self.check_square_overflow(carry.is_nonzero())
    }

    /// Compute the wrapping product of `self` and `rhs`, placing the result into `out`
    /// and returning a carry Limb.
    #[inline(always)]
    pub const fn wrapping_mul(&self, rhs: &UintRef, out: &mut UintRef) -> Limb {
        karatsuba::wrapping_mul(self, rhs, out, false)
    }

    /// Compute the wrapping squaring of `self`, placing the result into `out` and returning
    /// a carry Limb.
    #[inline(always)]
    pub const fn wrapping_square(&self, out: &mut UintRef) -> Limb {
        karatsuba::wrapping_square(self, out)
    }

    /// Determine whether overflow occurs during wrapped multiplication.
    ///
    /// We determine this by comparing limbs in `self[i=0..n]` and `rhs[j=0..m]`.
    /// Any combination where the sum of indexes `i + j >= n`, having `self[i] != 0`
    /// and `rhs[j] != 0` would cause an overflow. For efficiency, we OR all limbs in
    /// `rhs` that would apply to each limb in `self` in turn.
    #[inline(always)]
    pub(crate) const fn check_mul_overflow(&self, rhs: &UintRef, mut carry: Choice) -> Choice {
        let mut rhs_tail = Limb::ZERO;
        let mut i = 0;
        let mut j = self.nlimbs();
        let mut k = rhs.nlimbs().saturating_sub(1);
        while k > j {
            rhs_tail = rhs_tail.bitor(rhs.limbs[k]);
            k -= 1;
        }
        while i < self.nlimbs() {
            j = self.nlimbs() - i;
            if j < rhs.nlimbs() {
                rhs_tail = rhs_tail.bitor(rhs.limbs[j]);
                carry = carry.or(self.limbs[i].is_nonzero().and(rhs_tail.is_nonzero()));
            }
            i += 1;
        }
        carry
    }

    /// Determine whether overflow occurs during wrapped squaring.
    #[inline(always)]
    pub(crate) const fn check_square_overflow(&self, carry: Choice) -> Choice {
        self.check_mul_overflow(self, carry)
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{Limb, UintRef, WideWord, Word};

    /// Full-word inputs and zero-initialized one- or two-word output buffers.
    #[kani::proof]
    #[kani::unwind(5)]
    fn uint_ref_wrapping_mul_word_zeroed_output() {
        let (a, b): (Word, Word) = kani::any();
        let len: usize = kani::any();
        kani::assume(len == 1 || len == 2);
        let mut out = [Limb::ZERO; 2];
        let carry = UintRef::new(&[Limb(a)])
            .wrapping_mul(UintRef::new(&[Limb(b)]), UintRef::new_mut(&mut out[..len]));
        let product = a as WideWord * b as WideWord;
        assert_eq!(out[0].0, product as Word);
        assert_eq!(
            out[1].0,
            if len == 2 {
                (product >> Word::BITS) as Word
            } else {
                0
            }
        );
        assert_eq!(
            carry.0,
            if len == 1 {
                (product >> Word::BITS) as Word
            } else {
                0
            }
        );
    }

    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(5)]
    fn uint_ref_square_word_zeroed_output() {
        let a: Word = kani::any();
        let mut narrow = [Limb::ZERO];
        let mut wide = [Limb::ZERO; 2];
        let value = [Limb(a)];
        let value = UintRef::new(&value);
        let overflow = value.overflowing_square(UintRef::new_mut(&mut narrow));
        let carry = value.wrapping_square(UintRef::new_mut(&mut wide));
        let product = a as WideWord * a as WideWord;
        assert_eq!(narrow[0].0, product as Word);
        assert_eq!(wide[0].0, product as Word);
        assert_eq!(wide[1].0, (product >> Word::BITS) as Word);
        assert_eq!(overflow.to_bool(), product >> Word::BITS != 0);
        assert_eq!(carry, Limb::ZERO);
    }

    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(4)]
    fn uint_ref_overflowing_mul_word_zeroed_output() {
        let (a, b): (Word, Word) = kani::any();
        let mut out = [Limb::ZERO];
        let overflow = UintRef::new(&[Limb(a)])
            .overflowing_mul(UintRef::new(&[Limb(b)]), UintRef::new_mut(&mut out));
        let product = a as WideWord * b as WideWord;
        assert_eq!(out[0].0, product as Word);
        assert_eq!(overflow.to_bool(), product >> Word::BITS != 0);
    }
}

/// Desired public contracts; run separately with `cargo kani --tests --harness regression_uint_ref`.
#[cfg(all(test, kani))]
mod kani_regressions {
    use crate::{Limb, UintRef};

    #[kani::proof]
    #[kani::unwind(5)]
    fn regression_uint_ref_mul_overwrites_output() {
        let mut out = [Limb::ONE];
        UintRef::new(&[Limb::ONE])
            .wrapping_mul(UintRef::new(&[Limb::ONE]), UintRef::new_mut(&mut out));
        assert!(out[0].0 == 1);
    }

    #[kani::proof]
    #[kani::unwind(5)]
    fn regression_uint_ref_square_overwrites_output() {
        let mut out = [Limb::ONE];
        UintRef::new(&[Limb::ONE]).wrapping_square(UintRef::new_mut(&mut out));
        assert!(out[0].0 == 1);
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn regression_uint_ref_mul_overflow_uses_wide_output_width() {
        let value = [Limb::ZERO, Limb::ONE];
        let mut out = [Limb::ZERO; 4];
        let overflow =
            UintRef::new(&value).overflowing_mul(UintRef::new(&value), UintRef::new_mut(&mut out));
        assert_eq!(out, [Limb::ZERO, Limb::ZERO, Limb::ONE, Limb::ZERO]);
        assert!(!overflow.to_bool());
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn regression_uint_ref_mul_overflow_uses_narrow_output_width() {
        let mut out = [Limb::ZERO];
        let overflow = UintRef::new(&[Limb::ZERO, Limb::ONE])
            .overflowing_mul(UintRef::new(&[Limb::ONE]), UintRef::new_mut(&mut out));
        assert_eq!(out, [Limb::ZERO]);
        assert!(overflow.to_bool());
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn regression_uint_ref_square_overflow_uses_wide_output_width() {
        let mut out = [Limb::ZERO; 4];
        let overflow =
            UintRef::new(&[Limb::ZERO, Limb::ONE]).overflowing_square(UintRef::new_mut(&mut out));
        assert_eq!(out, [Limb::ZERO, Limb::ZERO, Limb::ONE, Limb::ZERO]);
        assert!(!overflow.to_bool());
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn regression_uint_ref_square_overflow_uses_narrow_output_width() {
        let mut out = [Limb::ZERO];
        let overflow = UintRef::new(&[Limb::ONE, Limb::ONE, Limb::ZERO])
            .overflowing_square(UintRef::new_mut(&mut out));
        assert_eq!(out, [Limb::ONE]);
        assert!(overflow.to_bool());
    }
}
