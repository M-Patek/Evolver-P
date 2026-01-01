use num_bigint::{BigInt, Sign, BigUint};
use num_traits::{Signed, Zero, One, Num, ToPrimitive};
use num_integer::Integer;
use serde::{Serialize, Deserialize};
use std::mem;
use sha2::{Sha256, Digest};

/// ClassGroupElement (类群元素)
/// Represents a binary quadratic form (a, b, c) corresponding to ax^2 + bxy + cy^2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassGroupElement {
    pub a: BigInt,
    pub b: BigInt,
    pub c: BigInt,
    // 为了方便后续计算，我们可以在元素内部携带其所属的宇宙标识（判别式）
    // 但为了节省序列化空间，标准形式通常只存 (a,b,c)。
    // 这里我们保持原样，但依赖外部确保 Delta 的一致性。
}

// 基础的相等性比较
impl PartialEq for ClassGroupElement {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c
    }
}

impl Eq for ClassGroupElement {}

/// 宇宙上下文：包含由 Context 决定的物理常数
pub struct Universe {
    pub discriminant: BigInt,
    pub context_hash: String,
}

impl ClassGroupElement {
    /// 构造一个新的类群元素
    pub fn new(a: BigInt, b: BigInt, c: BigInt) -> Self {
        Self { a, b, c }
    }

    /// 获取判别式 Δ = b^2 - 4ac
    pub fn discriminant(&self) -> BigInt {
        (&self.b * &self.b) - (BigInt::from(4) * &self.a * &self.c)
    }

    /// [理想模型核心实现]
    /// 真正的 "Contextual Universe Generation"
    /// 
    /// 1. 计算 Context 的哈希 H。
    /// 2. 从 H 开始搜索下一个满足 M ≡ 3 (mod 4) 的素数 M。
    /// 3. 设定宇宙判别式 Δ = -M。
    /// 4. 在该宇宙中生成初始种子。
    pub fn spawn_universe(context: &str) -> (Self, Universe) {
        // Step 1: 初始熵 (SHA-256)
        let mut hasher = Sha256::new();
        hasher.update(context.as_bytes());
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);
        
        let seed_bigint = BigInt::from_bytes_be(Sign::Plus, &hash_result);

        // Step 2: 寻找宇宙常数 M (Next Prime M ≡ 3 mod 4)
        // 这保证了虚二次域 Cl(-M) 的存在
        let m_prime = next_prime_3_mod_4(seed_bigint.clone());
        let delta = -m_prime; // Δ = -M

        // Step 3: 在确定的 Δ 宇宙中生成种子
        // 我们使用原始的 seed_bigint 作为 b 的来源，确保它与 Delta 也是绑定的
        let element = Self::generate_seed_in_delta(&delta, &seed_bigint);

        let universe = Universe {
            discriminant: delta,
            context_hash: hash_hex,
        };

        (element, universe)
    }

    /// 在给定的 Δ 中生成合法种子 (Internal Helper)
    fn generate_seed_in_delta(delta: &BigInt, initial_entropy: &BigInt) -> Self {
        let four = BigInt::from(4);
        
        // 扩展熵以匹配 Delta 的位宽 (简单的线性同余扩展)
        let bit_size = delta.bits(); 
        let mut b_expanded = initial_entropy.clone();
        let multiplier = BigInt::from_str_radix("5DEECE66D", 16).unwrap();
        let increment = BigInt::from(11u32);
        
        // 扩展直到位宽足够
        while b_expanded.bits() < bit_size {
            b_expanded = (&b_expanded * &multiplier) + &increment;
        }

        // 确保 b 的符号随机性 (利用熵的低位)
        if initial_entropy.is_odd() {
            b_expanded = -b_expanded;
        }

        // 调整奇偶性：b^2 ≡ Δ (mod 4)
        // 因为 Δ = -M，M ≡ 3 (mod 4) => Δ ≡ 1 (mod 4)
        // 所以 b^2 必须 ≡ 1 (mod 4)，即 b 必须是奇数
        if (&b_expanded % 2).is_zero() {
            b_expanded += BigInt::one();
        }

        // 计算 a = (b^2 - Δ) / 4
        let b_sq = &b_expanded * &b_expanded;
        let num = b_sq - delta;
        
        debug_assert!(&num % &four == BigInt::zero(), "Math logic error: numerator not divisible by 4");
        
        let a = num / &four;
        let c = BigInt::one();

        let mut element = Self::new(a, b_expanded, c);
        element.reduce();
        element
    }

    /// 高斯合成算法 (Gaussian Composition)
    pub fn compose(&self, other: &Self) -> Self {
        let delta = self.discriminant();
        // 在理想模型中，我们应当严格检查
        if delta != other.discriminant() {
            panic!("CRITICAL UNIVERSE COLLISION: Attempted to compose elements from different universes (Δ mismatch).");
        }

        let two = BigInt::from(2);

        // 1. Unification
        let s = (&self.b + &other.b) / &two;
        let n = (&self.b - &other.b) / &two;

        // 2. Extended GCD
        let egcd1 = self.a.extended_gcd(&other.a);
        let d1 = egcd1.gcd;
        let v = egcd1.y;

        let egcd2 = d1.extended_gcd(&s);
        let d = egcd2.gcd;
        let big_u = egcd2.x;
        let big_v = egcd2.y;

        // 3. Solve components
        let d_sq = &d * &d;
        let a1_a2 = &self.a * &other.a;
        let a3 = &a1_a2 / &d_sq;

        let term1 = &big_v * &n;
        let term2 = &big_u * &v * &other.c;
        let big_k = term1 - term2;
        let factor = &two * &other.a / &d;
        let b3_raw = &other.b + &factor * &big_k;

        let two_a3 = &two * &a3;
        let b3 = b3_raw.rem_euclid(&two_a3); 

        let b3_sq = &b3 * &b3;
        let num = &b3_sq - &delta;
        let four_a3 = &two * &two_a3;
        let c3 = num / four_a3;

        let mut result = ClassGroupElement::new(a3, b3, c3);
        result.reduce(); 
        result
    }

    pub fn inverse(&self) -> Self {
        let mut res = ClassGroupElement::new(self.a.clone(), -&self.b, self.c.clone());
        res.reduce();
        res
    }

    pub fn identity(discriminant: &BigInt) -> Self {
        let zero = BigInt::zero();
        let one = BigInt::one();
        let four = BigInt::from(4);

        let rem = discriminant.rem_euclid(&four);
        // Standard form identity depends on discriminant mod 4
        let (a, b, c) = if rem == zero {
            let c_val = -discriminant / &four;
            (one, zero, c_val)
        } else if rem == one {
            let c_val = (&one - discriminant) / &four;
            (one.clone(), one, c_val)
        } else {
            // This should not happen if M is generated correctly (3 mod 4 => Delta 1 mod 4)
             // But robust code handles 0 mod 4 too.
            panic!("Invalid discriminant structure");
        };

        let mut res = ClassGroupElement::new(a, b, c);
        res.reduce();
        res
    }

    fn reduce(&mut self) {
        let two_a = &self.a << 1; 
        loop {
            if self.b.abs() > self.a {
                let mut r = &self.b % &two_a; 
                if r > self.a { r -= &two_a; } 
                else if r <= -&self.a { r += &two_a; }
                
                let b_new = r;
                let k = (&b_new - &self.b) / &two_a;
                
                let term = &self.b + (&self.a * &k);
                self.c = &self.c + &k * term;
                self.b = b_new;
            }

            if self.a > self.c {
                mem::swap(&mut self.a, &mut self.c);
                self.b = -&self.b;
                continue;
            }

            if self.a == self.c || self.a == self.b.abs() {
                if self.b < BigInt::zero() {
                    self.b = -&self.b;
                }
            }
            break;
        }
    }
}

// --- Helper Functions for Primality Testing ---

/// 寻找下一个满足 p ≡ 3 (mod 4) 的素数
fn next_prime_3_mod_4(mut start: BigInt) -> BigInt {
    // 确保起始点是奇数
    if (&start % 2).is_zero() {
        start += 1;
    }
    
    // 确保起始点 ≡ 3 (mod 4)
    while (&start % 4) != BigInt::from(3) {
        start += 2;
    }

    // 暴力搜索（对于 256位 整数，素数密度足够高，通常很快能找到）
    loop {
        if is_probable_prime(&start, 10) {
            return start;
        }
        start += 4; // 步进 4，保持 ≡ 3 (mod 4) 性质
    }
}

/// Miller-Rabin 素性测试 (Deterministic behavior with fixed rounds or predictable randomness)
fn is_probable_prime(n: &BigInt, k: u32) -> bool {
    let one = BigInt::one();
    let two = BigInt::from(2);
    let zero = BigInt::zero();

    if *n <= one { return false; }
    if *n == two || *n == BigInt::from(3) { return true; }
    if (n % &two).is_zero() { return false; }

    // 写 n-1 = 2^s * d
    let mut d = n - &one;
    let mut s = 0;
    while (&d % &two).is_zero() {
        d /= &two;
        s += 1;
    }

    // 为了保持确定性 (Deterministic)，我们使用固定的前 k 个素数作为底数，
    // 或者基于 n 的哈希生成底数。
    // 这里为了简单且不需要引入 rand crate，我们使用线性同余生成伪随机底数。
    // 注意：这不是严谨的密码学证明，但对于 PoW 机制的 "Hardness" 足够了。
    
    let mut witness_gen = n.clone(); //以此为种子
    
    for _ in 0..k {
        // 生成 witness a in [2, n-2]
        // 简单的伪随机生成: a = (seed * 48271) % (n-3) + 2
        witness_gen = (&witness_gen * BigInt::from(48271u32)) % (n - &BigInt::from(3));
        let a = &witness_gen + &two;

        let mut x = mod_pow(&a, &d, n);
        
        if x == one || x == n - &one {
            continue;
        }

        let mut composite = true;
        for _ in 0..(s - 1) {
            x = mod_pow(&x, &two, n);
            if x == n - &one {
                composite = false;
                break;
            }
        }
        
        if composite {
            return false;
        }
    }

    true
}

/// 模幂运算 base^exp % modulus
fn mod_pow(base: &BigInt, exp: &BigInt, modulus: &BigInt) -> BigInt {
    base.modpow(exp, modulus)
}
