// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::tensor::HyperTensor;
use crate::core::affine::AffineTuple;
use std::collections::HashMap;

impl HyperTensor {
    pub fn calculate_global_root(&mut self) -> Result<AffineTuple, String> {
        if let Some(ref root) = self.cached_root {
            return Ok(root.clone());
        }

        let root = self.compute_root_internal()?;
        self.cached_root = Some(root.clone());
        Ok(root)
    }

    pub fn compute_root_internal(&self) -> Result<AffineTuple, String> {
        // [PERF FIX]: 移除旧的 build_active_prefixes (内存炸弹)，改为稀疏递归
        let root = self.fold_sparse(0, &self.data)?;
        Ok(root)
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
            layer_agg = layer_agg.compose(&sub_result, &self.discriminant)?;
        }

        Ok(layer_agg)
    }
}
