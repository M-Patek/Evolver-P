use crate::soul::algebra::IdealClass;
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use num_traits::ToPrimitive;
use num_bigint::BigInt;

/// 导航特征 (Navigation Features)
/// 用于启发式搜索的平滑几何特征。
/// 现在它是公开的，作为 VAPO 的“视觉系统”。
#[derive(Debug, Clone, PartialEq)]
pub struct NavigationFeatures {
    pub cos_x: f64,
    pub sin_x: f64,
    pub log_y: f64,
}

impl NavigationFeatures {
    /// 将特征扁平化为向量，用于计算欧氏距离
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

// 辅助函数保持不变
fn bigint_log_e(n: &BigInt) -> f64 {
    if n.sign() == num_bigint::Sign::NoSign {
        return f64::NEG_INFINITY;
    }
    let bits = n.bits();
    (bits as f64) * 2.0_f64.ln()
}

fn bigint_to_scaled_f64(numerator: &BigInt, denominator: &BigInt) -> f64 {
    let n_bits = numerator.bits();
    let d_bits = denominator.bits();
    if d_bits > n_bits + 64 { return 0.0; }
    
    let n_high = extract_high_64(numerator);
    let d_high = extract_high_64(denominator);
    
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
pub struct Projector {
    pub p_base: u64, 
}

impl Projector {
    pub fn new(p_base: u64) -> Self {
        Self { p_base }
    }

    /// [Truth Layer] 精确投影 (Avalanche)
    pub fn project_exact(&self, state: &IdealClass, time: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        state.a.hash(&mut hasher);
        state.b.hash(&mut hasher);
        state.c.hash(&mut hasher);
        time.hash(&mut hasher);
        hasher.finish() % self.p_base
    }

    /// [Will Layer] 连续投影 (Lipschitz)
    /// 返回几何特征向量，用于 Residual 计算
    pub fn project_continuous(&self, state: &IdealClass) -> Vec<f64> {
        NavigationFeatures::extract(state).to_vector()
    }

    /// 旧的启发式投影 (Bucket化)，保留用于兼容性
    pub fn project_heuristic(&self, state: &IdealClass, time: u64) -> u64 {
        let features = NavigationFeatures::extract(state);
        let bucket_cos = (features.cos_x * 1000.0) as i64;
        let bucket_sin = (features.sin_x * 1000.0) as i64;
        let bucket_log_y = (features.log_y * 10.0) as i64;
        
        let mut hasher = DefaultHasher::new();
        bucket_cos.hash(&mut hasher);
        bucket_sin.hash(&mut hasher);
        bucket_log_y.hash(&mut hasher);
        time.hash(&mut hasher);
        hasher.finish() % self.p_base
    }
}
