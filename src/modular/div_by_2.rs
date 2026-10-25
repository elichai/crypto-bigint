#[cfg(feature = "alloc")]
use crate::{BoxedUint, Integer};
use crate::{Limb, Odd, Uint};

pub(crate) const fn div_by_2<const LIMBS: usize>(
    a: &Uint<LIMBS>,
    modulus: &Odd<Uint<LIMBS>>,
) -> Uint<LIMBS> {
    // We are looking for such `b` that `b + b = a mod modulus`.
    // Two possibilities:
    // - if `a` is even, we can just divide by 2;
    // - if `a` is odd, we divide `(a + modulus)` by 2.

    // Note that this also works if `a` is a Montgomery representation modulo `modulus`
    // of some integer `x`.
    // If `b + b = a mod modulus` it means that `y + y = x mod modulus` where `y` is the integer
    // whose Montgomery representation is `b`.

    let is_odd = a.is_odd();
    let (if_odd, carry) = a.carrying_add(modulus.as_ref(), Limb::ZERO);
    let carry = Limb::select(Limb::ZERO, carry, is_odd);
    Uint::<LIMBS>::select(a, &if_odd, is_odd)
        .shr1()
        .set_bit(Uint::<LIMBS>::BITS - 1, carry.is_nonzero())
}

#[cfg(feature = "alloc")]
pub(crate) fn div_by_2_boxed(a: &BoxedUint, modulus: &Odd<BoxedUint>) -> BoxedUint {
    let mut result = a.clone();
    div_by_2_boxed_assign(&mut result, modulus);
    result
}

#[cfg(feature = "alloc")]
pub(crate) fn div_by_2_boxed_assign(a: &mut BoxedUint, modulus: &Odd<BoxedUint>) {
    debug_assert_eq!(a.bits_precision(), modulus.bits_precision());

    let is_odd = a.is_odd();
    let carry = a.conditional_carrying_add_assign(modulus, is_odd);
    a.shr1_assign();
    a.set_bit(a.bits_precision() - 1, carry);
}

#[cfg(kani)]
mod kani_proofs {
    use super::div_by_2;
    use crate::{Odd, U128};

    /// For odd `m` and `a < m`, the result `b` satisfies `b < m` and `2b ≡ a (mod m)`: it is
    /// `a / 2` for even `a` and `(a + m) / 2` (a 129-bit sum) for odd `a`.
    #[kani::proof]
    #[kani::unwind(5)]
    fn div_by_2_correct() {
        let (a, m): (u128, u128) = kani::any();
        kani::assume(m & 1 == 1 && a < m);

        let res = div_by_2(&U128::from_u128(a), &Odd::new(U128::from_u128(m)).unwrap());
        let expected = if a & 1 == 0 {
            a >> 1
        } else {
            let (sum, carry) = a.overflowing_add(m);
            (sum >> 1) | (u128::from(carry) << 127)
        };
        assert_eq!(u128::from(res), expected);
        assert!(expected < m);
    }
}
