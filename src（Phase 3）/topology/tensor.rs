// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::collections::HashMap;
use rug::Integer;
use crate::phase3::core::affine::AffineTuple;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use rand::seq::SliceRandom; // 用于维度打乱测试
use rand::thread_rng;

pub type Coordinate = Vec<usize>;

/// 🌳 TimeSegmentTree: 微观历史树
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimeSegmentTree {
    pub leaves: Vec<AffineTuple>,
}

impl TimeSegmentTree {
    pub fn new() -> Self {
        TimeSegmentTree { leaves: Vec::new() }
    }

    pub fn append(&mut self, tuple: AffineTuple) {
        self.leaves.push(tuple);
    }

    pub fn root(&self, discriminant: &Integer) -> Result<AffineTuple, String> {
        if self.leaves.is_empty() {
            return Ok(AffineTuple::identity(discriminant));
        }
        self.build_tree_recursive(&self.leaves, discriminant)
    }

    fn build_tree_recursive(&self, nodes: &[AffineTuple], discriminant: &Integer) -> Result<AffineTuple, String> {
        if nodes.len() == 0 {
            return Ok(AffineTuple::identity(discriminant));
        }
        if nodes.len() == 1 {
            return Ok(nodes[0].clone());
        }

        let mid = nodes.len() / 2;
        let left = self.build_tree_recursive(&nodes[0..mid], discriminant)?;
        let right = self.build_tree_recursive(&nodes[mid..], discriminant)?;

        // [Non-Commutative]: Left ⊕ Right
        left.compose(&right, discriminant)
    }

    pub fn generate_witness(&self, index: usize, discriminant: &Integer) -> Result<Vec<(AffineTuple, bool)>, String> {
        if index >= self.leaves.len() {
            return Err("Index out of bounds".to_string());
        }
        let mut witness = Vec::new();
        self.generate_witness_recursive(&self.leaves, index, 0, discriminant, &mut witness)?;
        Ok(witness)
    }

    fn generate_witness_recursive(
        &self, 
        nodes: &[AffineTuple], 
        target_abs_index: usize, 
        current_offset: usize,
        discriminant: &Integer,
        witness: &mut Vec<(AffineTuple, bool)>
    ) -> Result<AffineTuple, String> {
        if nodes.len() == 1 {
            return Ok(nodes[0].clone());
        }

        let mid = nodes.len() / 2;
        let left_slice = &nodes[0..mid];
        let right_slice = &nodes[mid..];

        if target_abs_index < current_offset + mid {
            let right_agg = self.build_tree_recursive(right_slice, discriminant)?;
            witness.push((right_agg, false)); 
            let left_agg = self.generate_witness_recursive(left_slice, target_abs_index, current_offset, discriminant, witness)?;
            return left_agg.compose(&self.build_tree_recursive(right_slice, discriminant)?, discriminant);
        } else {
            let left_agg = self.build_tree_recursive(left_slice, discriminant)?;
            witness.push((left_agg, true));
            let right_agg = self.generate_witness_recursive(right_slice, target_abs_index, current_offset + mid, discriminant, witness)?;
            return left_agg.compose(&right_agg, discriminant);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HyperTensor {
    pub dimensions: usize,
    pub side_length: usize,
    pub discriminant: Integer,
    
    // [FIX]: Value 从单一的 AffineTuple 升级为 TimeSegmentTree
    pub data: HashMap<Coordinate, TimeSegmentTree>,
    
    #[serde(skip)]
    pub cached_root: Option<AffineTuple>, 
}

impl HyperTensor {
    pub fn new(dim: usize, len: usize, discriminant: Integer) -> Self {
        HyperTensor {
            dimensions: dim,
            side_length: len,
            discriminant,
            data: HashMap::new(),
            cached_root: None,
        }
    }

    pub fn map_id_to_coord(&self, numeric_id: u64) -> Coordinate {
        let mut coord = Vec::with_capacity(self.dimensions);
        let mut temp = numeric_id;
        let l = self.side_length as u64;
        for _ in 0..self.dimensions {
            coord.push((temp % l) as usize);
            temp /= l;
        }
        coord
    }
    
    pub fn map_id_to_coord_hash(&self, user_id: &str) -> Coordinate {
        let mut hasher = blake3::Hasher::new();
        hasher.update(user_id.as_bytes());
        hasher.update(b":htp:coord:v2");
        let hash_output = hasher.finalize();
        
        let mut coord = Vec::with_capacity(self.dimensions);
        let reader = hash_output.as_bytes();
        let l = self.side_length as u128;
        
        let mut val = u128::from_le_bytes(reader[0..16].try_into().unwrap());
        
        for _ in 0..self.dimensions {
            coord.push((val % l) as usize);
            val /= l;
        }
        coord
    }

    pub fn insert(&mut self, user_id: &str, new_tuple: AffineTuple) -> Result<(), String> {
        let coord = self.map_id_to_coord_hash(user_id);
        
        self.data.entry(coord)
            .or_insert_with(TimeSegmentTree::new)
            .append(new_tuple);

        self.cached_root = None;
        Ok(())
    }
    
    // ... [save_to_disk / load_from_disk Omitted for brevity, assume unchanged] ...

    pub fn get_segment_tree_path(&self, coord: &Coordinate, _axis: usize) -> Vec<AffineTuple> {
        if let Some(tree) = self.data.get(coord) {
            if let Ok(root) = tree.root(&self.discriminant) {
                return vec![root];
            }
        }
        vec![AffineTuple::identity(&self.discriminant)]
    }
    
    /// 🛡️ [The Commutativity Limit Check]: 全息对称性验证
    /// 
    /// 这是 Evolver 的“判死刑”逻辑：
    /// 如果 Fold(Axis_A -> Axis_B) != Fold(Axis_B -> Axis_A)，
    /// 意味着空间算子混入了因果性（时间毒素），必须立即 Panic。
    pub fn verify_holographic_symmetry(&self) -> Result<bool, String> {
        // 1. Path A: 自然序 (Canonical Order)
        let order_a: Vec<usize> = (0..self.dimensions).collect();
        let root_a = self.compute_root_internal(&order_a)?;

        // 2. Path B: 置换序 (Permuted Order)
        let mut order_b = order_a.clone();
        if self.dimensions >= 2 {
            // 交换前两个维度做最严格的测试
            order_b.swap(0, 1); 
        } else {
            return Ok(true);
        }

        let root_b = self.compute_root_internal(&order_b)?;

        // 3. The Judgment (最终审判)
        // 比较 P 因子和 Q 移位是否完全一致
        let p_match = root_a.p_factor == root_b.p_factor;
        let q_match = root_a.q_shift == root_b.q_shift;

        if !p_match || !q_match {
            // [FALSIFIED]: 证伪成功，系统存在严重逻辑漏洞
            eprintln!("❌ HOLOGRAPHIC VIOLATION DETECTED!");
            eprintln!("   Order A {:?} -> Root: {:?}", order_a, root_a);
            eprintln!("   Order B {:?} -> Root: {:?}", order_b, root_b);
            return Ok(false);
        }

        // [VERIFIED]: 全息一致性通过
        Ok(true)
    }
}
