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

#[pyclass]
pub struct PyEvolver {
    p: u64, 
    k: u64, 
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

    /// 对齐函数
    /// Context -> Algebraic Seed -> Search -> Valid Logic Path
    #[pyo3(signature = (context, mode="prove", depth=16))]
    pub fn align(&self, context: String, mode: &str, depth: usize) -> PyResult<Vec<u64>> {
        // 1. 生成初始种子
        let seed = IdealClass::from_hash(&context, self.p); 
        
        // 2. 初始化投影仪
        // [Fix] Projector 现在包含 SHA-256 逻辑
        let eval_projector = Projector::new(self.p);

        // 3. 构造 Evaluator 和 Dynamics
        // [Fix] 显式构建 target_features。
        // 这里我们使用 Context Hash 生成的种子的连续特征作为“目标意图”。
        // 这意味着 VAPO 会倾向于寻找几何上接近初始 Context，但逻辑上合法的状态。
        let target_features = eval_projector.project_continuous(&seed);

        let (dynamics, evaluator): (Box<dyn TimeEvolution>, Box<dyn Evaluator>) = match mode {
            "fast" | "native" => (
                Box::new(IdentityDynamics),   
                Box::new(GeometricEvaluator), 
            ),
            "prove" | "vdf" => (
                Box::new(VDFDynamics::new(self.vdf_difficulty)), 
                // [Fix] 参数对齐：传入 projector, depth, target_features
                Box::new(StpEvaluator::new(eval_projector, depth, target_features)), 
            ),
            "hybrid" => (
                Box::new(VDFDynamics::new(self.vdf_difficulty)), 
                Box::new(GeometricEvaluator),                    
            ),
            _ => return Err(pyo3::exceptions::PyValueError::new_err(
                "Unknown mode. Available modes: 'fast', 'prove', 'hybrid'",
            )),
        };

        // 4. 执行意志搜索 (The Will)
        let optimizer = VapoOptimizer::new(evaluator, self.search_steps);
        
        // [Fix] 处理 search 返回的元组 (state, trace)
        let (optimized_state, _trace) = optimizer.search(&seed);
        // 注意：在生产环境中，_trace 应该被序列化并返回给 Python 端作为 ProofBundle。
        // 这里为了简化接口，只返回逻辑路径。

        // 5. 显化躯体 (The Body)
        let mut logic_path = Vec::with_capacity(depth);
        let mut state = optimized_state;
        
        // 输出阶段使用全新的 Projector 实例（尽管是无状态的，但保持逻辑清晰）
        let out_projector = Projector::new(self.p);

        for t in 0..depth {
            // 使用 project_exact (SHA-256) 生成最终路径
            let digit = out_projector.project_exact(&state, t as u64);
            logic_path.push(digit);

            state = dynamics.next(&state);
        }

        Ok(logic_path)
    }
}

#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    Ok(())
}
