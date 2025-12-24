// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::neuron::HTPNeuron;
use crate::core::affine::AffineTuple;
use rug::Integer;
use std::sync::Arc;
use std::collections::HashSet;

/// 🔮 HTPOracle (Generation Head): 代数预言机 / 生成头
/// 在 Crystal Brain 架构中，它的职责是从 HTPNeuron 的高维记忆张量中
/// 逆向“解码”出合法的 Token 候选集，实现自主生成。
pub struct HTPOracle {
    /// 绑定的宿主神经元（提供内存和权重）
    neuron: Arc<HTPNeuron>,
}

impl HTPOracle {
    pub fn new(neuron: Arc<HTPNeuron>) -> Self {
        HTPOracle { neuron }
    }

    /// 🔍 Core Generation Logic: 候选集提取 (Decoding)
    /// 返回一个包含所有在当前代数结构中“活跃”且“合法”的原始素数集合。
    /// 这是 Crystal Brain 生成下一个 Token 的核心步骤。
    pub fn suggest_candidates(&self) -> Result<HashSet<Integer>, String> {
        let memory_guard = self.neuron.memory.read().map_err(|_| "Lock poisoned")?;
        let weight = &self.neuron.p_weight;

        let mut candidates = HashSet::new();

        // [Direct Access]: 直接遍历稀疏张量的活跃节点
        // 相比于遍历整个词表 (Vocab Size)，这里只需要遍历活跃记忆单元 (Active Memory)。
        for (_coord, tuple) in memory_guard.data.iter() {
            // [Inverse Logic]: 逆向还原
            // 已知: P_stored = P_token * P_weight
            // 求解: P_token = P_stored / P_weight
            // 使用 AffineTuple 新增的辅助方法进行整除测试
            if let Some(quotient) = tuple.try_divide_p(weight) {
                // 找到了！quotient 就是原始的 Token Prime
                candidates.insert(quotient);
            }
        }

        // 返回候选集。
        // 下一步只需将这些 Prime 映射回 Token ID 即可完成“生成”。
        Ok(candidates)
    }

    /// 🧭 Spatial Query: 空间邻近查询 (Contextual Associativity)
    /// 查询“当前关注点”附近的坐标，用于联想生成。
    pub fn query_spatial_neighbors(&self, active_coords: &[Vec<usize>]) -> Result<Vec<AffineTuple>, String> {
        let memory_guard = self.neuron.memory.read().map_err(|_| "Lock poisoned")?;
        let mut neighbors = Vec::new();

        for coord in active_coords {
            // 简单的“曼哈顿距离”邻居搜索
            // 尝试在每个维度 +/- 1
            for dim in 0..coord.len() {
                let mut next_coord = coord.clone();
                // +1 Neighbor
                next_coord[dim] = (next_coord[dim] + 1) % memory_guard.side_length;
                if let Some(tuple) = memory_guard.data.get(&next_coord) {
                    neighbors.push(tuple.clone());
                }
                
                // -1 Neighbor
                let mut prev_coord = coord.clone();
                prev_coord[dim] = if prev_coord[dim] == 0 { memory_guard.side_length - 1 } else { prev_coord[dim] - 1 };
                if let Some(tuple) = memory_guard.data.get(&prev_coord) {
                    neighbors.push(tuple.clone());
                }
            }
        }
        
        Ok(neighbors)
    }
}
