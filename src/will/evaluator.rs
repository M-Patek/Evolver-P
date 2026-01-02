use crate::soul::algebra::IdealClass;
use crate::body::projection::Projector;
use crate::body::adapter;
use crate::dsl::stp_bridge::STPContext;
use num_traits::ToPrimitive;

/// Evaluator 定义了“意志”优化的目标函数接口。
pub trait Evaluator {
    /// 计算给定代数状态的能量。
    /// Lower energy = Better state.
    fn evaluate(&self, state: &IdealClass) -> f64;
    
    fn name(&self) -> &'static str;
}

/// 几何评估器 (Geometric Evaluator)
/// 
/// [Heuristic / Fast Mode]
/// 不进行逻辑展开，仅根据代数状态在模曲线上的几何位置进行启发式打分。
/// 例如：倾向于寻找“约化形式”(Reduced Forms) 或者特定的 \tau 分布。
pub struct GeometricEvaluator;

impl Evaluator for GeometricEvaluator {
    fn evaluate(&self, state: &IdealClass) -> f64 {
        // 启发式：偏好 a 较小的解 (通常意味着约化得更好)
        // 这是一个简单的 Proxy Metric。
        state.a.to_f64().unwrap_or(1e10)
    }

    fn name(&self) -> &'static str {
        "Geometric (Heuristic)"
    }
}

/// STP 评估器 (Rigorous Evaluator)
/// 
/// [Rigorous / Slow Mode]
/// 完整的闭环验证：
/// Soul (State) -> Body (Digits) -> Adapter (Actions) -> DSL (STP Energy)
pub struct StpEvaluator {
    projector: Projector,
    action_count: usize,
    digits_per_action: usize,
}

impl StpEvaluator {
    /// 创建 STP 评估器
    /// 
    /// # Arguments
    /// * `projector` - 用于将代数状态投影为数字序列
    /// * `action_count` - 每次评估需要生成的逻辑动作数量 (序列长度)
    pub fn new(projector: Projector, action_count: usize) -> Self {
        Self { 
            projector, 
            action_count,
            digits_per_action: 3, // 假设每个动作平均消耗 3 个数字 (Op, Param1, Param2)
        }
    }
}

impl Evaluator for StpEvaluator {
    fn evaluate(&self, state: &IdealClass) -> f64 {
        // 1. Body Projection: Materialize algebra into digital sequence
        // 需要生成的总数字量
        let total_digits = self.action_count * self.digits_per_action;
        let digits = self.projector.project_sequence(state, total_digits);

        // 2. Adapter Decoding: Convert digits to Logic Actions
        // 将线性数字流切分为动作块
        let actions: Vec<_> = digits
            .chunks(self.digits_per_action)
            .map(|chunk| adapter::path_to_proof_action(chunk))
            .collect();

        // 3. DSL Verification: Calculate Logic Energy via STP
        let mut context = STPContext::new();
        let energy = context.calculate_energy(&actions);

        // 4. Combined Energy: Barrier + Residual
        // 这里目前只返回 STP 能量 (离散势垒)，
        // 实际应用中可能混合几何残差以提供梯度引导。
        energy
    }

    fn name(&self) -> &'static str {
        "STP (Rigorous)"
    }
}
