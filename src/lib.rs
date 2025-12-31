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
// 模块声明
// -------------------------------------------------------------------------
pub mod control;
pub mod dsl;
pub mod interface;
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
// [Task 4.3] PyEvolver (原 AUEEngine)
// -------------------------------------------------------------------------
/// Evolver 的 Python 接口类。
/// 对外隐藏了复杂的群运算 (Soul) 和 矩阵逻辑 (STP)，
/// 只暴露极其简单的初始化和对齐接口。
#[pyclass]
pub struct PyEvolver {
    // 内部状态不对 Python 可见
    soul: ClassGroupElement, 
    body: VPuNNConfig,
    stp: RefCell<STPContext>, 
}

#[pymethods]
impl PyEvolver {
    /// 构造函数
    /// 
    /// # 参数
    /// * `p` (u64): 投影基底 (Projection Base)，通常是一个大素数 (如 409)。
    /// * `k` (usize): 神经网络/决策树的深度 (Depth)，决定了逻辑的复杂度 (如 19)。
    #[new]
    fn new(p: u64, k: usize) -> Self {
        println!("🐱 PyEvolver Initializing with p={}, k={}...", p, k);

        // 1. 初始化 STP 上下文 (逻辑裁判)
        let mut stp_ctx = STPContext::new();
        
        // 预设环境：n=Odd, m=Odd (模拟用户输入解析后的状态)
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

        // 2. 初始化灵魂 (代数核心)
        // 使用判别式 Delta = -23，这是最小的类数为 3 的虚二次域判别式之一。
        // 它足够简单，适合作为 demo 的“出厂设置”。
        let discriminant = BigInt::from(-23);
        let identity_soul = ClassGroupElement::identity(&discriminant);

        // 3. 初始化肉体 (拓扑配置)
        // 使用用户传入的参数 p 和 k
        let body_config = VPuNNConfig::new(k, p);

        PyEvolver {
            soul: identity_soul,
            body: body_config,
            stp: RefCell::new(stp_ctx),
        }
    }

    /// 核心接口：对齐 (Align)
    ///
    /// 接收自然语言上下文，返回修正后的逻辑路径。
    /// Python 端不需要知道什么是 ClassGroupElement，只需要拿到结果列表。
    ///
    /// # 参数
    /// * `context` (str): 用户的输入上下文 (Prompt)。
    ///
    /// # 返回
    /// * `List[int]`: 逻辑证明路径 (Proof Path)。
    fn align(&mut self, context: String) -> Vec<u64> {
        // 1. 感知：将上下文哈希化为种子
        let mut hasher = DefaultHasher::new();
        context.hash(&mut hasher);
        let seed = hasher.finish();
        
        // 2. 直觉：灵魂演化
        // 这一步是确定性的：相同的上下文永远产生相同的初始直觉。
        self.soul = self.soul.evolve(seed);

        // 3. 意志：VAPO 优化
        // 在代数空间中搜索能量为 0 的状态。
        // 这里使用了 RefCell 的借用机制来连接 STP。
        let evaluator = StpBridge { context: &self.stp };
        let optimized_soul = optimizer::optimize(&self.soul, &evaluator);

        // 4. 承诺：更新状态
        self.soul = optimized_soul;
        
        // 5. 行动：投影回现实
        // 将抽象的代数对象转化为具体的数字路径
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

        let proof_path = materialize(&self.soul);
        
        // 返回纯粹的数据给 Python，隐藏背后的代数复杂性
        proof_path
    }
}

#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    // 注册 PyEvolver 类
    m.add_class::<PyEvolver>()?;
    Ok(())
}
