// src/lib.rs
// 核心库入口: 连接 Soul, Will 和 Body

use pyo3::prelude::*;
use num_bigint::BigInt;
use crate::soul::algebra::ClassGroupElement;
use crate::will::optimizer::VapoOptimizer;
use crate::body::topology::{VPuNNConfig, project_state_to_digits};
use crate::dsl::stp_bridge::STPContext;
use crate::will::perturber::EnergyEvaluator;
use crate::dsl::schema::{self, ProofAction};
use crate::body::adapter::path_to_proof_action;

// 引入新的导航模块
use crate::body::navigator::NavigationFeatures;

pub mod soul;
pub mod will;
pub mod body;
pub mod dsl;

// ==========================================
// [Refactor] STPEvaluator: 基于模特征的评估
// ==========================================

struct STPEvaluator;

impl EnergyEvaluator for STPEvaluator {
    fn evaluate(&self, digits: &[u64]) -> f64 {
        // 1. 翻译: 将数字信号 (Will) 转换为逻辑动作 (Logic)
        let action = path_to_proof_action(digits);
        let actions = vec![action]; 

        // 2. 计算: 调用 STP 引擎计算逻辑能量
        let mut stp = STPContext::new();
        stp.calculate_energy(&actions)
    }
}

// ==========================================
// PyProofBundle 定义
// ==========================================

#[pyclass(name = "ProofBundle")]
#[derive(Clone)]
pub struct PyProofBundle {
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

// ==========================================
// PyEvolver 主逻辑
// ==========================================

#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    m.add_class::<PyProofBundle>()?; 
    Ok(())
}

#[pyclass]
struct PyEvolver {
    k: usize,
}

#[pymethods]
impl PyEvolver {
    #[new]
    fn new(k: usize) -> PyResult<Self> {
        Ok(PyEvolver { k })
    }

    fn align(&self, context: String) -> PyResult<PyProofBundle> {
        // 1. Inception
        let (seed, universe) = ClassGroupElement::spawn_universe(&context);
        
        let ctx_hash = universe.context_hash;
        let discriminant_hex = universe.discriminant.to_str_radix(16);
        let start_seed_clone = seed.clone();
        
        // 2. The Will: Optimizer
        let mut optimizer = VapoOptimizer::new(seed);
        
        let mut best_energy = f64::MAX;
        let mut best_logic_path_strings = Vec::new();
        let mut best_state = start_seed_clone.clone();
        
        let config = VPuNNConfig {
            depth: self.k,
            p_base: 997,
            layer_decay: 0.9,
        };

        let evaluator = STPEvaluator;
        let max_iterations = 50; 

        // 3. Evolution Loop
        for _ in 0..max_iterations {
            let (candidate_state, gen_idx) = optimizer.perturb();
            
            // [NEW] Canonical Modular Projection
            // 使用新的 navigator -> topology 管道
            let mut path_digits = Vec::new();
            
            // 为了生成逻辑路径，我们在这里做一个简单的时间展开 (Time Unfolding)
            // 注意：Will 的搜索是在静态图上，但评估需要看到这一点的"逻辑结果"
            let mut time_state = candidate_state.clone();
            for t in 0..config.depth {
                 // 在每一步时间演化中，投影当前状态
                 let digit = project_state_to_digits(&time_state, &config, t as u64);
                 path_digits.push(digit);
                 
                 // 简单的群操作模拟时间演化 (Repeated Squaring or simple composition)
                 // 这里简化为保持状态不变或简单变换，视具体 Time Dynamics 定义而定
                 // 真实的 VDF 应该在这里做 square()，为了演示保持简单
                 time_state = time_state.square(); 
            }
            
            // 评估能量
            let energy = evaluator.evaluate(&path_digits);

            // [NEW] Heuristic Guidance (Navigator)
            // 除了 STP 的硬能量，我们还可以加入 Navigation Features 的几何势能
            // 这里暂且只用 Logic Energy，但底层机制已经 ready
            
            if energy < best_energy {
                best_energy = energy;
                best_state = candidate_state.clone();
                
                let action = path_to_proof_action(&path_digits);
                best_logic_path_strings = vec![format!("{:?}", action)];
                
                optimizer.accept(candidate_state, gen_idx);
            } else {
                optimizer.reject();
            }

            if best_energy == 0.0 {
                break;
            }
        }

        // 4. Revelation
        let generator_spec = schema::GeneratorSpec {
            algorithm_version: "v2.1_modular_feature".to_string(), // Updated version
            count: 50,
            max_norm: None,
        };

        let bundle = schema::ProofBundle {
            context_hash: ctx_hash,
            discriminant_hex: discriminant_hex,
            start_seed_a: start_seed_clone.a.to_string(),
            final_state_a: best_state.a.to_string(),
            generator_spec: generator_spec,
            perturbation_trace: optimizer.trace.clone(),
            logic_path: best_logic_path_strings,
            energy: best_energy,
        };

        Ok(PyProofBundle::from(bundle))
    }
}
