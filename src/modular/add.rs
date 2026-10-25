use crate::{Odd, Uint};

pub(crate) const fn add_montgomery_form<const LIMBS: usize>(
    a: &Uint<LIMBS>,
    b: &Uint<LIMBS>,
    modulus: &Odd<Uint<LIMBS>>,
) -> Uint<LIMBS> {
    a.add_mod(b, modulus.as_nz_ref())
}

pub(crate) const fn double_montgomery_form<const LIMBS: usize>(
    a: &Uint<LIMBS>,
    modulus: &Odd<Uint<LIMBS>>,
) -> Uint<LIMBS> {
    a.double_mod(modulus.as_nz_ref())
}

#[cfg(kani)]
mod kani_proofs {
    use super::{add_montgomery_form, double_montgomery_form};
    use crate::{Limb, Odd, U64, Word};

    /// Reduced 64-bit Montgomery representatives add canonically, including an overflowing sum.
    #[kani::proof]
    #[kani::unwind(3)]
    fn montgomery_add_u64_reduced() {
        let (a, b, m): (u64, u64, u64) = kani::any();
        kani::assume(m & 1 == 1);
        kani::assume(a < m && b < m);
        let sum = u128::from(a) + u128::from(b);
        kani::cover!(sum > u128::from(u64::MAX), "sum carries beyond 64 bits");
        let expected = if sum >= u128::from(m) {
            sum - u128::from(m)
        } else {
            sum
        };
        let a = U64::from_u64(a);
        let b = U64::from_u64(b);
        let modulus = Odd::new(U64::from_u64(m)).unwrap();
        let (low, carry) = a.carrying_add(&b, Limb::ZERO);
        assert_eq!(u64::from(low), sum as u64);
        assert_eq!(carry.0, (sum >> 64) as Word);
        let got = u64::from(add_montgomery_form(&a, &b, &modulus));
        assert_eq!(u128::from(got), expected);
        assert!(got < m);
    }

    /// Doubling a reduced 64-bit Montgomery representative returns the canonical result.
    #[kani::proof]
    #[kani::unwind(3)]
    fn montgomery_double_u64_reduced() {
        let (a, m): (u64, u64) = kani::any();
        kani::assume(m & 1 == 1);
        kani::assume(a < m);
        let sum = u128::from(a) * 2;
        kani::cover!(
            sum > u128::from(u64::MAX),
            "doubling carries beyond 64 bits"
        );
        let expected = if sum >= u128::from(m) {
            sum - u128::from(m)
        } else {
            sum
        };
        let a = U64::from_u64(a);
        let (low, carry) = a.shl1_with_carry(Limb::ZERO);
        assert_eq!(u64::from(low), sum as u64);
        assert_eq!(carry.0, (sum >> 64) as Word);
        let modulus = Odd::new(U64::from_u64(m)).unwrap();
        let got = u64::from(double_montgomery_form(&a, &modulus));
        assert_eq!(u128::from(got), expected);
        assert!(got < m);
    }
}
