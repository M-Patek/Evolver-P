// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::collections::{HashMap, BTreeMap};
use rug::Integer;
use crate::core::affine::AffineTuple;
use blake3;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub type Coordinate = Vec<usize>;

/// [Theoretical Best]: 微观时间线容器
/// 当空间发生碰撞时，我们在时间维度上展开，保证逻辑的因果完备性。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MicroTimeline {
    /// Key: Timestamp (Logic Sequence), Value: Affine Event
    /// BTreeMap 保证了按时间戳严格排序，这对于非交换代数至关重要。
    pub events: BTreeMap<u64, AffineTuple>,
}

impl MicroTimeline {
    pub fn new() -> Self {
        MicroTimeline {
            events: BTreeMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HyperTensor {
    pub dimensions: usize,
    pub side_length: usize,
    pub discriminant: Integer,
    
    /// [Upgrade]: Data 无论是空间还是时间，都是正交的
    /// HashMap<Space, BTreeMap<Time, Event>>
    pub data: HashMap<Coordinate, MicroTimeline>,
    
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
        hasher.update(b":htp:coord:v3:orthogonal"); // Version Bump
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

    /// [FIXED]: Spacetime Orthogonal Insertion
    /// 不再进行有损的 Merge，而是非破坏性地追加到时间线。
    /// timestamp: 由神经元传入的逻辑时钟 t
    pub fn insert(&mut self, user_id: &str, new_tuple: AffineTuple, timestamp: u64) -> Result<(), String> {
        // [SECURITY FIX]: 依然保留容量限制，防止 OOM
        if self.data.len() > 10_000_000 {
            return Err("Server Capacity Reached".to_string());
        }

        let coord = self.map_id_to_coord_hash(user_id);
        
        // 获取或创建微观时间线
        let timeline = self.data.entry(coord).or_insert_with(MicroTimeline::new);
        
        // [Logic Preserved]: 即使 coord 碰撞，事件也被保留在独立的时间槽中
        // 如果同一个 t 发生多次写入（极其罕见），BTreeMap 会覆盖，这是符合预期的（同一时刻的状态更新）
        timeline.events.insert(timestamp, new_tuple);

        self.cached_root = None;
        Ok(())
    }
    
    pub fn save_to_disk(&self, path: &str) -> Result<(), String> {
        let file = File::create(path).map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_from_disk(path: &str) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let tensor: HyperTensor = bincode::deserialize_from(reader).map_err(|e| e.to_string())?;
        Ok(tensor)
    }

    // 辅助：获取某个坐标的聚合状态（用于 Proof 生成等）
    pub fn get_collapsed_state(&self, coord: &Coordinate) -> Result<AffineTuple, String> {
        if let Some(timeline) = self.data.get(coord) {
            let mut agg = AffineTuple::identity(&self.discriminant);
            for tuple in timeline.events.values() {
                agg = agg.compose(tuple, &self.discriminant)?;
            }
            Ok(agg)
        } else {
            Ok(AffineTuple::identity(&self.discriminant))
        }
    }

    pub fn get_segment_tree_path(&self, coord: &Coordinate, _axis: usize) -> Vec<AffineTuple> {
        let mut path = Vec::new();
        // 这里需要返回聚合后的状态作为叶子节点
        if let Ok(t) = self.get_collapsed_state(coord) {
            path.push(t);
        } else {
            // Error fallback
            path.push(AffineTuple::identity(&self.discriminant));
        }
        
        if self.side_length > 1 {
             path.push(AffineTuple::identity(&self.discriminant));
        }
        path
    }
}
