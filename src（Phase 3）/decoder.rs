// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::affine::AffineTuple;
use crate::core::primes::hash_to_prime;
use crate::topology::tensor::{HyperTensor, Coordinate};
use rug::Integer;
use std::collections::HashMap;

/// 🗺️ VocabularyTensor: 静态词汇宇宙
/// 这是一个预计算好的张量，将词表中的每个 Token 映射到一个固定的高维坐标上。
/// 在训练开始前，这个宇宙就是确定的。
pub struct VocabularyTensor {
    // Coordinate -> Token Prime
    // 这是一个反向查找表
    pub star_map: HashMap<Coordinate, Integer>,
    pub dimensions: usize,
    pub side_length: usize,
}

impl VocabularyTensor {
    /// 初始化词汇宇宙
    pub fn new(vocab_size: u32, dimensions: usize, side_length: usize) -> Self {
        let mut star_map = HashMap::new();
        // 简单的确定性映射：将 Token ID 均匀分布在超立方体中
        let l = side_length as u64;
        
        for i in 0..vocab_size {
            // 计算坐标
            let mut coord = Vec::new();
            let mut temp = i as u64;
            for _ in 0..dimensions {
                coord.push((temp % l) as usize);
                temp /= l;
            }

            // 计算该 Token 对应的素数
            let token_str = format!("tok_{}", i);
            if let Ok(p) = hash_to_prime(&token_str, 64) {
                star_map.insert(coord, p);
            }
        }

        VocabularyTensor {
            star_map,
            dimensions,
            side_length,
        }
    }
}

/// 🧭 InverseDecoder: 坐标导航器
pub struct InverseDecoder {
    pub vocab_tensor: VocabularyTensor,
}

impl InverseDecoder {
    pub fn new(vocab_size: u32) -> Self {
        // 假设我们使用 4维, 边长32 的张量来容纳词表 (32^4 > 100万)
        InverseDecoder {
            vocab_tensor: VocabularyTensor::new(vocab_size, 4, 32),
        }
    }

    /// 📍 Decode: Target Root -> Coordinate -> Token
    /// 解析模型输出的“高维词根”，还原为 Token
    pub fn decode(&self, target_root: &AffineTuple) -> Result<u32, String> {
        // 1. Extract Coordinate from Algebraic Structure
        // 这是一个关键的“投影”步骤。
        // 我们需要从 target_root (P, Q) 中提取出坐标信息。
        // 方案：利用 P_factor 的模运算作为坐标哈希。
        let predicted_coord = self.extract_coordinate(target_root);

        // 2. Spatial Lookup (查表)
        if let Some(token_prime) = self.vocab_tensor.star_map.get(&predicted_coord) {
             // 找到了！精确命中！
             // 在实际中，这里需要一个反向映射 Prime -> TokenID，或者遍历匹配
             // 为了演示，我们假设我们能直接反推 (或者在 map 里存的就是 ID)
             return Ok(self.prime_to_token_id_hack(token_prime));
        }

        // 3. Nearest Neighbor Search (模糊导航)
        // 如果没有精确命中，搜索最近的邻居 (纠错机制)
        // Master, 这里是处理“语义漂移”的好地方喵！
        // ... (省略 KNN 实现)

        Err("❌ Navigation Lost: Target coordinates point to empty void.".to_string())
    }

    fn extract_coordinate(&self, tuple: &AffineTuple) -> Coordinate {
        let mut coord = Vec::new();
        let l = self.vocab_tensor.side_length;
        let dim = self.vocab_tensor.dimensions;
        
        // 简单的提取逻辑：使用 P_factor 的位片段
        // 在实际训练中，模型会学习调整 P_factor 以匹配目标坐标
        let mut val = tuple.p_factor.to_u64_wrapping(); // 取低 64 位
        
        for _ in 0..dim {
            coord.push((val as usize) % l);
            val /= l as u64;
        }
        coord
    }

    // 仅作演示的 Hack，实际需要完整的双向 Map
    fn prime_to_token_id_hack(&self, _p: &Integer) -> u32 {
        42 // The answer to everything
    }
}
