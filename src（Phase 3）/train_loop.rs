// src/phase3/train_loop.rs
// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::structure::HTPModel;
use crate::phase3::decoder::InverseDecoder;
use crate::core::primes::hash_to_prime;
use std::sync::{Arc, RwLock};
use rand::{Rng, RngCore}; // [Updated Import]: 引入 RngCore 以支持 fill_bytes
use rand::rngs::OsRng;     // [Updated Import]: 引入操作系统级 CSPRNG
use rug::Integer;

/// 突变策略枚举
enum MutationStrategy {
    /// ☢️ Hard Reset: 彻底重置 (探索 Exploration)
    /// 用于处理幻觉。可能随机生成，也可能从基因池回溯。
    HardReset,
    
    /// 🔬 Local Shift: 局部游走 (利用 Exploitation)
    /// 用于消除漂移。在素数邻域内微调，模拟梯度下降。
    LocalShift,
}

/// 🧬 EvolutionaryTrainer: 进化训练器 (Enhanced with Memetic Search)
pub struct EvolutionaryTrainer {
    /// 模型本身被 RwLock 保护
    pub model: Arc<RwLock<HTPModel>>,
    pub decoder: InverseDecoder,
    pub learning_rate: f64, // 基础突变概率
    
    /// [FIX: Convergence Black-Box]
    /// 基因池 (Gene Pool): 存储历史上导致 "Zero Drift" 的成功素数权重
    /// 这打破了“死循环”，让进化有了方向记忆。
    pub gene_pool: Vec<Integer>,
    pub max_pool_size: usize,
}

impl EvolutionaryTrainer {
    pub fn new(model: Arc<RwLock<HTPModel>>, vocab_size: u32) -> Self {
        EvolutionaryTrainer {
            model,
            decoder: InverseDecoder::new(vocab_size),
            learning_rate: 0.05, // 5% 的概率发生突变
            gene_pool: Vec::new(),
            max_pool_size: 200, // 保留 200 个精英基因
        }
    }

    /// 🏋️ Train Step: 单步进化循环
    pub fn train_step(&mut self, input_ids: &[u32], target_id: u32) -> Result<f32, String> {
        // [Step 1]: Forward Pass (推理)
        let prediction_root = {
            let model_guard = self.model.read().map_err(|_| "Model Lock Poisoned")?;
            model_guard.forward(input_ids)?
        };

        // [Step 2]: Decode & Drift Check (验证与探针)
        let decode_result = self.decoder.decode(&prediction_root)
            .unwrap_or(crate::phase3::decoder::DecodeResult { token_id: u32::MAX, drift: usize::MAX });

        let is_target_hit = decode_result.token_id == target_id;
        let mut loss = 0.0;

        // [Step 3]: Evolution Strategy (进化策略)
        
        // Case A: 完全错误 -> 死刑 (Punish Mutation)
        if !is_target_hit {
            loss = 1.0;
            self.punish_path_mutation();
        } 
        // Case B: 命中但存在漂移 -> 精确性压力 (Precision Pressure)
        else if decode_result.drift > 0 {
            loss = 0.1 * (decode_result.drift as f32);
            let drift_risk = (decode_result.drift as f64) * 0.05; 
            
            let mut rng = rand::thread_rng();
            if rng.gen_bool(drift_risk.min(0.5)) { 
                self.apply_micro_mutation();
            }
        }
        // Case C: 完美命中 (Zero Drift) -> 奖励与收割 (Reward & Harvest)
        else {
            loss = 0.0;
            self.reward_and_harvest();
        }

        Ok(loss)
    }

    /// 🌾 Harvest: 收割精英基因
    fn reward_and_harvest(&mut self) {
        // 当我们获得完美推理时，当前的神经元配置是珍贵的。
        // 我们随机采样一部分当前网络的权重存入基因池。
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.1) { // 10% 的概率采样，防止池子更新太快
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
            self.gene_pool.remove(0); // 简单的 FIFO 淘汰
        }
        self.gene_pool.push(gene);
    }

    /// ☣️ Hard Mutation: 彻底重置
    fn punish_path_mutation(&mut self) {
        self.mutate_network(MutationStrategy::HardReset);
    }

    /// 🔬 Micro Mutation: 微扰突变
    fn apply_micro_mutation(&mut self) {
        self.mutate_network(MutationStrategy::LocalShift);
    }

    /// 通用突变逻辑 (Memetic Algorithm Implementation)
    fn mutate_network(&mut self, strategy: MutationStrategy) {
        // [PERFORMANCE NOTE]: 
        // 这里的 rng 仅用于决定是否发生突变 (概率判断) 和 LocalShift 的随机游走。
        // 对于关键的 HardReset 密钥生成，我们将在内部使用 OsRng。
        let mut rng = rand::thread_rng(); 
        
        let mut model_guard = self.model.write().expect("Model Lock Poisoned during mutation");

        for layer in &mut model_guard.layers {
            for neuron_lock in &layer.neurons {
                // 只有一定概率触发突变 (Learning Rate)
                if rng.gen_bool(self.learning_rate) {
                    
                    let mut neuron_mut = neuron_lock.write().expect("Neuron Lock Poisoned");

                    match strategy {
                        // [Strategy 1]: Hard Reset (Exploration)
                        MutationStrategy::HardReset => {
                            // 30% 概率从基因池复活 (Reincarnation)，70% 概率完全随机
                            if !self.gene_pool.is_empty() && rng.gen_bool(0.3) {
                                let elite_gene = self.gene_pool.choose(&mut rng).unwrap();
                                // 引入一点点突变，防止完全克隆
                                neuron_mut.p_weight = elite_gene.clone(); 
                                // Reset Memory
                                if let Ok(mut memory_guard) = neuron_mut.memory.write() {
                                    memory_guard.data.clear();
                                    memory_guard.cached_root = None;
                                }
                            } else {
                                // [SECURITY FIX]: 升级为 CSPRNG (Cryptographically Secure PRNG)
                                // 之前的 thread_rng().gen::<u64>() 熵不足 (64-bit) 且非密码学安全，
                                // 容易被攻击者通过监控进化路径来预测下一个素数权重。
                                // 这里我们从操作系统熵源获取 32 字节 (256-bit) 的真随机数。
                                let mut entropy_bytes = [0u8; 32];
                                OsRng.fill_bytes(&mut entropy_bytes);
                                
                                // 将随机字节转为十六进制字符串作为种子
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
                        
                        // [Strategy 2]: Local Shift (Exploitation)
                        // [FIX]: 不再随机重哈希，而是在素数空间游走
                        MutationStrategy::LocalShift => {
                            let current_p = &neuron_mut.p_weight;
                            
                            // 决定游走方向：变大还是变小
                            let direction = if rng.gen_bool(0.5) { 1 } else { -1 };
                            
                            // 寻找邻近的素数 (Simulated Gradient)
                            // 这里的随机性仅用于探索，不涉及密钥生成的安全性，因此 thread_rng 足够
                            let offset = Integer::from(rng.gen_range(100..10000));
                            let candidate_base = if direction == 1 {
                                current_p.clone() + offset
                            } else {
                                let temp = current_p.clone() - offset;
                                if temp < 1 { Integer::from(3) } else { temp }
                            };

                            let new_prime = candidate_base.next_prime();
                            
                            // 更新权重，保留记忆 (Soft Update)
                            neuron_mut.p_weight = new_prime;
                            if let Ok(mut memory_guard) = neuron_mut.memory.write() {
                                memory_guard.data.clear();
                                memory_guard.cached_root = None;
                            }
                        }
                    }
                }
            }
        }
    }
}
