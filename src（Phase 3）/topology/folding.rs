// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::tensor::HyperTensor;
use crate::phase3::core::affine::AffineTuple;
use crate::phase3::core::algebra::ClassGroupElement;
use rug::Integer;
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
        // 从 TimeSegmentTree 重建当前的空间快照
        // 这一步将每个 Cell 内部复杂的历史因果链坍缩为唯一的“现在”状态
        let flat_data = self.reconstruct_spatial_snapshot()?;

        // [Phase 2]: Macro-Fold (Spatial Aggregation - Commutative)
        // 空间维度：使用 commutative_merge (⊗_space)
        // 确保 Fold_xy == Fold_yx，实现多维正交验证的数学闭环
        let root = self.fold_sparse(0, &flat_data)?;
        Ok(root)
    }

    /// 🛠️ [FIXED]: 从时间线重建空间快照
    /// 连接 TimeSegmentTree (Micro) -> Spatial Fold (Macro)
    /// 填补了之前返回空 Map 的逻辑缺口，使 HyperTensor 真正具备了状态感知能力。
    fn reconstruct_spatial_snapshot(&self) -> Result<HashMap<Vec<usize>, AffineTuple>, String> {
        let mut snapshot = HashMap::new();
        let one = Integer::from(1);
        let identity_q = ClassGroupElement::identity(&self.discriminant);

        // 1. 遍历所有活跃的存储单元 (Cells)
        // self.data 是 HashMap<Coordinate, TimeSegmentTree>
        for (coord, time_tree) in &self.data {
            
            // 2. [Time Collapse]: 计算时间根
            // 调用 TimeSegmentTree::root()，这会执行非交换的时间聚合 (compose)
            // 这一步体现了因果律：历史顺序不同，生成的 root 也不同
            let cell_time_root = time_tree.root(&self.discriminant)?;

            // 3. [Sparse Optimization]: 稀疏性过滤
            // 只有非单位元的状态才值得参与昂贵的空间折叠。
            // 只要 P > 1，说明该节点包含有效信息（Accumulated Weight）。
            if cell_time_root.p_factor != one {
                snapshot.insert(coord.clone(), cell_time_root);
            } else {
                // ⚠️ 边缘情况检查：
                // 如果 P=1 但 Q 不是单位元（纯位移/噪声注入），也应该保留。
                // 这种情况可能发生在 "Zero Weight" 的纯噪声注入步骤。
                if cell_time_root.q_shift != identity_q {
                     snapshot.insert(coord.clone(), cell_time_root);
                }
            }
        }

        // 4. 返回快照，供 fold_sparse 使用
        Ok(snapshot)
    }

    // 内存友好的稀疏折叠算法 (O(N) 内存占用)
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

        // 按当前维度的索引分组 O(N)
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
            // 确保 fold 顺序不影响最终结果
            layer_agg = layer_agg.commutative_merge(&sub_result, &self.discriminant)?;
        }

        Ok(layer_agg)
    }
}
