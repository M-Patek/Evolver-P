use crate::soul::algebra::IdealClass;
use crate::body::projection::Projector;
use crate::body::adapter;
use crate::dsl::stp_bridge::STPContext;
use num_traits::ToPrimitive;

pub trait Evaluator {
    fn evaluate(&self, state: &IdealClass) -> f64;
    fn name(&self) -> &'static str;
}

pub struct GeometricEvaluator;
impl Evaluator for GeometricEvaluator {
    fn evaluate(&self, state: &IdealClass) -> f64 {
        state.a.to_f64().unwrap_or(1e10)
    }
    fn name(&self) -> &'static str { "Geometric" }
}

/// STP 评估器 (Rigorous Evaluator)
/// 
/// 实现了 "Unified Energy Metric"：
/// Energy = Barrier(Exact) + Residual(Heuristic)
pub struct StpEvaluator {
    projector: Projector,
    action_count: usize,
    digits_per_action: usize,
}

impl StpEvaluator {
    pub fn new(projector: Projector, action_count: usize) -> Self {
        Self { 
            projector, 
            action_count,
            digits_per_action: 3, 
        }
    }
}

impl Evaluator for StpEvaluator {
    fn evaluate(&self, state: &IdealClass) -> f64 {
        let total_digits = self.action_count * self.digits_per_action;
        let mut current_state = state.clone();
        
        // --- 1. 计算离散势垒 (Discrete Barrier) ---
        // 使用 Project_Exact: 只有完美的代数状态才能通过 STP 检查
        let mut exact_path = Vec::with_capacity(total_digits);
        for t in 0..total_digits {
            exact_path.push(self.projector.project_exact(&current_state, t as u64));
            current_state = current_state.square();
        }
        
        let actions: Vec<_> = exact_path
            .chunks(self.digits_per_action)
            .map(|chunk| adapter::path_to_proof_action(chunk))
            .collect();

        let mut context = STPContext::new();
        let barrier_energy = context.calculate_energy(&actions);

        // 如果已经是完美逻辑 (Barrier == 0)，直接返回 0 (Done)
        if barrier_energy < 1e-6 {
            return 0.0;
        }

        // --- 2. 计算连续残差 (Continuous Residual) ---
        // 使用 Project_Heuristic: 提供梯度方向引导优化器进入正确的 Basin
        // 这里简化实现：我们只看第一步的启发式投影与目标的“距离”
        // (注：这只是一个示意，完整的 Residual 计算需要定义 Target Feature)
        
        // Reset state for heuristic check
        let heuristic_digit = self.projector.project_heuristic(state, 0);
        let residual_energy = (heuristic_digit as f64) * 0.001; // 极小的权重，仅作 Tie-breaker

        // Total J(S)
        barrier_energy + residual_energy
    }

    fn name(&self) -> &'static str {
        "STP (Split Projection)"
    }
}
