// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::neuron::HTPNeuron;
use crate::core::affine::AffineTuple;
use rug::Integer;
use std::sync::Arc;
use std::collections::HashSet;

/// 🔮 HTPOracle (Generation Head)
pub struct HTPOracle {
    neuron: Arc<HTPNeuron>,
}

impl HTPOracle {
    pub fn new(neuron: Arc<HTPNeuron>) -> Self {
        HTPOracle { neuron }
    }

    /// 🔍 Core Generation Logic
    pub fn suggest_candidates(&self) -> Result<HashSet<Integer>, String> {
        let memory_guard = self.neuron.memory.read().map_err(|_| "Lock poisoned")?;
        let weight = &self.neuron.p_weight;

        let mut candidates = HashSet::new();

        // [Direct Access Upgrade]: 遍历所有坐标的所有微观事件
        // 即使发生了哈希碰撞，MicroTimeline 也完美保留了每个独立的事件
        for (_coord, timeline) in memory_guard.data.iter() {
            // 深入时间线内部
            for tuple in timeline.events.values() {
                // [Inverse Logic]: 尝试对每个微观事件进行整除逆向
                if let Some(quotient) = tuple.try_divide_p(weight) {
                    candidates.insert(quotient);
                }
            }
        }

        Ok(candidates)
    }

    /// 🧭 Spatial Query
    pub fn query_spatial_neighbors(&self, active_coords: &[Vec<usize>]) -> Result<Vec<AffineTuple>, String> {
        let memory_guard = self.neuron.memory.read().map_err(|_| "Lock poisoned")?;
        let mut neighbors = Vec::new();

        for coord in active_coords {
            for dim in 0..coord.len() {
                let mut next_coord = coord.clone();
                // +1 Neighbor
                next_coord[dim] = (next_coord[dim] + 1) % memory_guard.side_length;
                
                // [Access Upgrade]: 获取该坐标的坍缩状态 (Collapsed State)
                // 邻居的“意义”应当是其所有历史的总和
                if let Ok(tuple) = memory_guard.get_collapsed_state(&next_coord) {
                    // 过滤掉 Identity (空节点)
                    if tuple.p_factor != Integer::from(1) {
                         neighbors.push(tuple);
                    }
                }
                
                // -1 Neighbor
                let mut prev_coord = coord.clone();
                prev_coord[dim] = if prev_coord[dim] == 0 { memory_guard.side_length - 1 } else { prev_coord[dim] - 1 };
                
                if let Ok(tuple) = memory_guard.get_collapsed_state(&prev_coord) {
                    if tuple.p_factor != Integer::from(1) {
                         neighbors.push(tuple);
                    }
                }
            }
        }
        
        Ok(neighbors)
    }
}
