// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::neuron::HTPNeuron;
use crate::core::affine::AffineTuple;
use rug::{Integer, Complete};
use std::sync::Arc;
use std::collections::HashSet;

/// 🔮 HTPOracle: 代数预言机
/// 它的职责是利用张量的拓扑结构，直接“预知”合法的候选集，
/// 从而避免暴力的词表遍历。
pub struct HTPOracle {
    /// 绑定的宿主神经元（提供内存和权重）
    neuron: Arc<HTPNeuron>,
}

impl HTPOracle {
    pub fn new(neuron: Arc<HTPNeuron>) -> Self {
        HTPOracle { neuron }
    }

    /// 🔍 Core Function: 快速提取“合法邻居” (Candidate Extraction)
    /// 返回一个包含所有在当前代数结构中“活跃”的原始素数集合。
    /// O(Context_Len) 而非 O(Vocab_Size)
    pub fn suggest_candidates(&self) -> Result<HashSet<Integer>, String> {
        let memory_guard = self.neuron.memory.read().map_err(|_| "Lock poisoned")?;
        let weight = &self.neuron.p_weight;

        let mut candidates = HashSet::new();

        // [Direct Access]: 直接遍历稀疏张量的活跃节点
        // 这里我们利用了 HyperTensor 的 "Sparse" 特性。
        // 相比于遍历 50,000 个 Token，这里只需要遍历几千个活跃记忆单元。
        for (_coord, tuple) in memory_guard.data.iter() {
            // [Inverse Logic]: 逆向还原
            // 已知: P_stored = P_token * P_weight
            // 求解: P_token = P_stored / P_weight
            // 
            // 只有当 P_stored 能被 P_weight 整除时，这才是我们存进去的有效数据
            // (防止噪声干扰)
            let (quotient, rem) = tuple.p_factor.div_rem_ref(weight).into();

            if rem == Integer::from(0) {
                // 找到了！quotient 就是原始的 Token Prime
                candidates.insert(quotient);
            } else {
                // 如果不能整除，说明这个节点可能被聚合了或者是噪声，
                // 或者是其他神经元留下的痕迹。
                // 在更复杂的实现中，我们可能需要递归分解 (Recursive Factorization)。
            }
        }

        // 返回候选集。
        // 这个集合里的素数，都是在当前上下文中“有身份”的，
        // 也就是数学上“自洽”的候选者。
        Ok(candidates)
    }

    /// 🧭 Spatial Query: 空间邻近查询 (高级功能)
    /// 如果我们假设坐标 (Coordinate) 蕴含了语义（如 Phase 2 所述），
    /// 我们还可以查询“当前关注点”附近的坐标。
    pub fn query_spatial_neighbors(&self, active_coords: &[Vec<usize>]) -> Result<Vec<AffineTuple>, String> {
        let memory_guard = self.neuron.memory.read().map_err(|_| "Lock poisoned")?;
        let mut neighbors = Vec::new();

        for coord in active_coords {
            // 简单的“曼哈顿距离”邻居搜索 demo
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
