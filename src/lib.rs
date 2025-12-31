use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use num_bigint::BigInt;

use crate::soul::algebra::ClassGroupElement;
use crate::body::topology::VPuNNConfig;
use crate::dsl::stp_bridge::STPContext;
use crate::dsl::schema::ProofAction;
use crate::will::perturber::EnergyEvaluator;
use crate::will::optimizer;
use crate::body::decoder;

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

struct StpBridge<'a> {
    context: &'a RefCell<STPContext>,
}

impl<'a> EnergyEvaluator for StpBridge<'a> {
    fn evaluate(&self, path: &[u64]) -> f64 {
        // [Logic Decoding & Binding Check]
        // 这一步是将代数路径映射到具体的逻辑假设 (Hypothesis)
        let decision_seed = path.get(0).unwrap_or(&0);
        
        // VAPO 正在尝试猜测 sum_truth 的值
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
        
        // [Critical Fix]: 完整性预检 (Sanity Check)
        // 在执行任何运算前，必须确保上下文环境是健康的。
        // 如果 n 和 m 丢失，说明上下文被破坏或初始化失败，必须返回高能惩罚，
        // 迫使 VAPO 意识到这是一个极其糟糕的状态。
        if !stp.state.contains_key("n") || !stp.state.contains_key("m") {
            // println!("DEBUG: Context corrupted! Missing 'n' or 'm'.");
            return 100.0; // High Energy Penalty
        }

        // 1. 设置假设 (Set Hypothesis)
        // 这一步通常返回 0.0，因为 Define 是合法的
        stp.calculate_energy(&action);

        // 2. 验证假设 (Verify Hypothesis)
        // 检查: ModAdd(n, m) == sum_truth ?
        // 由于我们在 stp_bridge.rs 中加了严厉的 None 检查，
        // 如果 n/m/sum_truth 缺失，这里会返回 100.0。
        // 如果逻辑错误 (e.g. Odd+Odd=Odd)，返回 1.0。
        // 只有逻辑正确，才返回 0.0。
        let check_action = ProofAction::Apply {
            theorem_id: "ModAdd".to_string(),
            inputs: vec!["n".to_string(), "m".to_string()],
            output_symbol: "sum_truth".to_string(),
        };

        stp.calculate_energy(&check_action)
    }
}

#[pyclass]
pub struct PyEvolver {
    soul: ClassGroupElement, 
    body: VPuNNConfig,
    // 使用 RefCell 允许内部可变性，因为 Python 调用是独占的
    stp: RefCell<STPContext>, 
}

#[pymethods]
impl PyEvolver {
    #[new]
    fn new(p: u64, k: usize) -> Self {
        println!("🐱 PyEvolver Initializing with p={}, k={}...", p, k);

        let mut stp_ctx = STPContext::new();
        
        // [Initialization]
        // 这里定义了公理/前提：n 是奇数，m 是奇数。
        // 这些状态必须持久化在 stp_ctx 中。
        let setup_n = ProofAction::Define { 
            symbol: "n".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        };
        let setup_m = ProofAction::Define { 
            symbol: "m".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        };
        
        // 执行初始化，不应报错
        stp_ctx.calculate_energy(&setup_n);
        stp_ctx.calculate_energy(&setup_m);

        // 验证初始化是否成功
        if !stp_ctx.state.contains_key("n") || !stp_ctx.state.contains_key("m") {
            panic!("❌ Critical Error: Failed to initialize mathematical context!");
        }

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
        // 1. 种子注入 (Context Seeding)
        let mut hasher = DefaultHasher::new();
        context.hash(&mut hasher);
        let seed = hasher.finish();
        
        self.soul = self.soul.evolve(seed);

        // 2. 优化 (Optimization)
        // 构造 Evaluator，它借用了 self.stp
        let evaluator = StpBridge { context: &self.stp };
        
        // 运行 VAPO
        // 此时如果 Evaluator 发现状态不对，会返回 100.0，
        // 迫使 VAPO 继续寻找更好的扰动。
        let optimized_soul = optimizer::optimize(&self.soul, &self.body, &evaluator);

        self.soul = optimized_soul;
        
        // 3. 物质化 (Materialization)
        decoder::materialize_path(&self.soul, &self.body)
    }
}

#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    Ok(())
}
