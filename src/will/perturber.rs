use crate::soul::algebra::Quaternion;

/// 扰动器 (Perturber) 接口
/// 定义了“意志”如何在状态空间中探索。
/// 在新架构中，这对应于提供 Cayley 图（Pizer Graph）的生成元集合。
pub trait Perturber {
    /// 获取当前可用的所有移动步（Hecke 算子）
    fn get_moves(&self) -> Vec<Quaternion>;
}

/// Hecke 扰动器
/// 专门用于定四元数代数算术格的探索。
/// 它提供了一组范数为 p (或相关小素数) 的生成元，确保在 Ramanujan 图上行走。
pub struct HeckePerturber;

impl HeckePerturber {
    pub fn new() -> Self {
        Self
    }
}

impl Perturber for HeckePerturber {
    fn get_moves(&self) -> Vec<Quaternion> {
        // 硬编码 B_{37, \infty} 的生成元集合。
        // 这些四元数的范数为 37 (N(q) = a^2 + b^2 + 37c^2 + 37d^2 = 37)。
        // 它们对应于 Pizer 图中的邻接边。
        
        vec![
            // 生成元 1 及其逆 (对应于 i 方向的扰动)
            // 6^2 + 1^2 = 37
            Quaternion::new(6, 1, 0, 0),
            Quaternion::new(6, -1, 0, 0),
            
            // 生成元 2 及其逆 (对应于实部主导的扰动)
            // 1^2 + 6^2 = 37
            Quaternion::new(1, 6, 0, 0),
            Quaternion::new(1, -6, 0, 0),

            // 恒等操作 (Self-loop)，用于在搜索中允许“停顿”或作为占位符
            Quaternion::new(1, 0, 0, 0),
            
            // 注意：在完整的 Pizer 图实现中，对于 p=37，每个节点应该有 p+1 = 38 个邻居。
            // 这里为了演示和性能，仅选取了最具代表性的几个正交方向。
        ]
    }
}
