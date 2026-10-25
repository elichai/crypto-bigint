use crate::{Odd, Uint};

pub(crate) const fn sub_montgomery_form<const LIMBS: usize>(
    a: &Uint<LIMBS>,
    b: &Uint<LIMBS>,
    modulus: &Odd<Uint<LIMBS>>,
) -> Uint<LIMBS> {
    a.sub_mod(b, modulus.as_nz_ref())
}

#[cfg(kani)]
mod kani_proofs {
    use super::sub_montgomery_form;
    use crate::{Limb, Odd, U64, Word};

    /// Reduced 64-bit Montgomery representatives subtract canonically in either borrow case.
    #[kani::proof]
    #[kani::unwind(3)]
    fn montgomery_sub_u64_reduced() {
        let (a, b, m): (u64, u64, u64) = kani::any();
        kani::assume(m & 1 == 1);
        kani::assume(a < m && b < m);
        kani::cover!(a < b, "subtraction borrows beyond 64 bits");
        let expected = if a >= b { a - b } else { m - (b - a) };
        let a_uint = U64::from_u64(a);
        let b_uint = U64::from_u64(b);
        let (low, borrow) = a_uint.borrowing_sub(&b_uint, Limb::ZERO);
        assert_eq!(u64::from(low), a.wrapping_sub(b));
        assert_eq!(borrow.0, if a < b { Word::MAX } else { 0 });
        let modulus = Odd::new(U64::from_u64(m)).unwrap();
        let got = u64::from(sub_montgomery_form(&a_uint, &b_uint, &modulus));
        assert_eq!(got, expected);
        assert!(got < m);
    }
}
