use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use num_bigint::{BigInt, Sign};
use num_traits::{Num, Zero};
use crate::soul::algebra::ClassGroupElement;
use crate::will::optimizer::VapoOptimizer;
use crate::body::decoder::BodyProjector;
use crate::body::adapter::SemanticAdapter;
use crate::dsl::stp_bridge::STPContext;
use crate::will::perturber::EnergyEvaluator;
use crate::dsl::schema; // 引入 schema 定义

pub mod soul;
pub mod will;
pub mod body;
pub mod dsl;

// 定义一个简单的 EnergyEvaluator 实现
struct STPEvaluator;
impl EnergyEvaluator for STPEvaluator {
    fn evaluate(&self, actions: &[crate::body::adapter::ProofAction]) -> f64 {
        let mut stp = STPContext::new();
        stp.calculate_energy(actions)
    }
}

/// Python 暴露的 ProofBundle 包装器
/// 对应 src/dsl/schema.rs 中的标准定义
#[pyclass(name = "ProofBundle")]
#[derive(Clone)]
pub struct PyProofBundle {
    // 内部持有 schema 中的标准结构，或者直接映射字段
    // 为了 PyO3 的易用性，这里直接展开字段，但保证与 schema.rs 1:1 对应
    
    #[pyo3(get)]
    pub context_hash: String,
    
    #[pyo3(get)]
    pub discriminant_hex: String,
    
    #[pyo3(get)]
    pub start_seed_a: String,
    
    #[pyo3(get)]
    pub final_state_a: String,
    
    #[pyo3(get)]
    pub perturbation_trace: Vec<usize>,
    
    #[pyo3(get)]
    pub logic_path: Vec<String>,
    
    #[pyo3(get)]
    pub energy: f64,
}

impl From<schema::ProofBundle> for PyProofBundle {
    fn from(b: schema::ProofBundle) -> Self {
        PyProofBundle {
            context_hash: b.context_hash,
            discriminant_hex: b.discriminant_hex,
            start_seed_a: b.start_seed_a,
            final_state_a: b.final_state_a,
            perturbation_trace: b.perturbation_trace,
            logic_path: b.logic_path,
            energy: b.energy,
        }
    }
}

#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    m.add_class::<PyProofBundle>()?; 
    Ok(())
}

#[pyclass]
struct PyEvolver {
    discriminant: BigInt,
    k: usize,
}

#[pymethods]
impl PyEvolver {
    #[new]
    fn new(discriminant_hex: String, k: usize) -> PyResult<Self> {
        let delta = BigInt::from_str_radix(&discriminant_hex, 16)
            .map_err(|e| PyValueError::new_err(format!("Invalid hex discriminant: {}", e)))?;

        if delta >= BigInt::zero() {
            return Err(PyValueError::new_err("Discriminant must be negative."));
        }
        
        let rem = &delta % 4;
        let rem_val = if rem < BigInt::zero() { rem + 4 } else { rem };
        if rem_val != BigInt::zero() && rem_val != BigInt::from(1) {
             return Err(PyValueError::new_err("Discriminant must be 0 or 1 mod 4."));
        }

        Ok(PyEvolver { 
            discriminant: delta, 
            k 
        })
    }

    /// 对齐逻辑 (Align Logic)
    fn align(&self, context: String) -> PyResult<PyProofBundle> {
        // 1. Inception (SHA-256 Hash Alignment)
        let (seed, ctx_hash) = ClassGroupElement::from_context(&context, &self.discriminant);
        let start_seed_clone = seed.clone();
        
        // 2. The Will (Chaotic Deterministic Search)
        let mut optimizer = VapoOptimizer::new(seed);
        
        let mut best_energy = f64::MAX;
        let mut best_path_digits = Vec::new();
        let mut best_actions_strings = Vec::new();
        let mut best_state = start_seed_clone.clone();
        
        let p_projection = 997; 
        let evaluator = STPEvaluator;
        let max_iterations = 50;

        // 3. Evolution Loop
        for _ in 0..max_iterations {
            let (candidate_state, gen_idx) = optimizer.perturb();
            
            let path_digits = BodyProjector::project(&candidate_state, self.k, p_projection);
            let logic_actions = SemanticAdapter::materialize(&path_digits);
            let energy = evaluator.evaluate(&logic_actions);

            if energy < best_energy {
                best_energy = energy;
                best_path_digits = path_digits;
                best_state = candidate_state.clone();
                best_actions_strings = logic_actions.iter().map(|a| format!("{:?}", a)).collect();
                
                optimizer.accept(candidate_state, gen_idx);
            } else {
                optimizer.reject();
            }

            if best_energy == 0.0 {
                break;
            }
        }

        // 4. Revelation
        // 构建标准 Schema 定义的 Bundle
        let bundle = schema::ProofBundle {
            context_hash: ctx_hash, // 这里是 algebra.rs 返回的 SHA-256 Hex
            discriminant_hex: self.discriminant.to_str_radix(16),
            start_seed_a: start_seed_clone.a.to_string(),
            final_state_a: best_state.a.to_string(),
            perturbation_trace: optimizer.trace.clone(),
            logic_path: best_actions_strings,
            energy: best_energy,
        };

        // 转换为 Python 包装类
        Ok(PyProofBundle::from(bundle))
    }
}
