//! [`Uint`] modular addition operations.

use crate::{AddMod, Limb, NonZero, Uint};

impl<const LIMBS: usize> Uint<LIMBS> {
    /// Computes `self + rhs mod p`.
    ///
    /// Assumes `self + rhs` as unbounded integer is `< 2p`.
    #[must_use]
    pub const fn add_mod(&self, rhs: &Self, p: &NonZero<Self>) -> Self {
        let (w, carry) = self.carrying_add(rhs, Limb::ZERO);
        w.try_sub_with_carry(carry, p.as_ref()).0
    }

    /// Computes `self + rhs mod p` for the special modulus
    /// `p = MAX+1-c` where `c` is small enough to fit in a single [`Limb`].
    ///
    /// Assumes `self + rhs` as unbounded integer is `< 2p`.
    #[must_use]
    pub const fn add_mod_special(&self, rhs: &Self, c: Limb) -> Self {
        // `Uint::carrying_add` also works with a carry greater than 1.
        let (out, carry) = self.carrying_add(rhs, c);

        // If overflow occurred, then above addition of `c` already accounts
        // for the overflow. Otherwise, we need to subtract `c` again, which
        // in that case cannot underflow.
        let l = carry.0.wrapping_sub(1) & c.0;
        out.wrapping_sub(&Self::from_word(l))
    }

    /// Computes `self + self mod p`.
    ///
    /// Assumes `self` as unbounded integer is `< p`.
    #[must_use]
    pub const fn double_mod(&self, p: &NonZero<Self>) -> Self {
        let (w, carry) = self.shl1_with_carry(Limb::ZERO);
        w.try_sub_with_carry(carry, p.as_ref()).0
    }
}

impl<const LIMBS: usize> AddMod for Uint<LIMBS> {
    type Output = Self;

    fn add_mod(&self, rhs: &Self, p: &NonZero<Self>) -> Self {
        debug_assert!(self < p.as_ref());
        debug_assert!(rhs < p.as_ref());
        self.add_mod(rhs, p)
    }
}

#[cfg(test)]
mod tests {
    use crate::U256;

    #[cfg(feature = "rand_core")]
    use crate::{Limb, NonZero, Random, RandomMod, Uint};
    #[cfg(feature = "rand_core")]
    use rand_core::SeedableRng;

    #[test]
    fn add_mod_nist_p256() {
        let a =
            U256::from_be_hex("44acf6b7e36c1342c2c5897204fe09504e1e2efb1a900377dbc4e7a6a133ec56");
        let b =
            U256::from_be_hex("d5777c45019673125ad240f83094d4252d829516fac8601ed01979ec1ec1a251");
        let n =
            U256::from_be_hex("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551")
                .to_nz()
                .unwrap();

        let actual = a.add_mod(&b, &n);
        let expected =
            U256::from_be_hex("1a2472fde50286541d97ca6a3592dd75beb9c9646e40c511b82496cfc3926956");

        assert_eq!(expected, actual);
    }

    #[test]
    fn double_mod_expected() {
        let a =
            U256::from_be_hex("44acf6b7e36c1342c2c5897204fe09504e1e2efb1a900377dbc4e7a6a133ec56");
        let n =
            U256::from_be_hex("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551")
                .to_nz()
                .unwrap();

        assert_eq!(a.add_mod(&a, &n), a.double_mod(&n));
    }

    #[cfg(feature = "rand_core")]
    #[test]
    fn add_mod_special() {
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
                    (Uint::ZERO, Uint::ONE, Uint::ONE),
                    (minus_one, Uint::ONE, Uint::ZERO),
                    (Uint::ONE, minus_one, Uint::ZERO),
                ];
                for (a, b, c) in &base_cases {
                    let x = a.add_mod_special(b, *special.as_ref());
                    assert_eq!(*c, x, "{} + {} mod {} = {} != {}", a, b, p, x, c);
                }

                for _i in 0..100 {
                    let a = Uint::<LIMBS>::random_mod_vartime(&mut rng, p);
                    let b = Uint::<LIMBS>::random_mod_vartime(&mut rng, p);

                    let c = a.add_mod_special(&b, *special.as_ref());
                    assert!(c < **p, "not reduced: {} >= {} ", c, p);

                    let expected = a.add_mod(&b, p);
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

    /// 129-bit values are `(high bit, low 128 bits)` pairs. The high bit is a `u8`, not a `bool`:
    /// Kani 0.68 mis-evaluates `<`/`<=` on symbolic `bool`s, and tuple ordering uses them.
    fn wide(lo: u128, hi: bool) -> (u8, u128) {
        (u8::from(hi), lo)
    }

    /// For `a + b < 2p` (the documented precondition), `add_mod` returns `(a + b) mod p`.
    #[kani::proof]
    fn uint_add_mod_correct() {
        let (a, b, p): (u128, u128, u128) = kani::any();
        kani::assume(p != 0);
        let (sum, carry) = a.overflowing_add(b);
        let (twice_p, p_carry) = p.overflowing_add(p);
        kani::assume(wide(sum, carry) < wide(twice_p, p_carry));
        kani::cover!(carry, "the sum overflows 128 bits");

        let expected = if carry || sum >= p {
            sum.wrapping_sub(p)
        } else {
            sum
        };
        let p_nz = NonZero::new(U128::from_u128(p)).unwrap();
        let res = U128::from_u128(a).add_mod(&U128::from_u128(b), &p_nz);
        assert_eq!(u128::from(res), expected);
        if a == b {
            assert_eq!(u128::from(U128::from_u128(a).double_mod(&p_nz)), expected);
        }
    }

    /// For `p = 2^128 - c` (any single-limb `c`, including 0) and `a + b < 2p`,
    /// `add_mod_special` returns `(a + b) mod p`.
    #[kani::proof]
    fn uint_add_mod_special_correct() {
        let (a, b, c): (u128, u128, Word) = kani::any();
        let p = if c == 0 {
            wide(0, true)
        } else {
            wide(u128::from(c).wrapping_neg(), false)
        };
        let (sum, carry) = a.overflowing_add(b);
        // For `c > 0`, `2p = 2^129 - 2c`; for `c == 0` every sum is below `2p = 2^129`.
        kani::assume(c == 0 || wide(sum, carry) < wide(p.1 << 1, true));
        kani::cover!(c != 0 && carry, "p < 2^128 and the sum overflows 128 bits");

        let expected = if wide(sum, carry) >= p {
            sum.wrapping_sub(p.1)
        } else {
            sum
        };
        let res = U128::from_u128(a).add_mod_special(&U128::from_u128(b), Limb(c));
        assert_eq!(u128::from(res), expected);
    }
}
