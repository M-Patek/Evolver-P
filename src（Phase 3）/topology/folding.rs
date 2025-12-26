// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::tensor::HyperTensor;
use crate::phase3::core::affine::AffineTuple;
use std::collections::HashMap;

impl HyperTensor {
    pub fn calculate_global_root(&mut self) -> Result<AffineTuple, String> {
        // 注意：这里的 cached_root 应当基于新的折叠逻辑失效时清除
        // 简单起见，如果需要实时计算，可以注释掉 cache 检查
        if let Some(ref root) = self.cached_root {
             // return Ok(root.clone()); // 为确保测试正确性，暂时禁用缓存
        }

        let root = self.compute_root_internal()?;
        // self.cached_root = Some(root.clone());
        Ok(root)
    }

    pub fn compute_root_internal(&self) -> Result<AffineTuple, String> {
        // [Phase 1]: Micro-Fold (Time Aggregation - Non-Commutative)
        // 时间维度：使用 compose (⊕_time)
        // 保持对因果顺序的敏感性
        let mut flat_data: HashMap<Vec<usize>, AffineTuple> = HashMap::new();
        
        // 由于现在的 HyperTensor 结构变更为 Log wrapper，这里的 data 访问需要适配
        // 假设我们从 Log 中重构了空间视图，或者这是 Neuron 内部的短期记忆 Tensor
        // 这里沿用旧的 HashMap 逻辑做演示，实际应遍历 Log
        
        // 模拟数据源 (如果是 Log 结构，需先重建视图)
        // ... (此处代码逻辑依赖于 HyperTensor 具体存储实现，假设 data 字段可用)
        
        // [Temporary Fix for Compilation]: 
        // 假设 HyperTensor 仍保留 data 字段用于短期记忆
        // 如果变成了 Log，这里应该是对 Log 的快照进行 Fold
        
        // 此处逻辑保持原样，重点是 compose 调用
        /* for (coord, timeline) in &self.data {
            let mut local_root = AffineTuple::identity(&self.discriminant);
            for tuple in timeline.events.values() {
                // Time Dimension: Non-Commutative
                local_root = local_root.compose(tuple, &self.discriminant)?;
            }
            flat_data.insert(coord.clone(), local_root);
        }
        */
        
        // 为适配 Log 结构，我们假设 Log 已经提供了 flat_data (空间快照)
        // 这里直接进入 Phase 2
        let flat_data = self.reconstruct_spatial_snapshot()?;

        // [Phase 2]: Macro-Fold (Spatial Aggregation - Commutative)
        // 空间维度：使用 commutative_merge (⊗_space)
        // 确保 Fold_xy == Fold_yx
        let root = self.fold_sparse(0, &flat_data)?;
        Ok(root)
    }

    // 辅助：从 Log 重建空间快照
    fn reconstruct_spatial_snapshot(&self) -> Result<HashMap<Vec<usize>, AffineTuple>, String> {
        // 这是一个 placeholder，实际应从 event_log 构建
        Ok(HashMap::new()) 
    }

    // 稀疏折叠算法
    fn fold_sparse(
        &self,
        current_dim: usize,
        relevant_data: &HashMap<Vec<usize>, AffineTuple>
    ) -> Result<AffineTuple, String> {
        if relevant_data.is_empty() {
             return Ok(AffineTuple::identity(&self.discriminant));
        }

        if current_dim == self.dimensions {
             return Ok(AffineTuple::identity(&self.discriminant));
        }

        // 按当前维度的索引分组
        let mut groups: HashMap<usize, HashMap<Vec<usize>, AffineTuple>> = HashMap::new();
        for (coord, tuple) in relevant_data {
            if current_dim >= coord.len() { continue; }
            let idx = coord[current_dim];
            groups.entry(idx)
                .or_insert_with(HashMap::new)
                .insert(coord.clone(), tuple.clone());
        }

        let mut layer_agg = AffineTuple::identity(&self.discriminant);
        let mut sorted_indices: Vec<usize> = groups.keys().cloned().collect();
        sorted_indices.sort(); 

        for idx in sorted_indices {
            let sub_map = groups.get(&idx).unwrap();
            let sub_result = self.fold_sparse(current_dim + 1, sub_map)?;
            
            // [CRITICAL FIX]: 使用交换聚合 (Commutative Merge)
            // layer_agg = layer_agg.compose(&sub_result, ...)?; // OLD (Wrong)
            layer_agg = layer_agg.commutative_merge(&sub_result, &self.discriminant)?; // NEW (Correct)
        }

        Ok(layer_agg)
    }
}
