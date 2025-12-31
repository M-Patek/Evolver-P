use num_bigint::BigInt;
use num_traits::{ToPrimitive, Signed, Zero};
use crate::soul::algebra::ClassGroupElement;

/// 线性同余投影 (Linear Congruence Projection)
///
/// 取代了原先基于 FNV 哈希的伪随机投影。
/// 这是一个严格的 Artin-like 映射，具备 Lipschitz 连续性。
///
/// # 数学定义
/// \Psi_k(S) = (a + k * b) \pmod{P}
///
/// # 性质
/// 1. **结构保持性 (Structure-Preserving)**: 
///    代数状态 (a, b) 的微小变化（如 VAPO 的 Fine-tuning）会线性地传递到输出。
///    这使得能量景观呈现出梯度特征，而非白噪声。
/// 2. **深度敏感性 (Depth-Aware)**:
///    参数 `depth` (k) 作为线性系数，确保了不同层级的投影平面是“旋转”的，
///    从而避免了不同层级的逻辑退化为简单的重复。
///
/// # 参数
/// * `g` - 理想类群元素 (State)
/// * `p` - 投影模数 (Prime Base)
/// * `depth` - 当前决策树的深度 k
pub fn project_to_digit(g: &ClassGroupElement, p: u64, depth: u64) -> u64 {
    // 将 u64 参数转换为 BigInt 以便运算
    let p_bi = BigInt::from(p);
    let k_bi = BigInt::from(depth);

    // 核心公式: val = a + k * b
    // 这里的线性组合提取了二次型的几何特征
    let term_b = &k_bi * &g.b;
    let linear_comb = &g.a + term_b;

    // 取模 P
    let mut rem = linear_comb % &p_bi;

    // 修正负数余数 (Rust 的 % 运算符保留符号)
    // 我们需要结果在 [0, p-1] 区间内
    if rem < BigInt::zero() {
        rem += &p_bi;
    }

    // 安全转换为 u64 (结果必然在 [0, p-1] 范围内)
    rem.to_u64().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    // Mock object helper
    fn make_element(a: i64, b: i64, c: i64) -> ClassGroupElement {
        ClassGroupElement {
            a: BigInt::from(a),
            b: BigInt::from(b),
            c: BigInt::from(c),
        }
    }

    #[test]
    fn test_linearity() {
        // 验证 Lipschitz 连续性：输入微变，输出微变
        let p = 100u64;
        let depth = 1u64;
        
        let s1 = make_element(10, 5, 1);     // val = 10 + 1*5 = 15
        let s2 = make_element(11, 5, 1);     // val = 11 + 1*5 = 16 (变化 1)
        
        let d1 = project_to_digit(&s1, p, depth);
        let d2 = project_to_digit(&s2, p, depth);
        
        assert_eq!(d2 as i64 - d1 as i64, 1);
    }

    #[test]
    fn test_depth_rotation() {
        // 验证深度参数能否改变投影视角
        let p = 100u64;
        let s = make_element(10, 5, 1);
        
        // k=0: 10 + 0 = 10
        let d_k0 = project_to_digit(&s, p, 0);
        assert_eq!(d_k0, 10);

        // k=1: 10 + 5 = 15
        let d_k1 = project_to_digit(&s, p, 1);
        assert_eq!(d_k1, 15);
    }
}
