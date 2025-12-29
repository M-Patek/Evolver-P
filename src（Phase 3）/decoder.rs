// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::core::affine::AffineTuple;
use crate::phase3::core::primes::hash_to_prime;
use crate::phase3::topology::tensor::Coordinate; 
use rug::Integer;
use std::collections::{HashMap, HashSet};

/// [Optimization]: K-D Tree Node
/// 用于加速高维空间最近邻搜索的数据结构
#[derive(Debug)]
pub struct KdNode {
    pub point: Coordinate,
    pub left: Option<Box<KdNode>>,
    pub right: Option<Box<KdNode>>,
    pub axis: usize,
}

/// 🗺️ VocabularyTensor: 静态词汇宇宙 (The Atlas)
/// 存储了 Token 在超空间中的确切位置。
pub struct VocabularyTensor {
    /// 正向映射: Coordinate -> Token Prime
    pub star_map: HashMap<Coordinate, Integer>,
    /// 反向映射: Token Prime -> Token ID (用于最终解码)
    pub prime_to_id: HashMap<Integer, u32>,
    
    /// K-D Tree Root for O(log N) search
    pub kd_tree: Option<Box<KdNode>>,
    
    pub dimensions: usize,
    pub side_length: usize,
}

impl VocabularyTensor {
    pub fn new(vocab_size: u32, dimensions: usize, side_length: usize) -> Self {
        let mut star_map = HashMap::new();
        let mut prime_to_id = HashMap::new();
        let mut points_for_tree = Vec::new();
        
        let mut occupied_primes: HashSet<Integer> = HashSet::new();
        let l = side_length as u64;
        
        // 初始化宇宙：将所有 Token 映射到空间中
        for tid in 0..vocab_size {
            let mut coord = Vec::with_capacity(dimensions);
            let mut temp = tid as u64;
            for _ in 0..dimensions {
                coord.push((temp % l) as usize);
                temp /= l;
            }

            // [DCAP Algorithm]: 生成绝对唯一的 Token Prime
            let base_token_str = format!("tok_{}", tid);
            let p = Self::generate_unique_prime(&base_token_str, &occupied_primes);
            
            occupied_primes.insert(p.clone());
            star_map.insert(coord.clone(), p.clone());
            prime_to_id.insert(p, tid);
            points_for_tree.push(coord);
        }

        // 构建 K-D Tree
        let kd_tree = Self::build_kdtree(&mut points_for_tree, 0, dimensions);

        VocabularyTensor {
            star_map,
            prime_to_id,
            kd_tree,
            dimensions,
            side_length,
        }
    }


    /// 🔁 Deterministic Reverse Mapping: Token ID -> Coordinate
    /// 与 `new()` 中的初始化逻辑保持一致：用 base-`side_length` 展开得到坐标。
    /// 注意：index 0 是最低位 digit (LSD)。
    pub fn map_id_to_coord(&self, tid: u64) -> Coordinate {
        let mut coord = Vec::with_capacity(self.dimensions);
        let mut temp = tid;
        let l = self.side_length as u64;
        for _ in 0..self.dimensions {
            coord.push((temp % l) as usize);
            temp /= l;
        }
        coord
    }



    /// 🛡️ [FALSIFIABILITY BOUNDARY B2]: Vocabulary Space Exhausted
    /// 确保语义指纹的绝对唯一性。
    fn generate_unique_prime(base_str: &str, occupied: &HashSet<Integer>) -> Integer {
        let mut nonce = 0u64;
        const MAX_COLLISION_RETRIES: u64 = 1_000_000;

        while nonce < MAX_COLLISION_RETRIES {
            let input_str = if nonce == 0 {
                base_str.to_string()
            } else {
                format!("{}#collision_fix_{}", base_str, nonce)
            };

            if let Ok(candidate) = hash_to_prime(&input_str, 64) {
                if !occupied.contains(&candidate) {
                    return candidate;
                }
            }
            nonce += 1;
        }
        
        panic!("❌ Fatal Error: Vocabulary Space Exhausted. Unable to assign unique prime fingerprint.");
    }

    fn build_kdtree(points: &mut [Coordinate], depth: usize, k: usize) -> Option<Box<KdNode>> {
        if points.is_empty() { return None; }

        let axis = depth % k;
        points.sort_by(|a, b| a[axis].cmp(&b[axis]));
        let mid = points.len() / 2;

        let point = points[mid].clone();
        let (left_slice, right_slice_inclusive) = points.split_at_mut(mid);
        let (_, right_slice) = right_slice_inclusive.split_first_mut().unwrap();

        Some(Box::new(KdNode {
            point,
            left: Self::build_kdtree(left_slice, depth + 1, k),
            right: Self::build_kdtree(right_slice, depth + 1, k),
            axis,
        }))
    }
}

/// 解码结果
pub struct DecodeResult {
    pub token_id: u32,
    pub drift: usize, // 曼哈顿漂移量
}

/// 🧭 InverseDecoder: 坐标导航器 (Phase 4 Upgraded)
/// 集成了 VAPO 所需的超度量观测能力。
pub struct InverseDecoder {
    pub vocab_tensor: VocabularyTensor,
    /// 动态搜索半径：如果直接找不到，允许在多大范围内搜索
    pub search_radius: usize,
}

impl InverseDecoder {
    pub fn new(vocab_size: u32) -> Self {
        InverseDecoder {
            vocab_tensor: VocabularyTensor::new(vocab_size, 4, 32),
            search_radius: 5,
        }
    }

    /// 📍 Decode (Legacy): 仅用于兼容旧逻辑
    pub fn decode(&self, target_root: &AffineTuple) -> Result<DecodeResult, String> {
        self.decode_with_bias(target_root, &vec![0; self.vocab_tensor.dimensions])
    }

    /// 🚀 Decode with Bias (The VAPO Interface)
    /// 将 Bias 纳入观测链，使 fitness 能感知到 Bias 的微调。
    /// 这是解决 "Fatal Coupling" 的关键步骤：让优化器的动作 (Bias Mutation) 在观测端有响应。
    pub fn decode_with_bias(&self, target_root: &AffineTuple, bias: &[usize]) -> Result<DecodeResult, String> {
        // 1. 原始代数投影 (Extract raw algebraic coordinate)
        let mut predicted_coord = self.extract_coordinate(target_root);
        
        // 2. 施加 Bias 校准 (Apply VAPO linear correction)
        self.apply_bias_to_coord(&mut predicted_coord, bias);

        // 3. 完美的零漂移匹配 (Exact Match)
        if let Some(token_prime) = self.vocab_tensor.star_map.get(&predicted_coord) {
             if let Some(&tid) = self.vocab_tensor.prime_to_id.get(token_prime) {
                 return Ok(DecodeResult { token_id: tid, drift: 0 });
             }
        }

        // 4. KNN 鲁棒搜索 (Robust Search)
        if let Some(nearest_coord) = self.find_nearest_neighbor_robust(&predicted_coord) {
            let token_prime = self.vocab_tensor.star_map.get(&nearest_coord).unwrap();
            let tid = self.vocab_tensor.prime_to_id.get(token_prime).unwrap();
            
            let drift = self.manhattan_distance(&predicted_coord, &nearest_coord);
            return Ok(DecodeResult { token_id: *tid, drift });
        }

        Err(format!("❌ Navigation Lost: No neighbors within radius {}.", self.search_radius))
    }

    /// 📏 [Ultrametric CPL]: Coarse-to-Fine Common Prefix Length
    /// 基于 20-bit (4 dims * 5 bits) 的前缀一致性度量。
    /// 
    /// **关键修正**: `extract_coordinate` 生成的是 Little-Endian (index 0 是 LSD)，
    /// 所以必须用 `.rev()` 从高维（Coarse）向低维（Fine）比较，
    /// 从而建立正确的层级观测。
    pub fn ultrametric_cpl_20bits(&self, a: &Coordinate, b: &Coordinate) -> u32 {
        let mut cpl: u32 = 0;

        // 从最高有效维度 (Coarse) 开始比较
        for (&da, &db) in a.iter().rev().zip(b.iter().rev()) {
            let xa = (da as u32) & 0x1F; // 确保只取 5 bits (side_len=32)
            let xb = (db as u32) & 0x1F;

            if xa == xb {
                cpl += 5; // 整个维度匹配
                continue;
            }

            // 维度内不匹配，计算 5-bit 窗口内的 MSB 前缀
            let diff = (xa ^ xb) & 0x1F;
            // 计算前导零，需减去无效的高位 (32 - 5 = 27)
            let lz = diff.leading_zeros().saturating_sub(27);
            cpl += lz.min(5);
            break; // 超度量特性：一旦高位不同，低位再像也没意义
        }

        cpl
    }

    /// 🔧 Apply Bias: 简单的模加性平移
    fn apply_bias_to_coord(&self, coord: &mut Coordinate, bias: &[usize]) {
        let l = self.vocab_tensor.side_length;
        // Bias 向量长度可能与坐标维度不同，取交集
        for (i, b) in bias.iter().enumerate().take(coord.len()) {
            coord[i] = (coord[i] + (b % l)) % l;
        }
    }

    /// 🌀 Semantic Lattice Projection (代数晶格投影)
    pub fn extract_coordinate(&self, tuple: &AffineTuple) -> Coordinate {
        let s = &tuple.q_shift; 
        
        let mut val = s.a.clone();
        let mut coord = Vec::new();
        
        let l = self.vocab_tensor.side_length as u64;
        let l_int = Integer::from(l);
        let dim = self.vocab_tensor.dimensions;

        for _ in 0..dim {
            let (q, r) = val.div_rem_ref(&l_int).into();
            let raw_remainder = r.to_u32().unwrap_or(0) as usize;
            
            // Logic: 偶数周期正向走，奇数周期反向走 (Zig-Zag)
            let mapped_val = if q.is_even() {
                raw_remainder
            } else {
                (self.vocab_tensor.side_length - 1) - raw_remainder
            };
            
            coord.push(mapped_val);
            val = q;
        }
        
        coord
    }
    
    // [HELPER]: 暴露曼哈顿距离计算
    pub fn calculate_distance(&self, a: &Coordinate, b: &Coordinate) -> usize {
        self.manhattan_distance(a, b)
    }

    /// 🔎 [Robust] K-D Tree Search
    fn find_nearest_neighbor_robust(&self, target: &Coordinate) -> Option<Coordinate> {
        let mut best_dist = usize::MAX;
        let mut best_coord = None;

        if let Some(ref root) = self.vocab_tensor.kd_tree {
            self.search_kdtree_recursive(root, target, &mut best_dist, &mut best_coord);
        }
        
        if best_dist > self.search_radius {
            return None;
        }

        best_coord
    }

    fn search_kdtree_recursive(
        &self, 
        node: &KdNode, 
        target: &Coordinate, 
        best_dist: &mut usize, 
        best_coord: &mut Option<Coordinate>
    ) {
        let d = self.manhattan_distance(&node.point, target);
        if d < *best_dist {
            *best_dist = d;
            *best_coord = Some(node.point.clone());
        }

        if *best_dist == 0 { return; }

        let axis = node.axis;
        let diff = (target[axis] as isize) - (node.point[axis] as isize);
        
        let (near, far) = if diff <= 0 {
            (&node.left, &node.right)
        } else {
            (&node.right, &node.left)
        };

        if let Some(ref child) = near {
            self.search_kdtree_recursive(child, target, best_dist, best_coord);
        }

        let axis_dist = diff.abs() as usize;
        if axis_dist < *best_dist {
            if let Some(ref child) = far {
                self.search_kdtree_recursive(child, target, best_dist, best_coord);
            }
        }
    }

    fn manhattan_distance(&self, a: &Coordinate, b: &Coordinate) -> usize {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| if x > y { x - y } else { y - x })
            .sum()
    }
}
