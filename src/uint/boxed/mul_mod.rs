//! [`BoxedUint`] modular multiplication operations.

use crate::{
    BoxedUint, ConcatenatingMul, ConcatenatingSquare, Limb, MulMod, NonZero, SquareMod, UintRef,
    div_limb::mul_rem,
};

impl BoxedUint {
    /// Computes `self * rhs mod p` for non-zero `p`.
    #[must_use]
    pub fn mul_mod(&self, rhs: &BoxedUint, p: &NonZero<BoxedUint>) -> BoxedUint {
        self.concatenating_mul(rhs).rem(p)
    }

    /// Computes `self * rhs mod p` for the special modulus
    /// `p = MAX+1-c` where `c` is small enough to fit in a single [`Limb`].
    ///
    /// When `c` is zero, `p = MAX+1` and the result is wrapping multiplication.
    ///
    /// For the modulus reduction, this function implements Algorithm 14.47 from
    /// the "Handbook of Applied Cryptography", by A. Menezes, P. van Oorschot,
    /// and S. Vanstone, CRC Press, 1996.
    #[must_use]
    pub fn mul_mod_special(&self, rhs: &Self, c: Limb) -> Self {
        debug_assert_eq!(self.bits_precision(), rhs.bits_precision());

        if self.nlimbs() == 1 {
            let a = self.limbs[0];
            let b = rhs.limbs[0];

            // For c = 0, use a nonzero placeholder divisor and select the
            // wrapping product in constant time.
            let (p, c_is_nonzero) = Limb::ZERO.wrapping_sub(c).to_nz_or_one();
            let reduced = mul_rem(a, b, p);
            let reduced = Limb::select(a.wrapping_mul(b), reduced, c_is_nonzero);
            return Self::from(reduced);
        }

        let product = self.concatenating_mul(rhs);
        let (lo, hi) = product.as_uint_ref().split_at(self.nlimbs());

        // Now use Algorithm 14.47 for the reduction
        let (mut lo, carry) = mac_by_limb(lo, hi, c, Limb::ZERO);

        let carry = {
            // Compute (carry + 1) * c without overflowing when carry is Limb::MAX.
            let rhs = carry.carrying_mul_add(c, c, Limb::ZERO);
            lo.carrying_add_assign(UintRef::new(&[rhs.0, rhs.1]), Limb::ZERO)
        };

        {
            let rhs = carry.wrapping_sub(Limb::ONE) & c;
            lo.borrowing_sub_assign(rhs, Limb::ZERO);
        }

        lo
    }

    /// Computes `self * self mod p`.
    #[must_use]
    pub fn square_mod(&self, p: &NonZero<BoxedUint>) -> Self {
        self.concatenating_square().rem(p)
    }

    /// Computes `self * self mod p` in variable time with respect to `p`.
    #[must_use]
    pub fn square_mod_vartime(&self, p: &NonZero<BoxedUint>) -> Self {
        self.concatenating_square().rem_vartime(p)
    }
}

impl MulMod for BoxedUint {
    type Output = Self;

    fn mul_mod(&self, rhs: &Self, p: &NonZero<Self>) -> Self {
        self.mul_mod(rhs, p)
    }
}

impl SquareMod for BoxedUint {
    type Output = Self;

    fn square_mod(&self, p: &NonZero<Self>) -> Self {
        self.square_mod(p)
    }
}

/// Computes `a + (b * c) + carry`, returning the result along with the new carry.
fn mac_by_limb(a: &UintRef, b: &UintRef, c: Limb, carry: Limb) -> (BoxedUint, Limb) {
    let mut res = BoxedUint::zero_with_precision(a.bits_precision());
    let mut carry = carry;

    for i in 0..a.nlimbs() {
        (res.limbs[i], carry) = b.limbs[i].carrying_mul_add(c, a.limbs[i], carry);
    }

    (res, carry)
}

#[cfg(kani)]
mod kani_proofs {
    use super::mac_by_limb;
    use crate::uint::boxed::kani_util::{L64, any_boxed, value};
    use crate::{BoxedUint, Limb, NonZero, Word};

    fn native_product(a: u64, b: u64) -> u128 {
        if Word::BITS == 32 {
            let (a_lo, a_hi) = (u64::from(a as u32), a >> 32);
            let (b_lo, b_hi) = (u64::from(b as u32), b >> 32);
            // Exact a*b; native radix-2^32 carries share multiplier circuits with the implementation.
            let p00 = a_lo * b_lo;
            let p01 = a_lo * b_hi + (p00 >> 32);
            let p10 = a_hi * b_lo + u64::from(p01 as u32);
            let p11 = a_hi * b_hi + (p01 >> 32) + (p10 >> 32);
            u128::from(p00 as u32) | (u128::from(p10 as u32) << 32) | (u128::from(p11) << 64)
        } else {
            u128::from(a) * u128::from(b)
        }
    }

    /// All 64-bit operands and full-word multiplier/carry; the total fits u128.
    #[kani::proof]
    #[cfg_attr(cpubits = "32", kani::solver(cadical))]
    #[cfg_attr(not(cpubits = "32"), kani::solver(z3))]
    #[kani::unwind(6)]
    fn boxed_mac_by_limb_64() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<L64>());
        let (c, carry): (Word, Word) = kani::any();
        let expected = value(&a) + native_product(value(&b) as u64, c as u64) + u128::from(carry);
        let (result, high) = mac_by_limb(a.as_uint_ref(), b.as_uint_ref(), Limb(c), Limb(carry));
        assert_eq!(result.bits_precision(), 64);
        assert_eq!(value(&result), u128::from(expected as u64));
        assert_eq!(u128::from(high.0), expected >> 64);
    }

    /// The former one-limb c=0 regression, with its exact input and result.
    #[kani::proof]
    #[kani::unwind(140)]
    fn boxed_mul_mod_special_zero_c_regression() {
        let result =
            BoxedUint::from(Limb(3)).mul_mod_special(&BoxedUint::from(Limb(7)), Limb::ZERO);
        assert_eq!(result, BoxedUint::from(Limb(21)));
    }

    /// Every word-sized pair, including values whose product overflows a word.
    #[kani::proof]
    #[kani::unwind(140)]
    fn boxed_mul_mod_special_zero_one_word() {
        let (a, b): (Word, Word) = kani::any();
        let result =
            BoxedUint::from(Limb(a)).mul_mod_special(&BoxedUint::from(Limb(b)), Limb::ZERO);
        assert_eq!(result.bits_precision(), Word::BITS);
        assert_eq!(result, BoxedUint::from(Limb(a.wrapping_mul(b))));
    }

    /// Word-sized operands in two-word storage, with an exact native product oracle.
    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(6)]
    fn boxed_mul_mod_special_zero_two_words() {
        let (a, b): (Word, Word) = kani::any();
        let lhs = BoxedUint::from_words([a, 0]);
        let rhs = BoxedUint::from_words([b, 0]);
        let result = lhs.mul_mod_special(&rhs, Limb::ZERO);
        assert_eq!(result.bits_precision(), 2 * Word::BITS);
        assert_eq!(value(&result), u128::from(a) * u128::from(b));
    }

    /// The same carry boundary as the fixed-width regression, through boxed reduction.
    #[kani::proof]
    #[kani::unwind(10)]
    fn boxed_mul_mod_special_max_carry() {
        let a = BoxedUint::from_words([0, Word::MAX - 1, Word::MAX]);
        let result = a.mul_mod_special(&a, Limb::MAX);
        assert_eq!(result.bits_precision(), 3 * Word::BITS);
        assert_eq!(result, BoxedUint::from_words([1, 2, 1]));
    }

    /// For p=B^3-B+1 and d=B+k, check k=0, 1, MAX with independent result words.
    #[kani::proof]
    #[kani::unwind(10)]
    fn boxed_mul_mod_special_max_c_three_word_boundaries() {
        let (a, expected) = match kani::any::<u8>() {
            0 => ([1, Word::MAX - 1, Word::MAX], [0, 0, 1]),
            1 => ([0, Word::MAX - 1, Word::MAX], [1, 2, 1]),
            _ => ([2, Word::MAX - 2, Word::MAX], [1, Word::MAX - 3, 3]),
        };
        let a = BoxedUint::from_words(a);
        let result = a.mul_mod_special(&a, Limb::MAX);
        assert_eq!(result.bits_precision(), 3 * Word::BITS);
        assert_eq!(result, BoxedUint::from_words(expected));
    }

    /// Every reduced u64 pair modulo 2^64-1, including the multi-limb reduction on 32-bit words.
    #[kani::proof]
    #[cfg_attr(cpubits = "32", kani::solver(cadical))]
    #[cfg_attr(not(cpubits = "32"), kani::solver(z3))]
    #[kani::unwind(140)]
    fn boxed_mul_mod_special_64_mersenne() {
        let (a, b) = (any_boxed::<L64>(), any_boxed::<L64>());
        let modulus = u128::from(u64::MAX);
        kani::assume(value(&a) < modulus && value(&b) < modulus);
        // Both are exact a*b; retain the measured native expression for each word width.
        let product = if Word::BITS == 32 {
            native_product(value(&a) as u64, value(&b) as u64)
        } else {
            value(&a) * value(&b)
        };
        // Since 2^64 ≡ 1 (mod m), folding then subtracting computes product % m.
        // Reduced inputs give folded < 2m, so one subtraction replaces a costly division oracle.
        let folded = (product & modulus) + (product >> 64);
        let expected = if folded >= modulus {
            folded - modulus
        } else {
            folded
        };
        let result = a.mul_mod_special(&b, Limb::ONE);
        assert_eq!(result.bits_precision(), 64);
        assert_eq!(value(&result), expected);
        assert!(value(&result) < modulus);
    }

    /// Full u8 operands in 64-bit storage; u16 operands modulo 65521 timed out with both solvers.
    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(140)]
    fn boxed_mul_mod_u8_fixed_prime() {
        let (a, b): (u8, u8) = kani::any();
        let modulus = NonZero::new(BoxedUint::from(251u64)).unwrap();
        let result =
            BoxedUint::from(u64::from(a)).mul_mod(&BoxedUint::from(u64::from(b)), &modulus);
        assert_eq!(result.bits_precision(), 64);
        assert_eq!(
            value(&result),
            u128::from(u64::from(a) * u64::from(b) % 251)
        );
    }

    /// Both square APIs, for every u16 in 64-bit storage and modulus 65521.
    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(140)]
    fn boxed_square_mod_u16_fixed_prime() {
        let n: u16 = kani::any();
        let a = BoxedUint::from(u64::from(n));
        let modulus = NonZero::new(BoxedUint::from(65521u64)).unwrap();
        let expected = u128::from(u64::from(n) * u64::from(n) % 65521);
        let result = a.square_mod(&modulus);
        assert_eq!(result.bits_precision(), 64);
        assert_eq!(value(&result), expected);
        let result = a.square_mod_vartime(&modulus);
        assert_eq!(result.bits_precision(), 64);
        assert_eq!(value(&result), expected);
    }
}

#[cfg(all(test, feature = "rand_core"))]
mod tests {
    use crate::{BoxedUint, ConcatenatingMul, Limb, NonZero, Random, RandomMod, Resize};
    use rand_core::SeedableRng;

    #[test]
    fn mul_mod_special_zero_c() {
        for size in [1, 2, 3, 4] {
            let bits = size * Limb::BITS;
            let zero = BoxedUint::zero_with_precision(bits);
            let one = BoxedUint::one_with_precision(bits);
            let max = zero.wrapping_sub(&one);

            for a in [&zero, &one, &max] {
                for b in [&zero, &one, &max] {
                    assert_eq!(a.mul_mod_special(b, Limb::ZERO), a.wrapping_mul(b));
                }
            }
        }
    }

    #[test]
    fn mul_mod_special_max_c() {
        for size in [3, 4] {
            let bits = size * Limb::BITS;
            let c = BoxedUint::from(Limb::MAX).resize(bits);
            let p = NonZero::new(BoxedUint::zero_with_precision(bits).wrapping_sub(&c)).unwrap();
            let a = p.wrapping_sub(&c);

            // (p - c)^2 mod p = c^2. With at least three limbs, the first
            // reduction produces a carry of Limb::MAX, so carry + 1 overflows.
            let expected = c.wrapping_mul(&c);
            assert_eq!(a.mul_mod_special(&a, Limb::MAX), expected);
            assert_eq!(a.mul_mod(&a, &p), expected);
        }
    }

    #[test]
    fn mul_mod_special() {
        let mut rng = chacha20::ChaCha8Rng::seed_from_u64(1);
        let moduli = [
            NonZero::<Limb>::random_from_rng(&mut rng),
            NonZero::<Limb>::random_from_rng(&mut rng),
        ];
        let sizes = if cfg!(miri) {
            &[1, 2, 3][..]
        } else {
            &[1, 2, 3, 4, 8, 16][..]
        };

        for size in sizes.iter().copied() {
            let bits = size * Limb::BITS;

            for special in &moduli {
                let zero = BoxedUint::zero_with_precision(bits);
                let one = BoxedUint::one_with_precision(bits);
                let p = NonZero::new(zero.wrapping_sub(special.get())).unwrap();
                let minus_one = p.wrapping_sub(Limb::ONE);

                let base_cases = [
                    (&zero, &zero, &zero),
                    (&one, &zero, &zero),
                    (&zero, &one, &zero),
                    (&one, &one, &one),
                    (&minus_one, &minus_one, &one),
                    (&minus_one, &one, &minus_one),
                    (&one, &minus_one, &minus_one),
                ];
                for (a, b, c) in base_cases {
                    let x = a.mul_mod_special(b, *special.as_ref());
                    assert_eq!(&x, c, "{} * {} mod {} = {} != {}", a, b, p, x, c);
                }

                for _i in 0..100 {
                    let a = BoxedUint::random_mod_vartime(&mut rng, &p);
                    let b = BoxedUint::random_mod_vartime(&mut rng, &p);

                    let c = a.mul_mod_special(&b, *special.as_ref());
                    assert!(c < p.as_ref(), "not reduced: {} >= {} ", c, p);

                    let expected = {
                        let prod = a.concatenating_mul(&b);
                        prod.rem_vartime(&p)
                    };
                    assert_eq!(c, expected, "incorrect result");
                }
            }
        }
    }
}
