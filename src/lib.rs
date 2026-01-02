use pyo3::prelude::*;
use crate::soul::algebra::IdealClass;
use crate::soul::dynamics::{TimeEvolution, IdentityDynamics, VDFDynamics};
use crate::will::optimizer::VapoOptimizer;
use crate::will::evaluator::{Evaluator, GeometricEvaluator, StpEvaluator};
use crate::body::projection::Projector;

pub mod soul;
pub mod will;
pub mod body;
pub mod dsl;

/// Python Interface for Evolver (新一代)
/// 
/// 编排层：负责组装 Soul, Will, Body 并暴露给 Python。
#[pyclass]
pub struct PyEvolver {
    // System Parameters
    p: u64, // 投影基数 (Prime Base)
    k: u64, // 冗余参数 (保留接口兼容性)
    
    // Configuration
    vdf_difficulty: usize,
    search_steps: usize,
}

#[pymethods]
impl PyEvolver {
    #[new]
    #[pyo3(signature = (p, k, vdf_difficulty=None, search_steps=None))]
    pub fn new(p: u64, k: u64, vdf_difficulty: Option<usize>, search_steps: Option<usize>) -> Self {
        PyEvolver {
            p,
            k,
            vdf_difficulty: vdf_difficulty.unwrap_or(1),
            search_steps: search_steps.unwrap_or(100),
        }
    }

    /// Align logic with the given context.
    /// 
    /// # Arguments
    /// * `context` - 输入提示词
    /// * `mode` - "fast" (几何直觉) | "prove" (STP 严谨验证) | "hybrid"
    /// * `depth` - 生成逻辑路径的长度
    #[pyo3(signature = (context, mode="prove", depth=16))]
    pub fn align(&self, context: String, mode: &str, depth: usize) -> PyResult<Vec<u64>> {
        // 1. Initialization (The Soul)
        // 从上下文哈希中通过“宇宙大爆炸”生成初始代数种子
        let seed = IdealClass::from_hash(&context, self.p); 

        // 2. Strategy Selection (The Decoupling)
        // 决定使用哪种“时间流逝”方式 (Dynamics) 和“价值判断”标准 (Evaluator)
        
        // 为了创建 StpEvaluator，我们需要一个专用的 Projector 实例
        // 因为 Evaluator trait object 需要拥有它
        let eval_projector = Projector::new(self.p);

        let (dynamics, evaluator): (Box<dyn TimeEvolution>, Box<dyn Evaluator>) = match mode {
            "fast" | "native" => (
                Box::new(IdentityDynamics),   // Time: 瞬时 (无 VDF)
                Box::new(GeometricEvaluator), // Will: 直觉 (几何形状)
            ),
            "prove" | "vdf" => (
                Box::new(VDFDynamics::new(self.vdf_difficulty)), // Time: 沉重 (VDF)
                // Will: 严谨 (STP 逻辑检查)
                // 注意：这里我们让 Evaluator 预演 `depth` 步来计算能量
                Box::new(StpEvaluator::new(eval_projector, depth)), 
            ),
            "hybrid" => (
                Box::new(VDFDynamics::new(self.vdf_difficulty)), // Time: 沉重
                Box::new(GeometricEvaluator),                    // Will: 直觉 (快速搜索)
            ),
            _ => return Err(pyo3::exceptions::PyValueError::new_err(
                "Unknown mode. Available modes: 'fast', 'prove', 'hybrid'",
            )),
        };

        // 3. The Will (Optimization)
        // 优化器在 Cayley 图上游走，寻找能量最低的种子
        let optimizer = VapoOptimizer::new(evaluator, self.search_steps);
        let optimized_state = optimizer.search(&seed);

        // 4. Materialization (The Body)
        // 将优化后的种子展开为时间序列
        let mut logic_path = Vec::with_capacity(depth);
        let mut state = optimized_state;
        
        // 用于最终输出的投影仪
        let out_projector = Projector::new(self.p);

        for t in 0..depth {
            // A. Projection: \Psi(S_t) -> d_t
            let digit = out_projector.project(&state, t as u64);
            logic_path.push(digit);

            // B. Evolution: S_{t+1} = Dynamics(S_t)
            // 这里使用了策略模式注入的 dynamics (Identity 或 VDF)
            state = dynamics.next(&state);
        }

        Ok(logic_path)
    }
}

/// Python 模块定义
#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    Ok(())
}
