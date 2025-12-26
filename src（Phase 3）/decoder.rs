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
        // [Mapping Strategy]: 
        // Token 被放置在固定的“家”中 (Static Addressing)。
        // 模型的任务是演化状态 S，使得 Project(S) 精确指向这个家。
        for tid in 0..vocab_size {
            // 1. 计算确定性坐标 (Linear Layout)
            // 这里我们使用简单的线性填充，因为投影函数 Project(S) 是连续的。
            // 模型可以通过调整权重来"爬升"到任意坐标。
            let mut coord = Vec::with_capacity(dimensions);
            let mut temp = tid as u64;
            for _ in 0..dimensions {
                coord.push((temp % l) as usize);
                temp /= l;
            }

            // 2. [DCAP Algorithm]: 生成绝对唯一的 Token Prime
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
        panic!("❌ Fatal Error: Vocabulary Space Exhausted.");
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
pub struct InverseDecoder {
    pub vocab_tensor: VocabularyTensor,
    /// 动态搜索半径：如果直接找不到，允许在多大范围内搜索
    pub search_radius: usize,
}

impl InverseDecoder {
    pub fn new(vocab_size: u32) -> Self {
        InverseDecoder {
            vocab_tensor: VocabularyTensor::new(vocab_size, 4, 32),
            search_radius: 5, // 默认允许一定的模糊导航
        }
    }

    /// 📍 Decode: S_state -> Coordinate -> Nearest Token
    pub fn decode(&self, target_root: &AffineTuple) -> Result<DecodeResult, String> {
        // 1. Extract Coordinate via Semantic Projection (Lattice Mapping)
        let predicted_coord = self.extract_coordinate(target_root);

        // 2. Exact Match Check (Zero Drift)
        if let Some(token_prime) = self.vocab_tensor.star_map.get(&predicted_coord) {
             if let Some(&tid) = self.vocab_tensor.prime_to_id.get(token_prime) {
                 return Ok(DecodeResult { token_id: tid, drift: 0 });
             }
        }

        // 3. Robust KNN Search (Non-Zero Drift)
        // 这里的 "Drift" 现在代表真实的代数距离误差。
        if let Some(nearest_coord) = self.find_nearest_neighbor_robust(&predicted_coord) {
            let token_prime = self.vocab_tensor.star_map.get(&nearest_coord).unwrap();
            let tid = self.vocab_tensor.prime_to_id.get(token_prime).unwrap();
            let drift = self.manhattan_distance(&predicted_coord, &nearest_coord);
            
            return Ok(DecodeResult { token_id: *tid, drift });
        }

        Err("❌ Navigation Lost: State drifted too far from semantic manifold.".to_string())
    }

    /// 🌀 [CORE REWRITE]: Semantic Lattice Projection (代数晶格投影)
    /// 
    /// [FIXED]: 移除了 Phase 2 的哈希映射。
    /// 现在我们将 ClassGroupElement 视为高维晶格上的点，
    /// 通过**模形式分解 (Integer Decomposition)** 将其投影到 Tensor 坐标系。
    /// 
    /// 数学意义：
    /// S.a (Ideal Norm) 的微小变化（加减）会直接映射为 Coordinate 的微小位移。
    /// 这恢复了 "LocalShift" 训练策略的梯度语义：
    /// 调整权重 -> S 微变 -> 坐标微变 -> Drift 降低。
    fn extract_coordinate(&self, tuple: &AffineTuple) -> Coordinate {
        let s = &tuple.q_shift; 
        
        // 使用 'a' 系数 (Norm of the Ideal) 作为主要的投影源。
        // 在类群中，a 的变化直接反映了理想类的结构变化。
        // 我们将其按 Tensor 的边长 L 进行进制分解 (Base-L Expansion)。
        let mut val = s.a.clone();
        
        let mut coord = Vec::new();
        let l = self.vocab_tensor.side_length as u64;
        let dim = self.vocab_tensor.dimensions;
        
        let l_int = Integer::from(l);

        for _ in 0..dim {
            // coord[i] = val % L
            // val = val / L
            // 这建立了一个连续的覆盖映射 (Covering Map)
            let (q, r) = val.div_rem_ref(&l_int).into();
            
            // r 是余数，必然 < l，安全转换
            coord.push(r.to_u32().unwrap_or(0) as usize);
            val = q;
        }
        
        coord
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
