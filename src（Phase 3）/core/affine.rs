// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::algebra::ClassGroupElement;
use rug::Integer;

/// ⚠️ [Safety Limit]: 局部算子 P 因子最大位宽
/// 限制为 8192 bits。这足以聚合 ~128 个 Token (假设每个 Token 64 bits)，
/// 但严禁用于全局历史累积。这从根本上杜绝了 P 因子爆炸问题。
const MAX_CHUNK_P_BITS: u32 = 8192;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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

    /// 🧩 Local Chunk Composition (局部聚合)
    /// 
    /// 注意：此方法仅用于将相邻的几个 Token 聚合成一个更大的算子 (Chunk Operator)。
    /// 严禁用于全局状态的串行累积！全局演化请使用 `ClassGroupElement::apply_affine`。
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // [SAFETY CHECK]: 防止 P 因子爆炸
        // 在 Phase 3 架构中，全局 P 累积是被数学禁止的。
        let p_bits_new = self.p_factor.significant_bits() + other.p_factor.significant_bits();
        if p_bits_new > MAX_CHUNK_P_BITS { 
             return Err(format!(
                 "⛔ Security Halt: Affine P-Factor overflow ({} bits). \
                 Global accumulation is forbidden. Use `apply_affine` for state evolution.", 
                 p_bits_new
             ));
        }

        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Composition Law: (P1, Q1) + (P2, Q2) = (P1*P2, Q1^P2 * Q2)
        // 注意顺序：先应用 other 的 P2 到 self 的 Q1，再加上 other 的 Q2
        // 这里体现了非交换性：S ^ (P1*P2) * (Q1^P2 * Q2)
        let q1_pow_p2 = self.q_shift.pow(&other.p_factor, discriminant)?;
        let new_q = q1_pow_p2.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    /// 逆向操作辅助函数：用于 Oracle 提取
    pub fn try_divide_p(&self, denominator: &Integer) -> Option<Integer> {
        let (quotient, rem) = self.p_factor.div_rem_ref(denominator).into();
        if rem == Integer::from(0) {
            Some(quotient)
        } else {
            None
        }
    }
}
