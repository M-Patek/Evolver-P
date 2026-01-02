use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Signed, Zero, One, ToPrimitive};
use rand::prelude::*;
use crate::soul::algebra::IdealClass;

/// 扰动器 (Perturber)
/// 负责在 Cayley 图上生成邻居节点。
/// 它预先计算并缓存了一组生成元 (Generators)，用于对当前状态施加微小的代数扰动。
pub struct Perturber {
    /// 生成元集合 \mathcal{P}
    generators: Vec<IdealClass>,
}

impl Perturber {
    /// 创建一个新的扰动器
    /// 
    /// # Arguments
    /// * `discriminant` - 宇宙判别式 \Delta
    /// * `count` - 需要生成的生成元数量 (影响图的连通度)
    pub fn new(discriminant: &BigInt, count: usize) -> Self {
        let generators = generate_perturbations(discriminant, count);
        Self { generators }
    }

    /// 对当前状态施加扰动 (Apply Perturbation)
    /// S_{next} = S_{curr} * g^{ \pm 1 }
    /// 
    /// 相当于在 Cayley 图上随机游走一步。
    pub fn perturb(&self, state: &IdealClass) -> IdealClass {
        if self.generators.is_empty() {
            return state.clone();
        }

        let mut rng = rand::thread_rng();
        
        // 1. 随机选择一个生成元
        let gen = self.generators.choose(&mut rng).unwrap();
        
        // 2. 随机决定方向 (正向或逆向)
        // 在 Cayley 图中，边通常是无向的 (如果包含逆元)
        if rng.gen_bool(0.5) {
            state.compose(gen)
        } else {
            // Compose with inverse: S * g^{-1}
            state.compose(&gen.inverse())
        }
    }
}

/// 生成扰动元 (Internal Helper)
/// 寻找在 \Delta 中分裂的小素数 p，构造理想类 (p, b, c)。
fn generate_perturbations(discriminant: &BigInt, count: usize) -> Vec<IdealClass> {
    let mut perturbations = Vec::with_capacity(count);
    let mut p_candidate = 2u64;

    while perturbations.len() < count {
        if is_prime(p_candidate) {
            if let Some(element) = try_create_prime_form(discriminant, p_candidate) {
                perturbations.push(element);
            }
        }
        p_candidate += 1;
        
        // 安全熔断，防止死循环
        if p_candidate > 10000 { break; } 
    }

    perturbations
}

/// 尝试为素数 p 构造一个类群元素 (p, b, c)
fn try_create_prime_form(discriminant: &BigInt, p: u64) -> Option<IdealClass> {
    let p_bi = BigInt::from(p);
    let four_p = BigInt::from(4) * &p_bi;
    
    // b^2 ≡ Delta (mod 4p)
    let target = discriminant.mod_floor(&four_p);

    // 优化搜索起点：b 的奇偶性需与 Delta 相同
    let start = if discriminant.is_odd() { 1 } else { 0 };
    let step = 2;
    let limit = 4 * p; 
    
    let mut b_curr = start;
    while b_curr < limit {
        let b_bi = BigInt::from(b_curr);
        let b_sq = &b_bi * &b_bi;

        if b_sq.mod_floor(&four_p) == target {
            let numerator = &b_sq - discriminant;
            let c_val = numerator / &four_p;

            return Some(IdealClass::new(p_bi, b_bi, c_val));
        }
        b_curr += step;
    }

    None
}

fn is_prime(n: u64) -> bool {
    if n <= 1 { return false; }
    if n <= 3 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}
