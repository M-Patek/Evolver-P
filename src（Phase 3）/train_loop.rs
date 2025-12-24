// src/phase3/train_loop.rs
// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::structure::HTPModel;
use crate::phase3::decoder::InverseDecoder;
use crate::core::primes::hash_to_prime;
use std::sync::{Arc, RwLock};
use rand::Rng;

/// 🧬 EvolutionaryTrainer: 进化训练器
pub struct EvolutionaryTrainer {
    /// 模型本身被 RwLock 保护，以便我们可以修改其结构或参数
    pub model: Arc<RwLock<HTPModel>>,
    pub decoder: InverseDecoder,
    pub learning_rate: f64, // 突变概率 (Mutation Probability)
}

impl EvolutionaryTrainer {
    pub fn new(model: Arc<RwLock<HTPModel>>, vocab_size: u32) -> Self {
        EvolutionaryTrainer {
            model,
            decoder: InverseDecoder::new(vocab_size),
            learning_rate: 0.05, // 5% 的概率发生突变
        }
    }

    /// 🏋️ Train Step: 单步进化循环
    pub fn train_step(&mut self, input_ids: &[u32], target_id: u32) -> Result<f32, String> {
        // [Step 1]: Forward Pass (推理)
        // 获取模型读锁，进行计算
        let prediction_root = {
            let model_guard = self.model.read().map_err(|_| "Model Lock Poisoned")?;
            model_guard.forward(input_ids)?
        };

        // [Step 2]: Decode & Check (验证)
        let predicted_id = self.decoder.decode(&prediction_root)
            .unwrap_or(u32::MAX); // 如果导航失败，设为 MAX

        let is_correct = predicted_id == target_id;
        
        // Loss 仅用于监控，不用于梯度
        let loss = if is_correct { 0.0 } else { 1.0 };

        // [Step 3]: Evolution (进化)
        if is_correct {
            self.reward_path();
        } else {
            // 预测错误 -> 触发突变
            self.punish_path_mutation();
        }

        Ok(loss)
    }

    fn reward_path(&self) {
        // 正确的路径不需要改变，这就是最好的奖励。
        // 可选：记录日志
        // println!("✨ Logic Path Validated.");
    }

    /// ☣️ Mutation Logic: 核心代码
    /// 这里演示了如何穿透 Arc 和 RwLock 来修改底层数据
    fn punish_path_mutation(&mut self) {
        let mut rng = rand::thread_rng();
        
        // 1. 获取模型的写锁 (Write Lock)
        // 这会暂时阻塞所有的读取操作，确保突变时的独占访问
        let mut model_guard = self.model.write().expect("Model Lock Poisoned during mutation");

        // println!("💥 Mutation triggered: Rewiring neurons...");

        // 2. 遍历每一层
        for layer in &mut model_guard.layers {
            // 3. 随机遍历神经元
            for neuron_lock in &layer.neurons {
                // 根据学习率决定是否突变这个神经元
                if rng.gen_bool(self.learning_rate) {
                    
                    // 4. 获取神经元的写锁 (关键步骤！)
                    // 这里的 `write()` 让我们获得了 `&mut HTPNeuron`
                    let mut neuron_mut = neuron_lock.write().expect("Neuron Lock Poisoned");

                    // 5. 执行突变：改变语义指纹 (p_weight)
                    // 使用新的随机种子生成素数
                    let new_seed = format!("mutated_{}_{}", 
                        rng.gen::<u64>(), 
                        neuron_mut.discriminant // 混入一些熵
                    );

                    match hash_to_prime(&new_seed, 128) {
                        Ok(new_prime) => {
                            // [Action A]: 更新权重
                            neuron_mut.p_weight = new_prime;

                            // [Action B]: 清空记忆张量
                            // 因为语义变了，旧的记忆变成了垃圾数据，必须清除
                            // memory 也是一个 Arc<RwLock>，需要再次获取写锁
                            if let Ok(mut memory_guard) = neuron_mut.memory.write() {
                                memory_guard.data.clear();
                                memory_guard.cached_root = None;
                            }

                            // println!("   🧬 Neuron re-hashed.");
                        },
                        Err(_) => {
                            // 如果生成素数失败（极罕见），跳过
                            continue;
                        }
                    }
                }
            }
        }
    }
}
