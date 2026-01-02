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
// [Refactor] 引入 Tracer 以解耦搜索与证明记录
use crate::will::tracer::{SilentTracer, ProvenTracer, WillTracer};

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
// [NEW] EvolvedPath: 轻量级原生输出
// ==========================================
/// 原生逻辑路径输出，不包含繁重的 Proof 元数据。
/// 适用于不需要链上验证的常规生成场景。
#[pyclass(name = "EvolvedPath")]
#[derive(Clone)]
pub struct PyEvolvedPath {
    #[pyo3(get)]
    pub logic_path: Vec<String>,
    #[pyo3(get)]
    pub energy: f64,
}

// ==========================================
// PyProofBundle 定义 (保留用于可验证场景)
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
    m.add_class::<PyEvolvedPath>()?; // 注册新类
    Ok(())
}

#[pyclass]
struct PyEvolver {
    k: usize,
}

impl PyEvolver {
    /// 内部通用演化循环
    /// 使用泛型 T: WillTracer 来适配 "静默模式" 和 "证明模式"
    /// 避免了代码重复。
    fn evolve<T: WillTracer>(
        &self, 
        context: &str, 
        tracer: &mut T
    ) -> (f64, Vec<String>, ClassGroupElement, ClassGroupElement, String, String) {
        // 1. Inception
        let (seed, universe) = ClassGroupElement::spawn_universe(context);
        
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
            
            // Canonical Modular Projection & Time Unfolding
            let mut path_digits = Vec::new();
            let mut time_state = candidate_state.clone();
            for t in 0..config.depth {
                 let digit = project_state_to_digits(&time_state, &config, t as u64);
                 path_digits.push(digit);
                 time_state = time_state.square(); 
            }
            
            let energy = evaluator.evaluate(&path_digits);

            if energy < best_energy {
                best_energy = energy;
                best_state = candidate_state.clone();
                
                let action = path_to_proof_action(&path_digits);
                best_logic_path_strings = vec![format!("{:?}", action)];
                
                // [Decoupled]: 这里不再直接 push 到 trace，而是通过接口通知
                optimizer.accept(tracer, candidate_state, gen_idx);
            } else {
                optimizer.reject(tracer);
            }

            if best_energy == 0.0 {
                break;
            }
        }

        (best_energy, best_logic_path_strings, best_state, start_seed_clone, ctx_hash, discriminant_hex)
    }
}

#[pymethods]
impl PyEvolver {
    #[new]
    fn new(k: usize) -> PyResult<Self> {
        Ok(PyEvolver { k })
    }

    /// 默认对齐方法：原生轻量模式
    /// 仅返回逻辑路径，不记录 Proof Trace，性能最优。
    fn align(&self, context: String) -> PyResult<PyEvolvedPath> {
        // 使用 SilentTracer，零开销
        let mut tracer = SilentTracer;
        
        let (energy, logic_path, _, _, _, _) = self.evolve(&context, &mut tracer);

        Ok(PyEvolvedPath {
            logic_path,
            energy,
        })
    }

    /// 带证明的对齐方法：可验证模式
    /// 记录完整轨迹，返回 ProofBundle，可用于构建 Proof of Will。
    fn align_with_proof(&self, context: String) -> PyResult<PyProofBundle> {
        // 使用 ProvenTracer，记录每一步
        let mut tracer = ProvenTracer::new();
        
        let (energy, logic_path, final_state, start_seed, ctx_hash, disc_hex) = self.evolve(&context, &mut tracer);

        // 获取生成元规格 (用于验证者重构环境)
        let generator_spec = VapoOptimizer::default_spec();

        let bundle = schema::ProofBundle {
            context_hash: ctx_hash,
            discriminant_hex: disc_hex,
            start_seed_a: start_seed.a.to_string(),
            final_state_a: final_state.a.to_string(),
            generator_spec,
            perturbation_trace: tracer.trace, // 从 tracer 中提取轨迹
            logic_path,
            energy,
        };

        Ok(PyProofBundle::from(bundle))
    }
}
