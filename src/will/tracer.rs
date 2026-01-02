use serde::{Deserialize, Serialize};
use crate::soul::algebra::IdealClass;

pub type Energy = f64;

/// 验证结果
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    Verified { energy: Energy, steps: usize },
    InvalidDynamics { step: usize },
    EnergyMismatch { claimed: Energy, calculated: Energy },
}

/// 优化轨迹 (Proof of Will Certificate)
/// 现在它携带真正的代数载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationTrace {
    pub id: String,
    pub timestamp: u64,
    
    /// 初始种子 (S_0)
    pub initial_state: IdealClass,
    
    /// 扰动序列 (u_0, u_1, ..., u_k)
    /// 这些必须是合法的 IdealClass 生成元
    pub perturbations: Vec<IdealClass>,
    
    /// 最终状态 (S_final = S_0 * product(u_i))
    /// 包含它是为了快速校验，防止重放整个链条前的快速失败检查
    pub final_state: IdealClass,
    
    pub claimed_energy: Energy,
}

impl OptimizationTrace {
    pub fn new(initial_state: IdealClass) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            initial_state: initial_state.clone(),
            perturbations: Vec::new(),
            final_state: initial_state, // 初始时 final = initial
            claimed_energy: f64::MAX,
        }
    }

    pub fn record_step(&mut self, perturbation: IdealClass) {
        // 实时更新 final_state，避免最后再算一遍
        self.final_state = self.final_state.compose(&perturbation);
        self.perturbations.push(perturbation);
    }

    pub fn finalize(&mut self, final_energy: Energy) {
        self.claimed_energy = final_energy;
    }
}

/// 验证器 (The Verifier)
/// 实现了 Verification_Protocol.md 中的 "Deterministic Replay"
pub struct TraceVerifier;

impl TraceVerifier {
    pub fn verify<E>(
        trace: &OptimizationTrace, 
        energy_fn: E
    ) -> VerificationResult
    where 
        E: Fn(&IdealClass) -> Energy,
    {
        // 1. Replay Algebraic Path (L0 Check)
        // 验证者从 S_0 开始，严格重放所有群运算
        let mut calculated_state = trace.initial_state.clone();
        
        for (i, u) in trace.perturbations.iter().enumerate() {
            // 在群论中，只要 u 是群元素，compose 必然成功
            // 但如果 u 损坏（不是合法 IdealClass），compose 可能会 panic 或产生错误
            // 这里假设 IdealClass 数据结构本身保证了它是合法的二元二次型
            calculated_state = calculated_state.compose(u);
            
            // 可选：检查每一步是否都在群内（IdealClass 类型系统已保证）
            // if !calculated_state.is_valid() { ... }
        }

        // 检查重放结果是否与 Trace 中声称的 final_state 一致
        if calculated_state != trace.final_state {
            return VerificationResult::InvalidDynamics { step: trace.perturbations.len() };
        }

        // 2. Audit Logic Energy (L1 Check)
        // 使用验证者的评估函数（包含 STP 逻辑检查）计算最终能量
        let calculated_energy = energy_fn(&calculated_state);
        
        let epsilon = 1e-6;
        if (calculated_energy - trace.claimed_energy).abs() > epsilon {
            return VerificationResult::EnergyMismatch { 
                claimed: trace.claimed_energy, 
                calculated: calculated_energy 
            };
        }

        VerificationResult::Verified { 
            energy: calculated_energy, 
            steps: trace.perturbations.len() 
        }
    }
}
