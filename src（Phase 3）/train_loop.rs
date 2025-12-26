// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::structure::HTPModel;
use crate::phase3::decoder::InverseDecoder;
use crate::phase3::core::primes::hash_to_prime;
use crate::phase3::core::affine::AffineTuple;
use crate::phase3::core::algebra::ClassGroupElement;

use std::sync::{Arc, RwLock};
use rand::{Rng, RngCore}; 
use rand::rngs::OsRng;     
use rug::Integer;

/// 突变策略枚举
enum MutationStrategy {
    HardReset,
    LocalShift,
}

/// 🧬 EvolutionaryTrainer: 进化训练器 (Enhanced with Memetic Search)
pub struct EvolutionaryTrainer {
    pub model: Arc<RwLock<HTPModel>>,
    pub decoder: InverseDecoder,
    pub learning_rate: f64, 
    pub gene_pool: Vec<Integer>,
    pub max_pool_size: usize,
}

impl EvolutionaryTrainer {
    pub fn new(model: Arc<RwLock<HTPModel>>, vocab_size: u32) -> Self {
        EvolutionaryTrainer {
            model,
            decoder: InverseDecoder::new(vocab_size),
            learning_rate: 0.05, 
            gene_pool: Vec::new(),
            max_pool_size: 200, 
        }
    }

    /// 🏋️ Train Step: 单步进化循环
    pub fn train_step(&mut self, input_ids: &[u32], target_id: u32) -> Result<f32, String> {
        let prediction_root = {
            let model_guard = self.model.read().map_err(|_| "Model Lock Poisoned")?;
            model_guard.forward(input_ids)?
        };

        let decode_result = self.decoder.decode(&prediction_root)
            .unwrap_or(crate::phase3::decoder::DecodeResult { token_id: u32::MAX, drift: usize::MAX });

        let is_target_hit = decode_result.token_id == target_id;
        let mut loss = 0.0;

        if !is_target_hit {
            loss = 1.0;
            self.punish_path_mutation();
        } 
        else if decode_result.drift > 0 {
            loss = 0.1 * (decode_result.drift as f32);
            let drift_risk = (decode_result.drift as f64) * 0.05; 
            
            let mut rng = rand::thread_rng();
            if rng.gen_bool(drift_risk.min(0.5)) { 
                self.apply_micro_mutation();
            }
        }
        else {
            loss = 0.0;
            self.reward_and_harvest();
        }

        Ok(loss)
    }

    fn reward_and_harvest(&mut self) {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.1) { 
             if let Ok(model_guard) = self.model.read() {
                 for layer in &model_guard.layers {
                     if let Some(neuron) = layer.neurons.choose(&mut rng) {
                         if let Ok(guard) = neuron.read() {
                             self.add_to_gene_pool(guard.p_weight.clone());
                         }
                     }
                 }
             }
        }
    }

    fn add_to_gene_pool(&mut self, gene: Integer) {
        if self.gene_pool.len() >= self.max_pool_size {
            self.gene_pool.remove(0); 
        }
        self.gene_pool.push(gene);
    }

    fn punish_path_mutation(&mut self) {
        self.mutate_network(MutationStrategy::HardReset);
    }

    fn apply_micro_mutation(&mut self) {
        self.mutate_network(MutationStrategy::LocalShift);
    }

    /// 通用突变逻辑 (Memetic Algorithm + Lipschitz Filter)
    fn mutate_network(&mut self, strategy: MutationStrategy) {
        let mut rng = rand::thread_rng(); 
        
        let mut model_guard = self.model.write().expect("Model Lock Poisoned during mutation");

        for layer in &mut model_guard.layers {
            for neuron_lock in &layer.neurons {
                if rng.gen_bool(self.learning_rate) {
                    
                    let mut neuron_mut = neuron_lock.write().expect("Neuron Lock Poisoned");

                    match strategy {
                        // [Strategy 1]: Hard Reset (Exploration)
                        MutationStrategy::HardReset => {
                            if !self.gene_pool.is_empty() && rng.gen_bool(0.3) {
                                let elite_gene = self.gene_pool.choose(&mut rng).unwrap();
                                neuron_mut.p_weight = elite_gene.clone(); 
                                if let Ok(mut memory_guard) = neuron_mut.memory.write() {
                                    memory_guard.data.clear();
                                    memory_guard.cached_root = None;
                                }
                            } else {
                                let mut entropy_bytes = [0u8; 32];
                                OsRng.fill_bytes(&mut entropy_bytes);
                                let entropy_hex: String = entropy_bytes.iter()
                                    .map(|b| format!("{:02x}", b))
                                    .collect();
                                let new_seed = format!("hard_mut_{}_{}", entropy_hex, neuron_mut.discriminant);
                                
                                if let Ok(new_prime) = hash_to_prime(&new_seed, 128) {
                                    neuron_mut.p_weight = new_prime;
                                    if let Ok(mut memory_guard) = neuron_mut.memory.write() {
                                        memory_guard.data.clear();
                                        memory_guard.cached_root = None;
                                    }
                                }
                            }
                        },
                        
                        // [Strategy 2]: Local Shift (Exploitation) with Lipschitz Filter
                        MutationStrategy::LocalShift => {
                            let current_p = &neuron_mut.p_weight;
                            
                            // 1. 捕获当前坐标 (基准点)
                            // 我们使用 Generator 作为标准输入来测量 P 的投影特性
                            let dummy_tuple_old = AffineTuple { 
                                p_factor: current_p.clone(), 
                                q_shift: ClassGroupElement::generator(&neuron_mut.discriminant).pow(current_p, &neuron_mut.discriminant).unwrap() 
                            };
                            let old_coord = self.decoder.extract_coordinate(&dummy_tuple_old);

                            // 2. 生成候选突变 (Trial)
                            let direction = if rng.gen_bool(0.5) { 1 } else { -1 };
                            let offset = Integer::from(rng.gen_range(100..10000));
                            
                            let candidate_base = if direction == 1 {
                                current_p.clone() + offset
                            } else {
                                let temp = current_p.clone() - offset;
                                if temp < 3 { Integer::from(3) } else { temp }
                            };
                            let new_prime = candidate_base.next_prime();

                            // 3. 预计算新坐标 (Simulation)
                            let dummy_tuple_new = AffineTuple { 
                                p_factor: new_prime.clone(), 
                                q_shift: ClassGroupElement::generator(&neuron_mut.discriminant).pow(&new_prime, &neuron_mut.discriminant).unwrap() 
                            };
                            let new_coord = self.decoder.extract_coordinate(&dummy_tuple_new);

                            // 4. Lipschitz 过滤器 (The Filter)
                            // 防止坐标瞬移。如果跳得太远，说明这个素数导致了投影空间的“断裂”。
                            // 阈值设为搜索半径的 2 倍。
                            let jump_distance = self.decoder.calculate_distance(&old_coord, &new_coord);
                            let continuity_threshold = self.decoder.search_radius * 2;

                            if jump_distance <= continuity_threshold {
                                // ✅ 接受：这是一个平滑的移动
                                neuron_mut.p_weight = new_prime;
                                if let Ok(mut memory_guard) = neuron_mut.memory.write() {
                                    memory_guard.data.clear();
                                    memory_guard.cached_root = None;
                                }
                            } else {
                                // ❌ 拒绝：这是不连续的跳变 (Continuity Trap)
                                // 保持原权重不变，等待下一次随机游走
                            }
                        }
                    }
                }
            }
        }
    }
}
