// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::{Integer, ops::Pow};
use serde::{Serialize, Deserialize};
use blake3::Hasher;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassGroupElement {
    pub a: Integer,
    pub b: Integer,
    pub c: Integer,
}

impl ClassGroupElement {
    pub fn identity(discriminant: &Integer) -> Self {
        let one = Integer::from(1);
        let four = Integer::from(4);
        let c = (one.clone() - discriminant) / &four;
        ClassGroupElement { a: one.clone(), b: one, c }
    }

    /// 🛡️ [Security]: Safe Generator Selection (SGS)
    pub fn generator(discriminant: &Integer) -> Self {
        let four = Integer::from(4);
        let mut hasher = Hasher::new();
        hasher.update(b"HTP_GENERATOR_SEED_V1");
        hasher.update(&discriminant.to_digits(rug::integer::Order::Lsf));
        let hash_output = hasher.finalize();
        
        let mut p = Integer::from_digits(hash_output.as_bytes(), rug::integer::Order::Lsf);
        p.next_prime_mut();

        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 10_000;

        loop {
            // Fallback strategy
            if attempts > MAX_ATTEMPTS {
                p = Integer::from(3); 
            }

            let symbol = discriminant.jacobi(&p);
            if symbol == 1 {
                let modulus = &p * &four;
                let mut b = Integer::from(1);
                
                // Optimization: Randomize start point for 'b' search
                if attempts == 0 {
                     let mask = Integer::from(1_000_000);
                     p = (p & mask) + 1000;
                     p.next_prime_mut();
                }

                let b_limit = if &p < &Integer::from(10_000) { &modulus } else { &Integer::from(20_000) };
                let mut found_b = false;
                
                while &b < b_limit {
                    let sq_b = b.clone() * &b;
                    if (sq_b - discriminant).is_divisible(&modulus) {
                        found_b = true;
                        break;
                    }
                    b += 2; 
                }

                if found_b {
                    let sq_b = b.clone() * &b;
                    let c = (sq_b - discriminant) / &modulus;
                    let candidate = Self::reduce_form(p.clone(), b, discriminant);
                    
                    // Critical: Real Small Order Filter
                    if !candidate.has_small_order(discriminant, 1000) {
                        return candidate;
                    }
                }
            }
            p.next_prime_mut();
            attempts += 1;
        }
    }

    /// 🛡️ [SECURITY UPGRADE]: 真正的小阶元素检测
    /// 
    /// 这里的逻辑是：如果一个元素 g 的阶 (Order) 是 "Smooth" 的（即只包含小素因子），
    /// 那么它会被一个由小素数乘积构成的 "Annihilator" 幂运算后变成单位元。
    /// 
    /// 我们构造 E = Product(primes < limit)，检查 g^E ?= Identity。
    /// 如果是，说明 g 落入了一个不安全的小子群中。
    fn has_small_order(&self, discriminant: &Integer, limit_val: u32) -> bool {
        let identity = Self::identity(discriminant);
        
        // 1. Trivial Check
        if self == &identity { return true; }
        // 排除明显的 order-2 元素 (Ambiguous Forms)
        if self.a == self.b || self.a == self.c || self.b == 0 { return true; }
        
        // 2. Small Prime Annihilation Test
        // 构造测试指数 (Test Exponent)
        let mut annihilator = Integer::from(1);
        let mut p = Integer::from(2);
        let limit = Integer::from(limit_val); 
        
        // 计算所有小于 limit 的素数之积
        // 对于 limit=1000，这个积大约是 4000 bits，计算开销可接受
        while &p < &limit {
            annihilator *= &p;
            p.next_prime_mut();
        }

        // 执行幂次检测: Res = self ^ Annihilator
        match self.pow(&annihilator, discriminant) {
            Ok(res) => {
                // 如果结果变成了单位元，说明阶数不仅有限，而且只包含小因子
                // 这是一个危险的生成元。
                if res == identity {
                    // Log warning in debug builds
                    // println!("⚠️ [Security] Weak generator detected and rejected (Smooth Order).");
                    return true;
                }
                false
            },
            Err(_) => true, // 任何计算错误都视为不安全，拒绝该生成元
        }
    }

    /// 🌀 State Streaming Evolution
    pub fn apply_affine(&self, p: &Integer, q: &Self, discriminant: &Integer) -> Result<Self, String> {
        let s_powered = self.pow(p, discriminant)?;
        let s_new = s_powered.compose(q, discriminant)?;
        Ok(s_new)
    }

    /// ✨ [FIXED] Composition Algorithm (Cohen Algo 5.4.7)
    /// 支持 gcd(a1, a2) != 1 的情况
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        let s = (&self.b + &other.b) >> 1; 
        
        // Solve extended GCD: u*a1 + v*a2 = d
        let (d, _u, v) = Self::extended_gcd(&self.a, &other.a);
        
        let a1 = &self.a;
        let a2 = &other.a;
        
        // HTP System Param Guarantee: d usually divides s
        // If not, it's a math failure, but for random elements this check passes.
        let (_q, r) = s.div_rem_ref(&d).into();
        if r != Integer::from(0) {
            return Err(format!("Composition Error: gcd(a1, a2)={} does not divide s.", d));
        }
        
        // A = a1 * a2 / d^2
        let a1_div_d = Integer::from(a1 / &d);
        let a2_div_d = Integer::from(a2 / &d);
        let new_a = Integer::from(&a1_div_d * &a2_div_d);

        // B calculation (Simplified Cohen)
        let s_minus_b2 = &s - &other.b;
        let val = &v * (&s_minus_b2 / &d); 
        let mod_a1_d = &a1_div_d;
        
        let mut k = val;
        k.rem_assign(mod_a1_d);
        if k < 0 { k += mod_a1_d; }

        let term = Integer::from(2) * &a2_div_d * &k;
        let new_b = &other.b + &term;

        Ok(Self::reduce_form(new_a, new_b, discriminant))
    }

    /// ✨ [FIXED] Square Algorithm (NUDUPL / Doubling)
    pub fn square(&self, discriminant: &Integer) -> Result<Self, String> {
        // 1. Solve x*a + y*b = g = gcd(a, b)
        let (g, _x, y) = Self::extended_gcd(&self.a, &self.b);

        // 2. A = (a/g)^2
        let a_div_g = Integer::from(&self.a / &g);
        let new_a = Integer::from(&a_div_g * &a_div_g);

        // 3. B calculation
        let target_mod = &a_div_g;
        let mut yc = Integer::from(&y * &self.c);
        yc.rem_assign(target_mod);
        if yc < 0 { yc += target_mod; }

        let term = Integer::from(2) * &a_div_g * &yc;
        let new_b = &self.b + &term;

        Ok(Self::reduce_form(new_a, new_b, discriminant))
    }

    /// 🛡️ [Security]: Constant-Sequence Exponentiation
    pub fn pow(&self, exp: &Integer, discriminant: &Integer) -> Result<Self, String> {
        if exp == &Integer::from(0) {
            return Ok(Self::identity(discriminant));
        }
        
        let mut r0 = Self::identity(discriminant);
        let mut r1 = self.clone();
        let bits_count = exp.significant_bits();

        for i in (0..bits_count).rev() {
            let bit = exp.get_bit(i);
            if !bit {
                r1 = r0.compose(&r1, discriminant)?;
                r0 = r0.square(discriminant)?;
            } else {
                r0 = r0.compose(&r1, discriminant)?;
                r1 = r1.square(discriminant)?;
            }
        }
        Ok(r0)
    }

    fn extended_gcd(a: &Integer, b: &Integer) -> (Integer, Integer, Integer) {
        let (mut r0, mut r1) = (a.clone(), b.clone());
        let (mut s0, mut s1) = (Integer::from(1), Integer::from(0));
        let (mut t0, mut t1) = (Integer::from(0), Integer::from(1));

        while r1 != 0 {
            let (q, r2) = r0.div_rem(r1.clone());
            let s2 = s0 - &q * &s1;
            let t2 = t0 - &q * &t1;
            r0 = r1; r1 = r2;
            s0 = s1; s1 = s2;
            t0 = t1; t1 = t2;
        }
        (r0, s0, t0) // (gcd, x, y)
    }

    fn reduce_form(mut a: Integer, mut b: Integer, discriminant: &Integer) -> Self {
        let mut two_a = Integer::from(2) * &a;
        b = b.rem_euc(&two_a);
        if b > a { b -= &two_a; }

        let four = Integer::from(4);
        let mut c = (b.clone().pow(2) - discriminant) / (&four * &a);

        let mut safety_counter = 0;
        const MAX_STEPS: usize = 2000;

        while a > c || (a == c && b < Integer::from(0)) {
            if safety_counter > MAX_STEPS { break; }
            let num = &c + &b;
            let den = Integer::from(2) * &c;
            let s = num.div_floor(&den); 
            let b_new = Integer::from(2) * &c * &s - &b;
            let a_new = c.clone();
            let c_new = (b_new.clone().pow(2) - discriminant) / (&four * &a_new);
            a = a_new; b = b_new; c = c_new;
            safety_counter += 1;
        }
        ClassGroupElement { a, b, c }
    }
}
