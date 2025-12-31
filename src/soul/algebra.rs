use num_bigint::BigInt;
use num_traits::{Signed, Zero, One}; // [Added] One 用于单位元
use num_integer::Integer; // [Added] 引入 Integer trait 以使用 extended_gcd
use serde::{Serialize, Deserialize};
use std::mem;

/// ClassGroupElement (类群元素)
/// Represents a binary quadratic form (a, b, c) corresponding to ax^2 + bxy + cy^2.
///
/// 它是虚二次域类群 Cl(Δ) 中的基本单元。
/// 在我们的架构中，它不仅仅是数学对象，更是 v-PuNNs 的“直觉状态”。
/// 随着它在群轨道上的演化，v-PuNNs 的权重会发生确定性的“混沌跳变”。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassGroupElement {
    pub a: BigInt,
    pub b: BigInt,
    pub c: BigInt,
}

// 基础的相等性比较
// 只要 (a, b, c) 三个系数完全一致，我们就认为这两个形态是相等的。
impl PartialEq for ClassGroupElement {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c
    }
}

// 标记为 Eq，因为 PartialEq 是自反、对称且传递的
impl Eq for ClassGroupElement {}

impl ClassGroupElement {
    /// 构造一个新的类群元素
    ///
    /// # 参数
    /// * `a`, `b`, `c`: 二次型 ax^2 + bxy + cy^2 的系数
    pub fn new(a: BigInt, b: BigInt, c: BigInt) -> Self {
        // 在构造时我们暂时不自动 reduce，允许中间状态的存在。
        // 完整的群运算逻辑会在 composition 中调用 reduce。
        Self { a, b, c }
    }

    /// 获取判别式 Δ = b^2 - 4ac
    /// 所有的群运算必须在相同的判别式 Δ 下进行
    pub fn discriminant(&self) -> BigInt {
        (&self.b * &self.b) - (BigInt::from(4) * &self.a * &self.c)
    }

    /// 高斯合成算法 (Gaussian Composition / Dirichlet Composition)
    ///
    /// 将两个二次型 f1 和 f2 融合成一个新的二次型 f3 = f1 * f2。
    /// 这里的“乘法”是类群中的群运算。
    ///
    /// # 算法流程
    /// 基于 Arndt (2010) 或 Cohen (Alg 5.4.7) 的标准 Dirichlet 合成公式：
    /// 设 f1 = (a1, b1, c1), f2 = (a2, b2, c2)
    /// 1. 计算 s = (b1 + b2) / 2
    /// 2. 使用扩展欧几里得算法计算 gcd 及其系数：
    ///    - gcd(a1, a2) -> d1, u, v 使得 u*a1 + v*a2 = d1
    ///    - gcd(d1, s)  -> d, U, V  使得 U*d1 + V*s = d
    ///      此时 d = gcd(a1, a2, s)
    /// 3. 计算新系数：
    ///    - a3 = (a1 * a2) / d^2
    ///    - b3 = b2 + 2 * (a2 / d) * (V * (b1 - b2)/2 - U * v * c2) mod 2*a3
    ///    - c3 = (b3^2 - Δ) / (4 * a3)
    /// 4. 调用 reduce() 归一化结果
    pub fn compose(&self, other: &Self) -> Self {
        // 安全检查：判别式必须一致，否则属于不同的群
        let delta = self.discriminant();
        assert_eq!(delta, other.discriminant(), "Discriminant mismatch in composition!");

        let two = BigInt::from(2);

        // 1. 准备中间变量
        // s = (b1 + b2) / 2
        let s = (&self.b + &other.b) / &two;
        // n = (b1 - b2) / 2 (用于后续 b3 计算)
        let n = (&self.b - &other.b) / &two;

        // 2. 第一次 Extended GCD: u*a1 + v*a2 = d1
        // extended_gcd 返回结构体 ExtendedGcd { gcd, x, y }
        let egcd1 = self.a.extended_gcd(&other.a);
        let d1 = egcd1.gcd;
        let _u = egcd1.x; // 我们不需要 u，只需要 v
        let v = egcd1.y;

        // 3. 第二次 Extended GCD: U*d1 + V*s = d
        // d = gcd(a1, a2, s)
        let egcd2 = d1.extended_gcd(&s);
        let d = egcd2.gcd;
        let big_u = egcd2.x; // U
        let big_v = egcd2.y; // V

        // 4. 计算 a3 = a1 * a2 / d^2
        let d_sq = &d * &d;
        let a1_a2 = &self.a * &other.a;
        let a3 = &a1_a2 / &d_sq;

        // 5. 计算 b3
        // 核心公式: K = V * ((b1 - b2) / 2) - U * v * c2
        // b3 = b2 + 2 * (a2 / d) * K
        
        let term1 = &big_v * &n;
        let term2 = &big_u * &v * &other.c;
        let big_k = term1 - term2;
        
        let factor = &two * &other.a / &d;
        let b3_raw = &other.b + &factor * &big_k;

        // 取模以保持数值大小可控: b3 = b3_raw mod 2*a3
        let two_a3 = &two * &a3;
        let b3 = b3_raw.rem_euclid(&two_a3); // 使用 rem_euclid 保证正余数处理更自然

        // 6. 计算 c3
        // c3 = (b3^2 - Δ) / 4a3
        // 注意：这里必须整除，如果不整除说明前面的计算有误
        let b3_sq = &b3 * &b3;
        let num = &b3_sq - &delta;
        let four_a3 = &two * &two_a3;
        let c3 = num / four_a3;

        // 7. 构造并约简
        let mut result = ClassGroupElement::new(a3, b3, c3);
        result.reduce(); // 坍缩到唯一态！

        result
    }

    /// 计算逆元 (Inverse)
    ///
    /// 类群中的逆元由 (a, b, c)^-1 = (a, -b, c) 给出。
    /// 这一步操作对应于在群轨道上的“回溯”。
    pub fn inverse(&self) -> Self {
        // (a, b, c)^-1 = (a, -b, c)
        // 构造后立即 reduce 以确保满足规范形式（特别是边界条件）
        let mut res = ClassGroupElement::new(self.a.clone(), -&self.b, self.c.clone());
        res.reduce();
        res
    }

    /// 获取单位元 (Identity / Principal Class)
    ///
    /// 对于给定的判别式 Δ，单位元是主形式 (Principal Form)。
    /// 它是群运算的起点，代表“无直觉”或“初始状态”。
    ///
    /// - 若 Δ ≡ 0 (mod 4): (1, 0, -Δ/4)
    /// - 若 Δ ≡ 1 (mod 4): (1, 1, (1-Δ)/4)
    pub fn identity(discriminant: &BigInt) -> Self {
        let zero = BigInt::zero();
        let one = BigInt::one();
        let four = BigInt::from(4);

        // 使用 rem_euclid 确保余数为正
        let rem = discriminant.rem_euclid(&four);

        let (a, b, c) = if rem == zero {
            // Δ ≡ 0 (mod 4) -> (1, 0, -Δ/4)
            let c_val = -discriminant / &four;
            (one, zero, c_val)
        } else if rem == one {
            // Δ ≡ 1 (mod 4) -> (1, 1, (1-Δ)/4)
            let c_val = (&one - discriminant) / &four;
            (one.clone(), one, c_val)
        } else {
            // 数学上，基本判别式必须模 4 余 0 或 1
            panic!("Invalid discriminant for binary quadratic form: must be 0 or 1 mod 4");
        };

        // 构造并约简（虽然主形式通常已是 reduced，但为了统一性还是调用）
        let mut res = ClassGroupElement::new(a, b, c);
        res.reduce();
        res
    }

    /// 演化 (Evolve)
    ///
    /// 基于输入种子生成一个确定性的随机群元素 g_in，
    /// 并将其作用于当前状态：self <- self * g_in
    ///
    /// 这里的生成逻辑采用简化的映射方案：
    /// 构造一个二次型 (a', b', c') 使得其判别式为 Δ，且 b' 由种子派生。
    /// 令 c' = 1, b' = seed (调整奇偶性), 则 a' = (b'^2 - Δ) / 4。
    /// 这种构造保证了 (a', b', c') 是一个合法的（虽然未经约简的）形式。
    /// 随后调用 reduce() 将其坍缩为规范形式，再与当前状态复合。
    pub fn evolve(&self, input_seed: u64) -> Self {
        let delta = self.discriminant();
        let four = BigInt::from(4);
        
        // 1. 确定 b 的奇偶性要求
        // Δ ≡ 0 (mod 4) => b 必须偶
        // Δ ≡ 1 (mod 4) => b 必须奇
        // (注：虚二次域判别式只能是 0 或 1 mod 4)
        let delta_mod_4 = delta.rem_euclid(&four);
        let target_b_parity = delta_mod_4 != BigInt::zero(); // true if odd, false if even

        // 2. 构造 b_in
        let mut b_in = BigInt::from(input_seed);
        if b_in.is_odd() != target_b_parity {
            b_in += 1;
        }

        // 3. 构造 a_in
        // 我们设定 c_in = 1
        // 由 Δ = b^2 - 4ac => 4a = b^2 - Δ => a = (b^2 - Δ) / 4
        // 因为调整了 b 的奇偶性，b^2 ≡ Δ (mod 4)，所以能整除
        let b_sq = &b_in * &b_in;
        let num = b_sq - &delta;
        let a_in = num / &four;
        let c_in = BigInt::one();

        // 4. 构造 g_in 并约简
        // 注意：这里构造出的 a_in 可能非常大，但这是合法的。
        // reduce() 会将其转化为规范形式（即 a 变小，接近 sqrt(|Δ|)）。
        let mut g_in = ClassGroupElement::new(a_in, b_in, c_in);
        g_in.reduce();

        // 5. 复合
        self.compose(&g_in)
    }

    /// 高斯约简算法 (Gaussian Reduction)
    ///
    /// 将二次型化简为唯一的最简形式 (Reduced Form)。
    /// 这是一个迭代过程，确保最终状态满足：
    /// 1. |b| <= a <= c (正规化)
    /// 2. 若 |b| == a 或 a == c，则 b >= 0 (边界唯一性)
    ///
    /// 这个过程保证了同一理想类的所有元素最终都会坍缩到同一个 (a,b,c) 状态。
    fn reduce(&mut self) {
        let zero = BigInt::zero();

        loop {
            // --- 步骤 1: 正规化 b (Normalize b) ---
            // 目标: 使得 -a < b <= a
            // 我们通过变换 b -> b + 2ka 来实现平移
            
            let two_a = &self.a << 1; // 2 * a
            
            // 检查 b 是否在 (-a, a] 范围内
            if self.b.abs() > self.a {
                // 计算 r = b % 2a (带符号)
                let mut r = &self.b % &two_a;
                
                // 调整 r 到 (-a, a] 区间
                if r > self.a {
                    r -= &two_a;
                } else if r <= -&self.a {
                    r += &two_a;
                }
                
                let b_new = r;
                
                // 反推 k 值，用于更新 c
                // b_new = b + 2ak => k = (b_new - b) / 2a
                let k = (&b_new - &self.b) / &two_a;
                
                // 更新 c: c' = c + k*b + k^2*a = c + k(b + ak)
                // 这里的公式利用了变换 (x, y) -> (x+ky, y) 保持判别式不变的性质
                let term = &self.b + (&self.a * &k);
                self.c = &self.c + &k * term;
                
                self.b = b_new;
            }

            // --- 步骤 2: 确保 a <= c ---
            // 如果 a > c，说明当前的椭圆还是太“扁”了，我们需要旋转坐标系。
            // 对应矩阵变换 S: (x, y) -> (-y, x)，即 (a, b, c) -> (c, -b, a)
            if self.a > self.c {
                mem::swap(&mut self.a, &mut self.c);
                self.b = -&self.b;
                
                // 交换后，a 变了（变成了原来的 c），可能破坏了 |b| <= a 的条件
                // 所以必须重新进入循环进行平移检查
                continue;
            }

            // --- 步骤 3: 边界条件处理 (Boundary Conditions) ---
            // 此时已满足 |b| <= a <= c
            // 但对于边界情况 a == c 或 a == |b|，会有两个等价形式 (a, -b, c) 和 (a, b, c)。
            // 为了唯一性，我们强制规定 b >= 0。
            
            if self.a == self.c || self.a == self.b.abs() {
                if self.b < zero {
                    self.b = -&self.b;
                    // b 变号不会改变 a 或 c 的大小关系，也不会破坏 |b| <= a
                    // 所以此时已经是最终形态，可以直接退出
                }
            }

            // 如果能走到这里，说明已经完全满足约简条件
            break;
        }
    }
}
