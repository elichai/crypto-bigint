//! Modular reduction implementation.

use crate::{Limb, Odd, Uint};

/// Algorithm 14.32 in Handbook of Applied Cryptography <https://cacr.uwaterloo.ca/hac/about/chap14.pdf>
#[inline(always)]
const fn montgomery_reduction_inner(
    upper: &mut [Limb],
    lower: &mut [Limb],
    modulus: &[Limb],
    mod_neg_inv: Limb,
) -> Limb {
    let nlimbs = modulus.len();
    debug_assert!(nlimbs == upper.len());
    debug_assert!(nlimbs == lower.len());

    let mut meta_carry = Limb::ZERO;
    let mut new_sum;

    let mut i = 0;
    while i < nlimbs {
        let u = lower[i].wrapping_mul(mod_neg_inv);

        let (_, mut carry) = u.carrying_mul_add(modulus[0], lower[i], Limb::ZERO);
        let mut new_limb;

        let mut j = 1;
        while j < (nlimbs - i) {
            (new_limb, carry) = u.carrying_mul_add(modulus[j], lower[i + j], carry);
            lower[i + j] = new_limb;
            j += 1;
        }
        while j < nlimbs {
            (new_limb, carry) = u.carrying_mul_add(modulus[j], upper[i + j - nlimbs], carry);
            upper[i + j - nlimbs] = new_limb;
            j += 1;
        }

        (new_sum, meta_carry) = upper[i].carrying_add(carry, meta_carry);
        upper[i] = new_sum;

        i += 1;
    }

    meta_carry
}

/// Algorithm 14.32 in Handbook of Applied Cryptography <https://cacr.uwaterloo.ca/hac/about/chap14.pdf>
pub(crate) const fn montgomery_reduction<const LIMBS: usize>(
    lower_upper: &(Uint<LIMBS>, Uint<LIMBS>),
    modulus: &Odd<Uint<LIMBS>>,
    mod_neg_inv: Limb,
) -> Uint<LIMBS> {
    let (mut lower, mut upper) = *lower_upper;
    let meta_carry = montgomery_reduction_inner(
        &mut upper.limbs,
        &mut lower.limbs,
        &modulus.as_ref().limbs,
        mod_neg_inv,
    );

    // Division is simply taking the upper half of the limbs
    // Final reduction (at this point, the value is at most 2 * modulus,
    // so `meta_carry` is either 0 or 1)
    upper.try_sub_with_carry(meta_carry, modulus.as_ref()).0
}

/// This algorithm corresponds to a Montgomery reduction of the wide input `(x, 0)`,
/// Algorithm 14.32 in Handbook of Applied Cryptography <https://cacr.uwaterloo.ca/hac/about/chap14.pdf>
/// Or to a Montgomery multiplication of `x` by `1` (Algorithm 14.36).
/// This version does not produce a carry and does not need further correction by
/// subtracting the modulus as long as `x < modulus`. This is guaranteed because
/// `x < modulus => u < modulus => ((x + u•modulus) << N) < modulus`.
#[inline(always)]
pub const fn montgomery_retrieve_inner(
    x: &[Limb],
    out: &mut [Limb],
    modulus: &[Limb],
    mod_neg_inv: Limb,
) {
    let nlimbs = modulus.len();
    assert!(nlimbs == x.len() && nlimbs == out.len());

    let mut i = 0;
    while i < nlimbs {
        let xi = x[i];
        let u = out[0].wrapping_add(xi).wrapping_mul(mod_neg_inv);

        out[0] = u.carrying_mul_add(modulus[0], xi, out[0]).1;

        let mut j = 1;
        while j < nlimbs {
            (out[j - 1], out[j]) = u.carrying_mul_add(modulus[j], out[j], out[j - 1]);
            j += 1;
        }

        i += 1;
    }
}

/// For input `x < modulus` in Montgomery form, compute `x•R^-1 mod modulus`.
pub const fn montgomery_retrieve<const LIMBS: usize>(
    montgomery_form: &Uint<LIMBS>,
    modulus: &Odd<Uint<LIMBS>>,
    mod_neg_inv: Limb,
) -> Uint<LIMBS> {
    debug_assert!(Uint::lt(montgomery_form, modulus.as_ref()).to_bool());
    let mut res = Uint::ZERO;
    montgomery_retrieve_inner(
        montgomery_form.as_limbs(),
        res.as_mut_limbs(),
        modulus.as_ref().as_limbs(),
        mod_neg_inv,
    );
    res
}

#[cfg(kani)]
mod kani_proofs {
    use super::{
        montgomery_reduction, montgomery_reduction_inner, montgomery_retrieve,
        montgomery_retrieve_inner,
    };
    use crate::{Limb, Odd, Uint, Word};

    /// At m = R - 1, -m^-1 = 1 and REDC(lower + R*upper) = lower + upper
    /// before the final subtraction. Both halves are unrestricted here, including the carry.
    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(3)]
    fn montgomery_reduction_inner_max_modulus_wide() {
        let (lower, upper): (Word, Word) = kani::any();
        let mut lo = Uint::<1>::from_words([lower]);
        let mut hi = Uint::<1>::from_words([upper]);
        let carry = montgomery_reduction_inner(
            &mut hi.limbs,
            &mut lo.limbs,
            &Uint::<1>::MAX.limbs,
            Limb::ONE,
        );
        let actual = u128::from(hi.limbs[0].0) | (u128::from(carry.0) << Word::BITS);
        assert_eq!(actual, u128::from(lower) + u128::from(upper));
    }

    /// All wide inputs T < m*R at m = Word::MAX; R = 2^Word::BITS is 1 modulo m.
    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(3)]
    fn montgomery_reduction_max_modulus_normalized_wide() {
        let (lower, upper): (Word, Word) = kani::any();
        kani::assume(upper < Word::MAX);
        let sum = u128::from(lower) + u128::from(upper);
        let m = u128::from(Word::MAX);
        let expected = if sum >= m { sum - m } else { sum };
        let actual = montgomery_reduction(
            &(
                Uint::<1>::from_words([lower]),
                Uint::<1>::from_words([upper]),
            ),
            &Odd::<Uint<1>>::new(Uint::MAX).unwrap(),
            Limb::ONE,
        );
        assert_eq!(u128::from(actual.limbs[0].0), expected);
        assert!(actual.limbs[0].0 < Word::MAX);
    }

    /// Every reduced representation at m = R - 1 retrieves unchanged, since R^-1 = 1.
    #[kani::proof]
    #[kani::solver(z3)]
    #[kani::unwind(3)]
    fn montgomery_retrieve_max_modulus_reduced() {
        let x: Word = kani::any();
        kani::assume(x < Word::MAX);
        let input = Uint::<1>::from_words([x]);
        let mut out = Uint::<1>::ZERO;
        montgomery_retrieve_inner(
            &input.limbs,
            &mut out.limbs,
            &Uint::<1>::MAX.limbs,
            Limb::ONE,
        );
        assert_eq!(out.limbs[0].0, x);
        assert_eq!(
            montgomery_retrieve(&input, &Odd::<Uint<1>>::new(Uint::MAX).unwrap(), Limb::ONE).limbs
                [0]
            .0,
            x,
        );
    }
}
