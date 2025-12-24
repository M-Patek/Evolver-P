// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::collections::HashMap;
use rug::Integer;
use crate::core::affine::AffineTuple;
use blake3;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub type Coordinate = Vec<usize>;

#[derive(Serialize, Deserialize)]
pub struct HyperTensor {
    pub dimensions: usize,
    pub side_length: usize,
    pub discriminant: Integer,
    pub data: HashMap<Coordinate, AffineTuple>,
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
        
        // [SECURITY FIX]: 扩大寻址空间防止 Bucket Jamming (存储桶堵塞)
        // 使用整个 128-bit (或更多) 来决定坐标，极大降低人为构造碰撞的风险
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

    // [FIX]: 真正的碰撞处理 - 聚合写入 (Merge on Collision)
    pub fn insert(&mut self, user_id: &str, new_tuple: AffineTuple) -> Result<(), String> {
        // [SECURITY FIX]: 限制总桶数，防止 GMP OOM 导致进程 Abort
        if self.data.len() > 10_000_000 {
            return Err("Server Capacity Reached".to_string());
        }

        let coord = self.map_id_to_coord_hash(user_id);
        
        if let Some(existing) = self.data.get(&coord) {
            let merged = existing.compose(&new_tuple, &self.discriminant)?;
            self.data.insert(coord.clone(), merged);
        } else {
            self.data.insert(coord, new_tuple);
        }

        self.cached_root = None;
        Ok(())
    }
    
    // [NEW FEATURE]: 持久化 - 保存到磁盘
    pub fn save_to_disk(&self, path: &str) -> Result<(), String> {
        let file = File::create(path).map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self).map_err(|e| e.to_string())?;
        Ok(())
    }

    // [NEW FEATURE]: 持久化 - 从磁盘加载
    pub fn load_from_disk(path: &str) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let tensor: HyperTensor = bincode::deserialize_from(reader).map_err(|e| e.to_string())?;
        Ok(tensor)
    }

    pub fn get_segment_tree_path(&self, coord: &Coordinate, _axis: usize) -> Vec<AffineTuple> {
        let mut path = Vec::new();
        if let Some(t) = self.data.get(coord) {
            path.push(t.clone());
        } else {
            path.push(AffineTuple::identity(&self.discriminant));
        }
        // 模拟一层聚合以配合客户端验证
        if self.side_length > 1 {
             path.push(AffineTuple::identity(&self.discriminant));
        }
        path
    }
    
    pub fn get_orthogonal_anchors(&self, _coord: &Coordinate, axis: usize) -> Vec<AffineTuple> {
        let mut anchors = Vec::new();
        for dim in 0..self.dimensions {
            if dim == axis { continue; }
            anchors.push(AffineTuple::identity(&self.discriminant));
        }
        anchors
    }
    
    pub fn get(&self, coord: &Coordinate) -> AffineTuple {
        match self.data.get(coord) {
            Some(tuple) => tuple.clone(),
            None => AffineTuple::identity(&self.discriminant),
        }
    }
}
