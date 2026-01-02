use crate::soul::algebra::IdealClass;
use std::f64::consts::PI;
use num_traits::ToPrimitive;
use num_bigint::BigInt;
use sha2::{Sha256, Digest}; // [Fix] 使用 SHA-256 替代 DefaultHasher

/// 导航特征 (Navigation Features)
/// 用于启发式搜索的平滑几何特征 (Will Layer)。
/// 这是 VAPO 的“眼睛”，它需要是连续的(Lipschitz)，这样微小的代数变化
/// 才能体现为特征空间中的平滑移动。
#[derive(Debug, Clone, PartialEq)]
pub struct NavigationFeatures {
    pub cos_x: f64,
    pub sin_x: f64,
    pub log_y: f64,
}

impl NavigationFeatures {
    pub fn to_vector(&self) -> Vec<f64> {
        vec![self.cos_x, self.sin_x, self.log_y]
    }

    pub fn extract(state: &IdealClass) -> Self {
        // 1. 计算 x = -b / 2a
        // x 在 [-0.5, 0.5] 之间，直接转 f64 安全
        let b_f64 = bigint_to_scaled_f64(&state.b, &state.a);
        let x = -b_f64 / 2.0;

        // 2. 计算 log(y)
        // log(y) = 0.5 * log(|D|) - log(2a)
        let log_delta = state.discriminant().abs().bits() as f64 * 2.0_f64.ln();
        let log_a = bigint_log_e(&state.a);
        
        let log_y = 0.5 * log_delta - (2.0_f64.ln() + log_a);

        NavigationFeatures {
            cos_x: (2.0 * PI * x).cos(),
            sin_x: (2.0 * PI * x).sin(),
            log_y,
        }
    }
}

// 辅助函数：大数对数计算
fn bigint_log_e(n: &BigInt) -> f64 {
    if n.sign() == num_bigint::Sign::NoSign {
        return f64::NEG_INFINITY;
    }
    let bits = n.bits();
    (bits as f64) * 2.0_f64.ln()
}

// 辅助函数：大数除法转浮点
fn bigint_to_scaled_f64(numerator: &BigInt, denominator: &BigInt) -> f64 {
    let n_bits = numerator.bits();
    let d_bits = denominator.bits();
    
    // 避免除以过大的数导致下溢，或者过大的结果
    if d_bits > n_bits + 64 { return 0.0; }
    
    let n_high = extract_high_64(numerator);
    let d_high = extract_high_64(denominator);
    
    if d_high == 0.0 { return 0.0; }

    let shift = (n_bits as i32) - (d_bits as i32);
    let val = n_high / d_high;
    val * 2.0_f64.powi(shift)
}

fn extract_high_64(n: &BigInt) -> f64 {
    if n.is_zero() { return 0.0; }
    let bits = n.bits();
    if bits <= 64 {
        n.to_f64().unwrap_or(0.0)
    } else {
        let shifted = n >> (bits - 62);
        shifted.to_f64().unwrap_or(0.0)
    }
}

/// Projector (投影仪)
/// 
/// 实现了双重投影架构：
/// 1. Exact Projection: 基于 SHA-256，用于生成不可伪造的逻辑路径。
/// 2. Continuous Projection: 基于模形式特征，用于引导 VAPO 搜索。
pub struct Projector {
    pub p_base: u64, 
}

impl Projector {
    pub fn new(p_base: u64) -> Self {
        Self { p_base }
    }

    /// [Truth Layer] 精确投影 (Avalanche)
    /// [Fix] 使用 SHA-256 替代 DefaultHasher，提供密码学强度的“承诺”。
    /// 即使代数状态 S 发生 1 bit 的改变，输出的 digit 也会发生雪崩效应。
    pub fn project_exact(&self, state: &IdealClass, time: u64) -> u64 {
        let mut hasher = Sha256::new();
        
        // 注入代数结构的规范形式
        // 注意：BigInt 的 to_bytes_be 能够提供稳定的字节表示
        // 我们使用 "|" 分隔符防止拼接攻击 (虽然在固定结构下不常见，但为了严谨)
        hasher.update(state.a.to_bytes_be().1);
        hasher.update(b"|"); 
        hasher.update(state.b.to_bytes_be().1);
        hasher.update(b"|");
        hasher.update(state.c.to_bytes_be().1);
        hasher.update(b"|");
        
        // 注入时间/深度因子，确保每一步的投影都是独立的随机预言机
        hasher.update(time.to_be_bytes());

        let result = hasher.finalize();

        // 将 SHA-256 的前 8 个字节转换为 u64
        // 这保留了 64 位的熵，远高于 DefaultHasher，足以抵抗碰撞
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[0..8]);
        let raw_hash = u64::from_be_bytes(bytes);

        // 最终映射到 p_base 域
        // 注意：虽然取模会损失熵，但在 Logic Layer 我们只需要这么多信息。
        // 安全性由 raw_hash 的不可预测性保证。
        raw_hash % self.p_base
    }

    /// [Will Layer] 连续投影 (Lipschitz)
    /// 返回几何特征向量，用于 Residual 计算
    pub fn project_continuous(&self, state: &IdealClass) -> Vec<f64> {
        NavigationFeatures::extract(state).to_vector()
    }
}
