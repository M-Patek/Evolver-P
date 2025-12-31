use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use num_bigint::BigInt;

// [Import] 引入核心组件
use crate::soul::algebra::ClassGroupElement;
use crate::body::topology::VPuNNConfig;
use crate::dsl::stp_bridge::STPContext;
use crate::dsl::schema::ProofAction;
use crate::will::perturber::EnergyEvaluator;
use crate::will::optimizer;

// -------------------------------------------------------------------------
// 模块声明 (已清理无用模块)
// -------------------------------------------------------------------------
pub mod dsl;
pub mod soul;
pub mod body {
    pub mod topology;
    pub mod projection;
    pub mod decoder;
    pub mod adapter;
}
pub mod will {
    pub mod optimizer;
    pub mod perturber;
}

// 注意：control 和 interface 模块已被移除，因为它们属于旧架构。

// -------------------------------------------------------------------------
// 辅助结构体：能量评估桥接器
// -------------------------------------------------------------------------
struct StpBridge<'a> {
    context: &'a RefCell<STPContext>,
}

impl<'a> EnergyEvaluator for StpBridge<'a> {
    fn evaluate(&self, path: &[u64]) -> f64 {
        // [Logic Decoding]
        // 将代数路径的第一位映射为逻辑决策
        let decision_seed = path.get(0).unwrap_or(&0);
        
        // 偶数 -> Even (正确逻辑)
        // 奇数 -> Odd (错误逻辑)
        let action = if decision_seed % 2 == 0 {
            ProofAction::Define {
                symbol: "sum_truth".to_string(),
                hierarchy_path: vec!["Even".to_string()]
            }
        } else {
            ProofAction::Define {
                symbol: "sum_truth".to_string(),
                hierarchy_path: vec!["Odd".to_string()]
            }
        };

        let mut stp = self.context.borrow_mut();
        
        // 1. 尝试定义
        stp.calculate_energy(&action);

        // 2. 检查一致性 (Odd + Odd = Even)
        let check_action = ProofAction::Apply {
            theorem_id: "ModAdd".to_string(),
            inputs: vec!["n".to_string(), "m".to_string()],
            output_symbol: "sum_truth".to_string(),
        };

        stp.calculate_energy(&check_action)
    }
}

// -------------------------------------------------------------------------
// PyEvolver (API 暴露)
// -------------------------------------------------------------------------
#[pyclass]
pub struct PyEvolver {
    soul: ClassGroupElement, 
    body: VPuNNConfig,
    stp: RefCell<STPContext>, 
}

#[pymethods]
impl PyEvolver {
    #[new]
    fn new(p: u64, k: usize) -> Self {
        println!("🐱 PyEvolver Initializing with p={}, k={}...", p, k);

        let mut stp_ctx = STPContext::new();
        // 预设环境：n=Odd, m=Odd
        let setup_n = ProofAction::Define { 
            symbol: "n".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        };
        let setup_m = ProofAction::Define { 
            symbol: "m".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        };
        stp_ctx.calculate_energy(&setup_n);
        stp_ctx.calculate_energy(&setup_m);

        let discriminant = BigInt::from(-23);
        let identity_soul = ClassGroupElement::identity(&discriminant);
        let body_config = VPuNNConfig::new(k, p);

        PyEvolver {
            soul: identity_soul,
            body: body_config,
            stp: RefCell::new(stp_ctx),
        }
    }

    fn align(&mut self, context: String) -> Vec<u64> {
        let mut hasher = DefaultHasher::new();
        context.hash(&mut hasher);
        let seed = hasher.finish();
        
        self.soul = self.soul.evolve(seed);

        let evaluator = StpBridge { context: &self.stp };
        let optimized_soul = optimizer::optimize(&self.soul, &evaluator);

        self.soul = optimized_soul;
        
        let materialize = |state: &ClassGroupElement| -> Vec<u64> {
            let extract_u64 = |n: &BigInt| -> u64 {
                let (_sign, bytes) = n.to_bytes_le();
                if bytes.is_empty() { 0 } 
                else {
                    let mut buf = [0u8; 8];
                    let len = std::cmp::min(bytes.len(), 8);
                    buf[..len].copy_from_slice(&bytes[..len]);
                    u64::from_le_bytes(buf)
                }
            };
            vec![
                extract_u64(&state.a),
                extract_u64(&state.b),
                extract_u64(&state.c)
            ]
        };

        materialize(&self.soul)
    }
}

#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    Ok(())
}
