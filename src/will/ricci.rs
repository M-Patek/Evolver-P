use crate::soul::algebra::IdealClass;
use crate::will::perturber::Perturber; // 假设 Perturber 提供了获取邻居的方法
use crate::body::projection::Projector; // 用于在特征空间计算距离
use std::collections::HashSet;

/// Discrete Ricci Flow Engine
/// 
/// 负责计算 Ollivier-Ricci 曲率并管理度量场的演化。
/// 核心思想：通过拉长负曲率（高熵）区域的边，迫使优化器绕道。
pub struct RicciFlow {
    /// 曲率修正强度 (gamma)
    sensitivity: f64,
    /// 模拟的时间步长 (dt)
    flow_step: f64,
    /// 投影仪，用于将代数状态映射到几何空间以计算 Wasserstein 距离
    projector: Projector,
}

impl RicciFlow {
    pub fn new(sensitivity: f64, p: u64) -> Self {
        Self {
            sensitivity,
            flow_step: 0.1, // 默认流速
            projector: Projector::new(p),
        }
    }

    /// 计算两个状态（边）之间的 Ollivier-Ricci 曲率
    /// 
    /// Formula: \kappa(x, y) = 1 - W_1(m_x, m_y) / d(x, y)
    /// 
    /// 注意：在巨大的隐式图中计算精确的 Wasserstein 距离 (W1) 是昂贵的。
    /// 这里我们使用基于特征空间投影的贪婪近似 (Greedy Approximation)。
    pub fn calculate_curvature(
        &self, 
        state_x: &IdealClass, 
        state_y: &IdealClass,
        perturber: &Perturber
    ) -> f64 {
        // 1. 获取 x 和 y 的邻域 (1-hop neighbors)
        // 注意：Perturber 需要支持 peek_neighbors 操作
        let neighbors_x = perturber.peek_neighbors(state_x);
        let neighbors_y = perturber.peek_neighbors(state_y);

        if neighbors_x.is_empty() || neighbors_y.is_empty() {
            return 0.0; // 孤立点，无曲率信息
        }

        // 2. 计算基准距离 d(x, y)
        // 在 Cayley 图中，相邻节点的图距离通常为 1，但在特征流形上并非如此。
        // 我们使用特征空间的欧氏距离作为度量。
        let feat_x = self.projector.project_continuous(state_x);
        let feat_y = self.projector.project_continuous(state_y);
        let base_dist = self.euclidean_dist(&feat_x, &feat_y).max(1e-6);

        // 3. 近似计算 W1 距离 (Earth Mover's Distance)
        // 简化逻辑：计算邻居集合在特征空间重心的移动距离 + 散度差异
        // 这是一个快速的启发式替代方案，比求解线性规划快得多。
        let w1_dist = self.approximate_w1(&neighbors_x, &neighbors_y);

        // 4. 计算曲率
        let kappa = 1.0 - (w1_dist / base_dist);

        // 钳制曲率范围，防止数值不稳定
        kappa.max(-2.0).min(1.0)
    }

    /// 计算“有效梯度修正项”
    /// 
    /// \nabla_{eff} = \nabla E + \gamma * exp(-\kappa)
    /// 
    /// 如果曲率 \kappa 是负的（陷阱），exp(-\kappa) 会很大，产生巨大的排斥势能。
    pub fn compute_penalty(&self, kappa: f64) -> f64 {
        if kappa >= 0.0 {
            // 正曲率区域（结构稳固），给予少量奖励或无惩罚
            return -0.1 * kappa.abs(); 
        } else {
            // 负曲率区域（熵增陷阱），给予指数级惩罚
            // 物理意义：将有效距离拉长
            return self.sensitivity * (-kappa).exp();
        }
    }

    // --- Helper Methods ---

    fn approximate_w1(&self, set_a: &[IdealClass], set_b: &[IdealClass]) -> f64 {
        // 计算两个点集的质心距离作为 W1 的下界代理
        let centroid_a = self.centroid(set_a);
        let centroid_b = self.centroid(set_b);
        self.euclidean_dist(&centroid_a, &centroid_b)
    }

    fn centroid(&self, states: &[IdealClass]) -> Vec<f64> {
        if states.is_empty() { return vec![0.0; 8]; } // 假设 8维特征
        
        let mut sum = vec![0.0; 8]; 
        let count = states.len() as f64;

        for s in states {
            let feat = self.projector.project_continuous(s);
            // 简单的维度对齐，实际应根据 Projector 维度动态调整
            for (i, v) in feat.iter().enumerate().take(8) {
                sum[i] += v;
            }
        }

        sum.iter().map(|v| v / count).collect()
    }

    fn euclidean_dist(&self, a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}
