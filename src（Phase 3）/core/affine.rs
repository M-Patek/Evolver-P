// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::algebra::ClassGroupElement;
use rug::Integer;

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

    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // [SECURITY FIX]: 限制 P-factor 大小为 4096 bits (常规 RSA 级别)
        // 防止 CPU DoS 和 存储桶堵塞攻击的先决条件
        let p_bits = self.p_factor.significant_bits() + other.p_factor.significant_bits();
        if p_bits > 4096 { 
             return Err(format!("❌ Security Halt: Affine P-Factor size ({} bits) exceeds safety limit (4096).", p_bits));
        }

        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Propagate math errors
        let q1_pow_p2 = self.q_shift.pow(&other.p_factor, discriminant)?;
        let new_q = q1_pow_p2.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    // [NEW FEATURE for Crystal Brain]: 逆向操作辅助函数
    // 用于 Oracle 从聚合状态中尝试分离出潜在的 Token Prime
    // 如果 self.p_factor 能被 denominator 整除，返回商 (Quotient)，否则返回 None
    pub fn try_divide_p(&self, denominator: &Integer) -> Option<Integer> {
        let (quotient, rem) = self.p_factor.div_rem_ref(denominator).into();
        if rem == Integer::from(0) {
            Some(quotient)
        } else {
            None
        }
    }
}
