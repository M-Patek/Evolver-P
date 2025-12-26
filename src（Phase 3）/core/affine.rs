// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::algebra::ClassGroupElement;
use rug::Integer;

/// ⚠️ [Safety Limit]: 局部算子 P 因子最大位宽
/// 边界定义 1: 仿射因子溢出 (P-Factor Overflow)
/// 证伪意义: 防止算子无限膨胀，阻断 CPU DoS 攻击。
/// 在 Phase 3 中，我们允许 8192 bits，这足以容纳一个微观时间片 (Chunk) 的历史，
/// 但绝不允许容纳无限历史。
const MAX_CHUNK_P_BITS: u32 = 8192;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AffineTuple {
    pub p_factor: Integer,      
    pub q_shift: ClassGroupElement, 
}

impl AffineTuple {
    pub fn identity(discriminant: &Integer) -> Self {
        AffineTuple {
            p_factor: Integer::from(1),
            q_shift: ClassGroupElement::identity(discriminant),
        }
    }

    /// ⏳ [Time Operator]: Non-Commutative Composition (时间演化 - 非交换)
    /// 公式: (P1, Q1) ⊕ (P2, Q2) = (P1*P2, Q1^P2 * Q2)
    /// 这里的“非交换性”体现了时间的因果律：先发生的事件会作为指数影响后发生的事件。
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // [FALSIFIABILITY CHECK 1]: P-Factor Overflow
        // 如果算子规模超过安全阈值，视为非法操作，立即熔断。
        // 这强迫上层逻辑必须使用 Streaming 模式处理长序列，而不是无限 Accumulate。
        let p_bits_new = self.p_factor.significant_bits() + other.p_factor.significant_bits();
        if p_bits_new > MAX_CHUNK_P_BITS { 
             return Err(format!("❌ Falsified: Affine P-Factor overflow ({} bits > {}). Global accumulation is forbidden; use streaming.", p_bits_new, MAX_CHUNK_P_BITS));
        }

        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Composition Law: Q_new = Q1^P2 * Q2
        // Q1^P2 使得 Q1 的语义被 P2 "扭曲" (Time-Warped)，从而绑定了发生顺序。
        let q1_pow_p2 = self.q_shift.pow(&other.p_factor, discriminant)?;
        let new_q = q1_pow_p2.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    /// 🌌 [Space Operator]: Commutative Aggregation (空间聚合 - 交换)
    /// 公式: (P1, Q1) ⊗ (P2, Q2) = (P1*P2, Q1*Q2)
    /// 用于在不同维度间合并信息，必须满足交换律以支持全息投影。
    pub fn commutative_merge(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // P_new = P1 * P2 (整数乘法，交换)
        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Q_new = Q1 * Q2 (群乘法，交换)
        // 注意：这里使用的是 compose 而非 pow，确保操作是阿贝尔的 (Abelian)。
        let new_q = self.q_shift.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }
}
