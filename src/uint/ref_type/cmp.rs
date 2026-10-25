//! [`UintRef`] comparison operations.
//!
//! Constant time unless explicitly noted otherwise.

use core::{cmp::Ordering, mem::transmute};

use super::UintRef;
use crate::{Choice, CtEq, Limb, word};

impl UintRef {
    /// Returns the truthy value if `self` is odd or the falsy value otherwise.
    #[inline]
    #[must_use]
    pub const fn is_odd(&self) -> Choice {
        debug_assert!(self.nlimbs() >= 1, "should have limbs");
        word::choice_from_lsb(self.limbs[0].0 & 1)
    }

    /// Returns [`Choice::TRUE`] if `self` != `0` or [`Choice::FALSE`] otherwise.
    #[inline]
    #[must_use]
    pub const fn is_nonzero(&self) -> Choice {
        let mut b = 0;
        let mut i = 0;
        while i < self.nlimbs() {
            b |= self.limbs[i].0;
            i += 1;
        }
        Limb(b).is_nonzero()
    }

    /// Are all of limbs equal to `0`?
    #[inline]
    #[must_use]
    pub const fn is_zero(&self) -> Choice {
        self.is_nonzero().not()
    }

    /// Determine in variable time whether the `self` is zero.
    #[inline]
    pub(crate) const fn is_zero_vartime(&self) -> bool {
        let mut i = 0;
        while i < self.nlimbs() {
            if self.limbs[i].0 != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    /// Returns the Ordering between `lhs` and `rhs`.
    #[allow(clippy::cast_possible_wrap)]
    #[inline(always)]
    pub(crate) const fn cmp(lhs: &Self, rhs: &Self) -> Ordering {
        let overlap = if lhs.nlimbs() < rhs.nlimbs() {
            lhs.nlimbs()
        } else {
            rhs.nlimbs()
        };

        let mut borrow = Limb::ZERO;
        let mut diff = Limb::ZERO;
        let mut i = 0;

        while i < overlap {
            let (w, b) = rhs.limbs[i].borrowing_sub(lhs.limbs[i], borrow);
            diff = diff.bitor(w);
            borrow = b;
            i += 1;
        }
        let lesser = rhs.trailing(overlap).is_nonzero();
        let greater = lhs.trailing(overlap).is_nonzero();

        // FIXME cast_signed() is stable in Rust 1.87
        let sgn = borrow
            .lsb_to_choice()
            .and(lesser.not())
            .or(greater)
            .select_u8(255, 1) as i8;
        let ord = (diff.is_nonzero().or(lesser).or(greater).to_u8() as i8) * sgn;

        #[allow(unsafe_code)]
        // SAFETY: Ordering is repr(i8)
        unsafe {
            transmute(ord)
        }
    }

    /// Returns the Ordering between `self` and `rhs` in variable time.
    #[inline(always)]
    #[must_use]
    pub const fn cmp_vartime(&self, rhs: &Self) -> Ordering {
        if self.nlimbs() < rhs.nlimbs() {
            return rhs.cmp_vartime(self).reverse();
        }

        let mut i = self.nlimbs();
        while i > rhs.nlimbs() {
            i -= 1;
            if self.limbs[i].0 != 0 {
                return Ordering::Greater;
            }
        }
        while i > 0 {
            i -= 1;
            let (val, borrow) = self.limbs[i].borrowing_sub(rhs.limbs[i], Limb::ZERO);
            if val.0 != 0 {
                return if borrow.0 != 0 {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }
        }

        Ordering::Equal
    }

    /// Returns the truthy value if `lhs < rhs` and the falsy value otherwise.
    #[inline(always)]
    pub(crate) const fn lt(lhs: &Self, rhs: &Self) -> Choice {
        let overlap = if lhs.nlimbs() < rhs.nlimbs() {
            lhs.nlimbs()
        } else {
            rhs.nlimbs()
        };

        // NOTE: this is effectively a `borrowing_sub` that only computes the borrow
        let mut i = 0;
        let mut borrow = Limb::ZERO;

        while i < overlap {
            borrow = lhs.limbs[i].borrowing_sub(rhs.limbs[i], borrow).1;
            i += 1;
        }
        let lesser = rhs.trailing(overlap).is_nonzero();
        let greater = lhs.trailing(overlap).is_nonzero();
        borrow.lsb_to_choice().and(greater.not()).or(lesser)
    }
}

impl Eq for UintRef {}

impl Ord for UintRef {
    fn cmp(&self, other: &Self) -> Ordering {
        Self::cmp(self, other)
    }
}

impl PartialOrd for UintRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for UintRef {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;

    use super::UintRef;
    use crate::Limb;

    #[test]
    fn cmp() {
        fn check(a: &UintRef, b: &UintRef, ord: Ordering) {
            assert_eq!(UintRef::cmp(a, b), ord);
            assert_eq!(UintRef::cmp_vartime(a, b), ord);
            assert_eq!(a.cmp(b), ord);
            if ord == Ordering::Equal {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }

        let z_small = UintRef::new(&[Limb::ZERO, Limb::ZERO]);
        let z_big = UintRef::new(&[Limb::ZERO, Limb::ZERO, Limb::ZERO]);
        let a = UintRef::new(&[Limb::ZERO, Limb::ZERO, Limb::ONE]);
        let b = UintRef::new(&[Limb::ONE, Limb::ZERO]);

        check(z_small, z_big, Ordering::Equal);
        check(z_big, z_small, Ordering::Equal);
        check(z_small, a, Ordering::Less);
        check(z_big, a, Ordering::Less);
        check(a, z_small, Ordering::Greater);
        check(a, z_big, Ordering::Greater);
        check(a, b, Ordering::Greater);
        check(b, a, Ordering::Less);
    }

    #[test]
    fn lt() {
        fn check(a: &UintRef, b: &UintRef) {
            assert!(UintRef::lt(a, b).to_bool());
            assert!(!UintRef::lt(b, a).to_bool());
            assert!(a < b);
            assert!(b > a);
        }

        let lesser = UintRef::new(&[Limb::ZERO, Limb::ZERO, Limb::ZERO]);
        let greater = UintRef::new(&[Limb::ZERO, Limb::ONE, Limb::ZERO]);
        check(lesser, greater);

        let smaller = UintRef::new(&[Limb::ZERO, Limb::ZERO]);
        let bigger = UintRef::new(&[Limb::ZERO, Limb::ZERO, Limb::ONE]);
        check(smaller, bigger);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::UintRef;
    use crate::uint::ref_type::kani_util::value;
    use crate::{CtEq, Limb, U64, U128, Word};

    /// Independent active lengths include empty views and redundant high zero words.
    #[kani::proof]
    #[kani::unwind(6)]
    fn uint_ref_ordering_mixed_lengths() {
        let (a, b): ([Word; 2], [Word; 2]) = kani::any();
        let (a_len, b_len): (usize, usize) = kani::any();
        kani::assume(a_len <= 2 && b_len <= 2);
        let (a, b) = (a.map(Limb), b.map(Limb));
        let (a, b) = (UintRef::new(&a[..a_len]), UintRef::new(&b[..b_len]));
        let (va, vb) = (value(a.as_limbs()), value(b.as_limbs()));
        let expected = va.cmp(&vb);
        assert_eq!(UintRef::cmp(a, b), expected);
        assert_eq!(UintRef::cmp(b, a), expected.reverse());
        assert_eq!(a.cmp_vartime(b), expected);
        assert_eq!(b.cmp_vartime(a), expected.reverse());
        assert_eq!(Ord::cmp(a, b), expected);
        assert_eq!(a.partial_cmp(b), Some(expected));
        assert_eq!(a == b, va == vb);
        assert_eq!(a < b, va < vb);
        assert_eq!(a > b, va > vb);
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn uint_ref_predicates_and_borrowed_wrappers() {
        let words: [Word; 2] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= 2);
        let limbs = words.map(Limb);
        let input = UintRef::new(&limbs[..len]);
        let n = value(input.as_limbs());
        assert_eq!(input.is_zero().to_bool(), n == 0);
        assert_eq!(input.is_zero_vartime(), n == 0);
        assert_eq!(input.is_nonzero().to_bool(), n != 0);
        let nonzero = input.as_nz_vartime();
        assert_eq!(nonzero.is_some(), n != 0);
        if let Some(nonzero) = nonzero {
            assert_eq!(value(nonzero.as_ref().as_limbs()), n);
            assert_eq!(nonzero.as_ref().nlimbs(), len);
        }
        // Oddness and lowest_u64 access the first limb, so require a nonempty view.
        if len > 0 {
            assert_eq!(input.is_odd().to_bool(), n & 1 == 1);
            assert_eq!(input.lowest_u64(), n as u64);
            let odd = input.as_odd_vartime();
            assert_eq!(odd.is_some(), n & 1 == 1);
            if let Some(odd) = odd {
                assert_eq!(value(odd.as_ref().as_limbs()), n);
                assert_eq!(odd.as_ref().nlimbs(), len);
            }
        }
    }

    /// Comparisons between a 128-bit and a 64-bit `UintRef` agree with `u128`, in both orders.
    #[kani::proof]
    #[kani::unwind(5)]
    fn uintref_cmp_mixed_width_correct() {
        let (a, b): (u128, u64) = kani::any();
        let (ua, ub) = (U128::from_u128(a), U64::from_u64(b));
        let (ra, rb) = (ua.as_uint_ref(), ub.as_uint_ref());
        let b = u128::from(b);

        assert_eq!(UintRef::cmp(ra, rb), a.cmp(&b));
        assert_eq!(UintRef::cmp(rb, ra), b.cmp(&a));
        assert_eq!(ra.cmp_vartime(rb), a.cmp(&b));
        assert_eq!(rb.cmp_vartime(ra), b.cmp(&a));
        assert_eq!(UintRef::lt(ra, rb).to_bool(), a < b);
        assert_eq!(UintRef::lt(rb, ra).to_bool(), b < a);
        assert_eq!(ra.ct_eq(rb).to_bool(), a == b);
        assert_eq!(rb.ct_eq(ra).to_bool(), a == b);
        assert_eq!(ua.ct_eq(&ub).to_bool(), a == b);
    }
}
