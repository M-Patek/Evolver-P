// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::affine::AffineTuple;
use crate::core::neuron::HTPNeuron;
use crate::core::algebra::ClassGroupElement;
use crate::core::primes::hash_to_prime;
use rug::Integer;
use std::sync::{Arc, RwLock};

/// 💎 EvolutionaryLayer: 并行神经元层
pub struct EvolutionaryLayer {
    /// [Thread-Safety]: 使用 RwLock 包装神经元，允许在训练时获取写锁进行突变
    pub neurons: Vec<Arc<RwLock<HTPNeuron>>>,
    pub width: usize,
}

impl EvolutionaryLayer {
    pub fn new(width: usize, dim: usize, side_len: usize, discriminant: Integer) -> Self {
        let mut neurons = Vec::new();
        for i in 0..width {
            let seed_str = format!("neuron_seed_{}_{}", dim, i);
            let p_weight = hash_to_prime(&seed_str, 128).unwrap();
            
            // 包装：HTPNeuron -> RwLock -> Arc
            let neuron = HTPNeuron::new(p_weight, dim, side_len, discriminant.clone());
            neurons.push(Arc::new(RwLock::new(neuron)));
        }
        EvolutionaryLayer { neurons, width }
    }

    /// 前向传播：Stream(In) -> [Neurons] -> Stream(Out)
    pub fn forward(&self, input_stream: &[AffineTuple], recursion_depth: usize) -> Result<Vec<AffineTuple>, String> {
        let mut output_stream = Vec::new();

        // ⚡ Parallel Activation
        // 在推理阶段，我们只需要获取 Read Lock (读锁)
        for neuron_arc in &self.neurons {
            let neuron_guard = neuron_arc.read().map_err(|_| "Neuron Lock Poisoned")?;
            
            // 激活神经元
            let (root, _proof) = neuron_guard.activate(input_stream.to_vec(), recursion_depth)?;
            output_stream.push(root);
        }

        Ok(output_stream)
    }
}

/// 🧠 HTPModel: The Crystal Brain
pub struct HTPModel {
    pub layers: Vec<EvolutionaryLayer>,
    pub discriminant: Integer,
}

impl HTPModel {
    pub fn new(layer_configs: Vec<(usize, usize, usize)>, discriminant: Integer) -> Self {
        let mut layers = Vec::new();
        for (width, dim, side_len) in layer_configs {
            layers.push(EvolutionaryLayer::new(width, dim, side_len, discriminant.clone()));
        }
        HTPModel { layers, discriminant }
    }

    pub fn embed(&self, token_ids: &[u32]) -> Result<Vec<AffineTuple>, String> {
        let mut stream = Vec::new();
        let generator = ClassGroupElement::generator(&self.discriminant);

        for &tid in token_ids {
            let token_str = format!("tok_{}", tid);
            let p = hash_to_prime(&token_str, 64).map_err(|e| e.to_string())?;
            stream.push(AffineTuple {
                p_factor: p,
                q_shift: generator.clone(),
            });
        }
        Ok(stream)
    }

    pub fn forward(&self, token_ids: &[u32]) -> Result<AffineTuple, String> {
        let mut current_stream = self.embed(token_ids)?;

        for (idx, layer) in self.layers.iter().enumerate() {
            current_stream = layer.forward(&current_stream, idx)?;
        }

        let mut final_root = AffineTuple::identity(&self.discriminant);
        for tuple in current_stream {
            final_root = final_root.compose(&tuple, &self.discriminant)?;
        }

        Ok(final_root)
    }
}
