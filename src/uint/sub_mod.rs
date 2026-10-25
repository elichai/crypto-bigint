//! [`Uint`] modular subtraction operations.

use crate::{Limb, NonZero, SubMod, Uint};

impl<const LIMBS: usize> Uint<LIMBS> {
    /// Computes `self - rhs mod p`.
    ///
    /// Assumes `self - rhs` as unbounded signed integer is in `[-p, p)`.
    #[must_use]
    pub const fn sub_mod(&self, rhs: &Self, p: &NonZero<Self>) -> Self {
        let (out, mask) = self.borrowing_sub(rhs, Limb::ZERO);

        // If underflow occurred on the final limb, borrow = 0xfff...fff, otherwise
        // borrow = 0x000...000. Thus, we use it as a mask to conditionally add the modulus.
        out.wrapping_add(&p.as_ref().bitand_limb(mask))
    }

    /// Tries to subtract `rhs` from `(self..., carry)`, where `carry <= 1`, without wrapping.
    /// Returns the result of the subtraction and the new carry.
    #[inline(always)]
    pub(crate) const fn try_sub_with_carry(&self, mut carry: Limb, rhs: &Self) -> (Self, Limb) {
        let (out, borrow) = self.borrowing_sub(rhs, Limb::ZERO);
        let revert = borrow.lsb_to_choice().and(carry.is_zero());
        (carry, _) = carry.borrowing_sub(Limb::ZERO, Limb::select(borrow, Limb::ZERO, revert));
        (Uint::select(&out, self, revert), carry)
    }

    /// Computes `self - rhs mod p` for the special modulus
    /// `p = MAX+1-c` where `c` is small enough to fit in a single [`Limb`].
    ///
    /// Assumes `self - rhs` as unbounded signed integer is in `[-p, p)`.
    #[must_use]
    pub const fn sub_mod_special(&self, rhs: &Self, c: Limb) -> Self {
        let (out, borrow) = self.borrowing_sub(rhs, Limb::ZERO);

        // If underflow occurred, then we need to subtract `c` to account for
        // the underflow. This cannot underflow due to the assumption
        // `self - rhs >= -p`.
        let l = borrow.0 & c.0;
        out.wrapping_sub(&Self::from_word(l))
    }
}

impl<const LIMBS: usize> SubMod for Uint<LIMBS> {
    type Output = Self;

    fn sub_mod(&self, rhs: &Self, p: &NonZero<Self>) -> Self {
        debug_assert!(self < p.as_ref());
        debug_assert!(rhs < p.as_ref());
        self.sub_mod(rhs, p)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Limb, U128, U256};

    #[cfg(feature = "rand_core")]
    use crate::{NonZero, Random, RandomMod, Uint};
    #[cfg(feature = "rand_core")]
    use rand_core::SeedableRng;

    #[test]
    fn sub_mod_nist_p256() {
        let a =
            U256::from_be_hex("1a2472fde50286541d97ca6a3592dd75beb9c9646e40c511b82496cfc3926956");
        let b =
            U256::from_be_hex("d5777c45019673125ad240f83094d4252d829516fac8601ed01979ec1ec1a251");
        let n =
            U256::from_be_hex("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551")
                .to_nz()
                .unwrap();

        let actual = a.sub_mod(&b, &n);
        let expected =
            U256::from_be_hex("44acf6b7e36c1342c2c5897204fe09504e1e2efb1a900377dbc4e7a6a133ec56");

        assert_eq!(expected, actual);
    }

    #[test]
    fn try_sub_with_carry() {
        let three = U128::from_u8(3);
        let five = U128::from_u8(5);

        // carry = 0, self >= rhs: subtract.
        assert_eq!(
            five.try_sub_with_carry(Limb::ZERO, &three),
            (U128::from_u8(2), Limb::ZERO)
        );
        // carry = 0, self < rhs: unchanged.
        assert_eq!(
            three.try_sub_with_carry(Limb::ZERO, &five),
            (three, Limb::ZERO)
        );
        // carry = 1, self < rhs: 2^128 + 3 - 5 = 2^128 - 2.
        assert_eq!(
            three.try_sub_with_carry(Limb::ONE, &five),
            (U128::MAX.wrapping_sub(&U128::ONE), Limb::ZERO)
        );
        // carry = 1, self >= rhs: 2^129 - 1 - (2^127 - 1) = 2^128 + 2^127.
        assert_eq!(
            U128::MAX.try_sub_with_carry(Limb::ONE, &U128::MAX.shr1()),
            (U128::ONE.shl_vartime(127), Limb::ONE)
        );
    }

    #[test]
    #[cfg(feature = "rand_core")]
    fn sub_mod() {
        fn test_size<const LIMBS: usize>() {
            let mut rng = chacha20::ChaCha8Rng::seed_from_u64(1);
            let moduli = [
                NonZero::<Uint<LIMBS>>::random_from_rng(&mut rng),
                NonZero::<Uint<LIMBS>>::random_from_rng(&mut rng),
            ];

            for p in &moduli {
                let base_cases = [
                    (1u64, 0u64, 1u64.into()),
                    (0, 1, p.wrapping_sub(&1u64.into())),
                    (0, 0, 0u64.into()),
                ];
                for (a, b, c) in &base_cases {
                    let a: Uint<LIMBS> = (*a).into();
                    let b: Uint<LIMBS> = (*b).into();

                    let x = a.sub_mod(&b, p);
                    assert_eq!(*c, x, "{} - {} mod {} = {} != {}", a, b, p, x, c);
                }

                if LIMBS > 1 {
                    for _i in 0..100 {
                        let a: Uint<LIMBS> = Limb::random_from_rng(&mut rng).into();
                        let b: Uint<LIMBS> = Limb::random_from_rng(&mut rng).into();
                        let (a, b) = if a < b { (b, a) } else { (a, b) };

                        let c = a.sub_mod(&b, p);
                        assert!(c < **p, "not reduced");
                        assert_eq!(c, a.wrapping_sub(&b), "result incorrect");
                    }
                }

                for _i in 0..100 {
                    let a = Uint::<LIMBS>::random_mod_vartime(&mut rng, p);
                    let b = Uint::<LIMBS>::random_mod_vartime(&mut rng, p);

                    let c = a.sub_mod(&b, p);
                    assert!(c < **p, "not reduced: {} >= {} ", c, p);

                    let x = a.wrapping_sub(&b);
                    if a >= b && x < **p {
                        assert_eq!(c, x, "incorrect result");
                    }
                }
            }
        }

        // Test requires 1-limb is capable of representing a 64-bit integer
        cpubits::cpubits! {
            64 => { test_size::<1>(); }
        }

        test_size::<2>();
        test_size::<3>();
        if cfg!(not(miri)) {
            test_size::<4>();
            test_size::<8>();
            test_size::<16>();
        }
    }

    #[cfg(feature = "rand_core")]
    #[test]
    fn sub_mod_special() {
        fn test_size<const LIMBS: usize>() {
            let mut rng = chacha20::ChaCha8Rng::seed_from_u64(1);
            let moduli = [
                NonZero::<Limb>::random_from_rng(&mut rng),
                NonZero::<Limb>::random_from_rng(&mut rng),
            ];

            for special in &moduli {
                let p = &NonZero::new(Uint::ZERO.wrapping_sub(&Uint::from(special.get()))).unwrap();

                let minus_one = p.wrapping_sub(&Uint::ONE);

                let base_cases = [
                    (Uint::ZERO, Uint::ZERO, Uint::ZERO),
                    (Uint::ONE, Uint::ZERO, Uint::ONE),
                    (Uint::ZERO, Uint::ONE, minus_one),
                    (minus_one, minus_one, Uint::ZERO),
                    (Uint::ZERO, minus_one, Uint::ONE),
                ];
                for (a, b, c) in &base_cases {
                    let x = a.sub_mod_special(b, *special.as_ref());
                    assert_eq!(*c, x, "{} - {} mod {} = {} != {}", a, b, p, x, c);
                }

                for _i in 0..100 {
                    let a = Uint::<LIMBS>::random_mod_vartime(&mut rng, p);
                    let b = Uint::<LIMBS>::random_mod_vartime(&mut rng, p);

                    let c = a.sub_mod_special(&b, *special.as_ref());
                    assert!(c < **p, "not reduced: {} >= {} ", c, p);

                    let expected = a.sub_mod(&b, p);
                    assert_eq!(c, expected, "incorrect result");
                }
            }
        }

        test_size::<1>();
        test_size::<2>();
        test_size::<3>();
        if cfg!(not(miri)) {
            test_size::<4>();
            test_size::<8>();
            test_size::<16>();
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use crate::{Limb, NonZero, U128, Word};

    /// For `carry <= 1`, subtracts `rhs` from the 129-bit value `(self, carry)` if that does not
    /// wrap, and leaves it unchanged otherwise.
    #[kani::proof]
    fn uint_try_sub_with_carry_correct() {
        let (a, carry, rhs): (u128, bool, u128) = kani::any();
        let (res, carry_out) =
            U128::from_u128(a).try_sub_with_carry(Limb(Word::from(carry)), &U128::from_u128(rhs));

        let (diff, borrow) = a.overflowing_sub(rhs);
        let expected = if carry || !borrow { diff } else { a };
        assert_eq!(u128::from(res), expected);
        assert_eq!(carry_out.0, Word::from(carry && !borrow));
    }

    /// For `a - b` in `[-p, p)` (the documented precondition), `sub_mod` returns `(a - b) mod p`.
    #[kani::proof]
    fn uint_sub_mod_correct() {
        let (a, b, p): (u128, u128, u128) = kani::any();
        kani::assume(p != 0);
        let in_range = if a >= b { a - b < p } else { b - a <= p };
        kani::assume(in_range);

        let expected = if a >= b { a - b } else { p - (b - a) };
        let p_nz = NonZero::new(U128::from_u128(p)).unwrap();
        let res = U128::from_u128(a).sub_mod(&U128::from_u128(b), &p_nz);
        assert_eq!(u128::from(res), expected);
    }

    /// For `p = 2^128 - c` (any single-limb `c`, including 0) and `a - b` in `[-p, p)`,
    /// `sub_mod_special` returns `(a - b) mod p`, and `neg_mod_special` is the `a == 0` case.
    #[kani::proof]
    fn uint_sub_mod_special_correct() {
        let (a, b, c): (u128, u128, Word) = kani::any();
        // `p - 1` fits in a `u128` for every `c`, including `c == 0` (`p = 2^128`).
        let p_minus_1 = u128::from(c).wrapping_neg().wrapping_sub(1);
        let in_range = if a >= b {
            a - b <= p_minus_1
        } else {
            b - a - 1 <= p_minus_1
        };
        kani::assume(in_range);

        // For `a < b`, `a - b + p` reduced mod `2^128` is `a - b - c`.
        let expected = a
            .wrapping_sub(b)
            .wrapping_sub(if a >= b { 0 } else { u128::from(c) });
        let res = U128::from_u128(a).sub_mod_special(&U128::from_u128(b), Limb(c));
        assert_eq!(u128::from(res), expected);
        if a == 0 {
            assert_eq!(
                u128::from(U128::from_u128(b).neg_mod_special(Limb(c))),
                expected
            );
        }
    }
}
