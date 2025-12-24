// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::affine::AffineTuple;
use crate::core::neuron::HTPNeuron;
use crate::core::algebra::ClassGroupElement;
use crate::core::primes::hash_to_prime;
use rug::Integer;
use std::sync::Arc;

/// 💎 CrystalLayer: 并行神经元层
/// 一层包含多个神经元，它们同时观察输入流，从不同角度（不同的 p_weight）提取特征。
/// 输出：一个由各个神经元的 GlobalRoot 组成的新的 "Semantic Stream"。
pub struct CrystalLayer {
    pub neurons: Vec<Arc<HTPNeuron>>,
    pub width: usize,
}

impl CrystalLayer {
    pub fn new(width: usize, dim: usize, side_len: usize, discriminant: Integer) -> Self {
        let mut neurons = Vec::new();
        for i in 0..width {
            // 为每个神经元生成唯一的语义指纹 (Semantic Fingerprint)
            // 简单起见，我们用层索引和神经元索引来生成种子
            let seed_str = format!("neuron_seed_{}_{}", dim, i);
            let p_weight = hash_to_prime(&seed_str, 128).unwrap(); // 128-bit 语义权重
            
            neurons.push(Arc::new(HTPNeuron::new(p_weight, dim, side_len, discriminant.clone())));
        }
        CrystalLayer { neurons, width }
    }

    /// 前向传播：Stream(In) -> [Neurons] -> Stream(Out)
    pub fn forward(&self, input_stream: &[AffineTuple], recursion_depth: usize) -> Result<Vec<AffineTuple>, String> {
        let mut output_stream = Vec::new();

        // 并行激活每个神经元
        // TODO: 在生产环境中这里应该使用 Rayon 进行多线程并行
        for neuron in &self.neurons {
            // 每个神经元“吃掉”整个输入流，坍缩出一个 Global Root
            let (root, _proof) = neuron.activate(input_stream.to_vec(), recursion_depth)?;
            output_stream.push(root);
        }

        Ok(output_stream)
    }
}

/// 🧠 HTPModel: The Crystal Brain
/// 端到端纯代数生成模型
pub struct HTPModel {
    pub layers: Vec<CrystalLayer>,
    pub discriminant: Integer,
}

impl HTPModel {
    pub fn new(layer_configs: Vec<(usize, usize, usize)>, discriminant: Integer) -> Self {
        let mut layers = Vec::new();
        for (width, dim, side_len) in layer_configs {
            layers.push(CrystalLayer::new(width, dim, side_len, discriminant.clone()));
        }
        HTPModel { layers, discriminant }
    }

    /// 🌌 Embedding Layer: Token -> AffineTuple Stream
    /// 将离散的 Token ID 映射为代数流
    pub fn embed(&self, token_ids: &[u32]) -> Result<Vec<AffineTuple>, String> {
        let mut stream = Vec::new();
        let generator = ClassGroupElement::generator(&self.discriminant);

        for &tid in token_ids {
            let token_str = format!("tok_{}", tid);
            let p = hash_to_prime(&token_str, 64).map_err(|e| e.to_string())?;
            
            // 基础嵌入：(P, G)
            stream.push(AffineTuple {
                p_factor: p,
                q_shift: generator.clone(),
            });
        }
        Ok(stream)
    }

    /// ⚡ Forward Pass
    /// 输入 Tokens -> 经过多层代数坍缩 -> 输出最终的高维词根 (Target Root)
    pub fn forward(&self, token_ids: &[u32]) -> Result<AffineTuple, String> {
        // 1. Embedding
        let mut current_stream = self.embed(token_ids)?;

        // 2. Hidden Layers (The Folding Process)
        for (idx, layer) in self.layers.iter().enumerate() {
            // [Residual Connection]: 代数残差
            // 下一层的输入 = Layer(Input) * Input (如果维度匹配)
            // 这里简化为直接传递流
            current_stream = layer.forward(&current_stream, idx)?;
        }

        // 3. Final Collapse
        // 最后一层输出的 Stream 需要再次聚合为一个唯一的 Tuple，作为预测结果
        // 我们可以简单地将最后一层的输出再做一次 Compose
        let mut final_root = AffineTuple::identity(&self.discriminant);
        for tuple in current_stream {
            final_root = final_root.compose(&tuple, &self.discriminant)?;
        }

        Ok(final_root)
    }
}
