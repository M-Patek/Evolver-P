// src/body/topology.rs

/// v-PuNN (Valuation-Adaptive Perturbation Neural Network) 的拓扑配置
/// 定义了决策树的深度和用于投影的基数
#[derive(Debug, Clone, Copy)]
pub struct VPuNNConfig {
    /// 决策树/递归层级的深度
    pub depth: usize,
    /// 用于代数投影的素数基底 (Prime Base)
    pub p_base: u64,
}

impl VPuNNConfig {
    /// 创建一个新的 v-PuNN 配置
    pub fn new(depth: usize, p_base: u64) -> Self {
        Self { depth, p_base }
    }

    /// WordNet 数据集的推荐配置
    /// depth = 19 (对应 WordNet 的层级深度)
    /// p_base = 409 (选定的素数基底)
    pub fn wordnet_default() -> Self {
        Self {
            depth: 19,
            p_base: 409,
        }
    }
}
