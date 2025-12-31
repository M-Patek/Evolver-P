use num_bigint::BigInt;
use num_traits::{Signed, Zero, One};
use num_integer::Integer;
use serde::{Serialize, Deserialize};
use std::mem;

/// ClassGroupElement (类群元素)
/// Represents a binary quadratic form (a, b, c) corresponding to ax^2 + bxy + cy^2.
///
/// 它是虚二次域类群 Cl(Δ) 中的基本单元。
/// 在我们的架构中，它不仅仅是数学对象，更是 v-PuNNs 的“直觉状态”。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassGroupElement {
    pub a: BigInt,
    pub b: BigInt,
    pub c: BigInt,
}

// 基础的相等性比较
impl PartialEq for ClassGroupElement {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c
    }
}

impl Eq for ClassGroupElement {}

impl ClassGroupElement {
    /// 构造一个新的类群元素
    pub fn new(a: BigInt, b: BigInt, c: BigInt) -> Self {
        Self { a, b, c }
    }

    /// 获取判别式 Δ = b^2 - 4ac
    pub fn discriminant(&self) -> BigInt {
        (&self.b * &self.b) - (BigInt::from(4) * &self.a * &self.c)
    }

    /// 高斯合成算法 (Gaussian Composition) - 严格模式
    ///
    /// 实现了 Cohen 算法 5.4.7，并增加了数学不变量的运行时断言。
    /// 任何违反群公理的计算都会导致 Panic，防止错误的逻辑传播。
    pub fn compose(&self, other: &Self) -> Self {
        // [Safety Check 1] 群封闭性预检：判别式必须一致
        let delta = self.discriminant();
        if delta != other.discriminant() {
            panic!(
                "CRITICAL MATH VIOLATION: Group operation attempted on elements with different discriminants!\nSelf: {}\nOther: {}",
                delta, other.discriminant()
            );
        }

        let two = BigInt::from(2);

        // 1. Unification
        let s = (&self.b + &other.b) / &two;
        let n = (&self.b - &other.b) / &two;

        // 2. Extended GCD
        // d1 = gcd(a1, a2) = u*a1 + v*a2
        let egcd1 = self.a.extended_gcd(&other.a);
        let d1 = egcd1.gcd;
        let v = egcd1.y;

        // d = gcd(d1, s) = U*d1 + V*s
        let egcd2 = d1.extended_gcd(&s);
        let d = egcd2.gcd;
        let big_u = egcd2.x;
        let big_v = egcd2.y;

        // 3. 计算 a3 = (a1 * a2) / d^2
        let d_sq = &d * &d;
        let a1_a2 = &self.a * &other.a;
        
        // [Safety Check 2] 确保整除性 (Gauss Lemma)
        // 如果这里不能整除，说明底层的数论逻辑崩塌了。
        if !(&a1_a2 % &d_sq).is_zero() {
            panic!("MATH FAILURE: a1*a2 is not divisible by d^2. This implies the forms are not composable.");
        }
        let a3 = &a1_a2 / &d_sq;

        // 4. 计算 b3
        // Formula: b3 = b2 + 2 * (a2/d) * [V*(b1-b2)/2 - U*v*c2] mod 2a3
        let term1 = &big_v * &n;
        let term2 = &big_u * &v * &other.c;
        let big_k = term1 - term2;
        
        // 这里的 (a2/d) 也必须整除
        if !(&other.a % &d).is_zero() {
             panic!("MATH FAILURE: a2 not divisible by d.");
        }
        let factor = &two * &other.a / &d;
        
        let b3_raw = &other.b + &factor * &big_k;

        // 取模以保持数值大小可控
        let two_a3 = &two * &a3;
        let b3 = b3_raw.rem_euclid(&two_a3); 

        // 5. 计算 c3 = (b3^2 - Δ) / 4a3
        let b3_sq = &b3 * &b3;
        let num = &b3_sq - &delta;
        let four_a3 = &two * &two_a3;

        // [Safety Check 3] 二次型完整性检查
        // b^2 - 4ac 必须等于 Δ，这意味着 (b^2 - Δ) 必须能被 4a 整除。
        if !(&num % &four_a3).is_zero() {
            panic!("MATH FAILURE: Resulting form is not a valid quadratic form of discriminant Δ.");
        }
        let c3 = num / four_a3;

        // 6. 约简与返回
        let mut result = ClassGroupElement::new(a3, b3, c3);
        result.reduce(); 

        result
    }

    /// 计算逆元 (Inverse)
    /// (a, b, c)^-1 = (a, -b, c) ~ (a, -b+2ka, ...)
    pub fn inverse(&self) -> Self {
        let mut res = ClassGroupElement::new(self.a.clone(), -&self.b, self.c.clone());
        res.reduce();
        res
    }

    /// 获取单位元 (Identity)
    pub fn identity(discriminant: &BigInt) -> Self {
        let zero = BigInt::zero();
        let one = BigInt::one();
        let four = BigInt::from(4);

        let rem = discriminant.rem_euclid(&four);

        let (a, b, c) = if rem == zero {
            let c_val = -discriminant / &four;
            (one, zero, c_val)
        } else if rem == one {
            let c_val = (&one - discriminant) / &four;
            (one.clone(), one, c_val)
        } else {
            panic!("Invalid discriminant: must be 0 or 1 mod 4");
        };

        let mut res = ClassGroupElement::new(a, b, c);
        res.reduce();
        res
    }

    /// 演化 (Evolve) - 用于生成测试样本
    pub fn evolve(&self, input_seed: u64) -> Self {
        let delta = self.discriminant();
        let four = BigInt::from(4);
        
        let delta_mod_4 = delta.rem_euclid(&four);
        let target_b_parity = delta_mod_4 != BigInt::zero(); 

        let mut b_in = BigInt::from(input_seed);
        if b_in.is_odd() != target_b_parity {
            b_in += 1;
        }

        let b_sq = &b_in * &b_in;
        let num = b_sq - &delta;
        let a_in = num / &four;
        let c_in = BigInt::one();

        let mut g_in = ClassGroupElement::new(a_in, b_in, c_in);
        g_in.reduce();

        self.compose(&g_in)
    }

    /// 高斯约简算法 (Gaussian Reduction)
    /// 确保 |b| <= a <= c
    fn reduce(&mut self) {
        let zero = BigInt::zero();

        loop {
            // Step 1: Normalize b into (-a, a]
            let two_a = &self.a << 1; 
            if self.b.abs() > self.a {
                let mut r = &self.b % &two_a;
                if r > self.a { r -= &two_a; } 
                else if r <= -&self.a { r += &two_a; }
                
                let b_new = r;
                let k = (&b_new - &self.b) / &two_a;
                
                // c' = c + k*b + k^2*a
                let term = &self.b + (&self.a * &k);
                self.c = &self.c + &k * term;
                self.b = b_new;
            }

            // Step 2: Swap if a > c
            if self.a > self.c {
                mem::swap(&mut self.a, &mut self.c);
                self.b = -&self.b;
                continue;
            }

            // Step 3: Canonicalize boundary
            if self.a == self.c || self.a == self.b.abs() {
                if self.b < zero {
                    self.b = -&self.b;
                }
            }
            break;
        }
    }
}

// ==========================================
// 🛡️ 数学内核验证套件 (Verification Suite)
// ==========================================
#[cfg(test)]
mod verification_tests {
    use super::*;
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    // 辅助函数：生成一个随机的类群环境和其中的若干元素
    fn setup_random_group(seed: u64) -> (BigInt, Vec<ClassGroupElement>) {
        // 1. 生成判别式 (模拟 crypto_utils 的逻辑)
        let mut rng = StdRng::seed_from_u64(seed);
        
        // 简单找一个 -M (M=3 mod 4)
        // 为了测试速度，找小一点的判别式，比如 -23, -31 等
        // 这里硬编码几个经典的虚二次域用于回归测试
        let known_discriminants = vec![
            -23, -31, -47, -71, -10007 // 确保包含一些稍大的
        ];
        let d_val = known_discriminants[rng.gen_range(0..known_discriminants.len())];
        let delta = BigInt::from(d_val);

        // 2. 生成单位元
        let id = ClassGroupElement::identity(&delta);
        
        // 3. 生成若干随机元素
        let mut elements = vec![];
        let mut current = id.clone();
        for _ in 0..5 {
            // 随机演化几步
            let rand_step: u64 = rng.gen_range(100..10000);
            current = current.evolve(rand_step);
            elements.push(current.clone());
        }

        (delta, elements)
    }

    #[test]
    fn verify_axiom_closure_and_invariance() {
        // 验证：运算结果是否仍然是判别式为 Δ 的合法形式
        let (delta, elements) = setup_random_group(42);
        
        for a in &elements {
            for b in &elements {
                let c = a.compose(b);
                assert_eq!(c.discriminant(), delta, "Discriminant changed after composition!");
                
                // 验证 reduce 是否破坏了判别式
                let b2 = &c.b * &c.b;
                let 4ac = BigInt::from(4) * &c.a * &c.c;
                assert_eq!(b2 - 4ac, delta, "Reduction broke the quadratic form structure!");
            }
        }
    }

    #[test]
    fn verify_axiom_identity() {
        // 验证：A * E = A 且 E * A = A
        let (delta, elements) = setup_random_group(123);
        let id = ClassGroupElement::identity(&delta);

        for x in &elements {
            let left = id.compose(x);
            let right = x.compose(&id);

            assert_eq!(left, *x, "Identity element failed on left multiplication");
            assert_eq!(right, *x, "Identity element failed on right multiplication");
        }
    }

    #[test]
    fn verify_axiom_inverse() {
        // 验证：A * A^-1 = E
        let (delta, elements) = setup_random_group(777);
        let id = ClassGroupElement::identity(&delta);

        for x in &elements {
            let inv = x.inverse();
            let res = x.compose(&inv);
            
            assert_eq!(res, id, "Inverse composition did not yield Identity!");
            assert_eq!(inv.discriminant(), delta, "Inverse changed discriminant!");
        }
    }

    #[test]
    fn verify_axiom_commutativity() {
        // 验证：A * B = B * A (类群是阿贝尔群)
        let (_, elements) = setup_random_group(999);

        for i in 0..elements.len() {
            for j in i..elements.len() {
                let a = &elements[i];
                let b = &elements[j];
                
                let ab = a.compose(b);
                let ba = b.compose(a);
                
                assert_eq!(ab, ba, "Commutativity violated!");
            }
        }
    }

    #[test]
    fn verify_axiom_associativity() {
        // 验证：(A * B) * C = A * (B * C)
        // 这是最难满足的，也是检验算法正确性的试金石
        let (_, elements) = setup_random_group(2025);
        
        if elements.len() < 3 { return; }

        let a = &elements[0];
        let b = &elements[1];
        let c = &elements[2];

        let ab = a.compose(b);
        let ab_c = ab.compose(c);

        let bc = b.compose(c);
        let a_bc = a.compose(&bc);

        assert_eq!(ab_c, a_bc, "Associativity violated! The group structure is broken.");
    }

    #[test]
    fn stress_test_discriminant_preservation() {
        // 压力测试：连续运算 100 次，确保判别式不漂移
        let (delta, mut elements) = setup_random_group(101);
        let mut curr = elements[0].clone();
        
        for i in 1..100 {
            // 循环与列表中的元素复合
            let target = &elements[i % elements.len()];
            curr = curr.compose(target);
            assert_eq!(curr.discriminant(), delta, "Discriminant drift at iteration {}", i);
        }
    }
}
