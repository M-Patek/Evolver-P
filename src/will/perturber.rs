use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Signed, Zero, One, ToPrimitive};
use rand::prelude::*;
use crate::soul::algebra::IdealClass;

pub struct Perturber {
    generators: Vec<IdealClass>,
}

impl Perturber {
    pub fn new(discriminant: &BigInt, count: usize) -> Self {
        let generators = generate_perturbations(discriminant, count);
        Self { generators }
    }

    /// [Updated] 返回 (新状态, 施加的扰动元)
    /// 用于 Trace 记录
    pub fn perturb_with_source(&self, state: &IdealClass) -> (IdealClass, IdealClass) {
        if self.generators.is_empty() {
            // Identity element placeholder logic would be needed here for strict correctness
            // For now, just clone state and return a dummy if empty
            return (state.clone(), state.clone()); 
        }

        let mut rng = rand::thread_rng();
        let gen = self.generators.choose(&mut rng).unwrap();
        
        if rng.gen_bool(0.5) {
            // S * g
            (state.compose(gen), gen.clone())
        } else {
            // S * g^-1
            let inv = gen.inverse();
            (state.compose(&inv), inv)
        }
    }
    
    // 保留旧接口兼容
    pub fn perturb(&self, state: &IdealClass) -> IdealClass {
        self.perturb_with_source(state).0
    }
}

// ... 辅助函数保持不变 ...
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
        if p_candidate > 10000 { break; } 
    }
    perturbations
}

fn try_create_prime_form(discriminant: &BigInt, p: u64) -> Option<IdealClass> {
    let p_bi = BigInt::from(p);
    let four_p = BigInt::from(4) * &p_bi;
    let target = discriminant.mod_floor(&four_p);
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
