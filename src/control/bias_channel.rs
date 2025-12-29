// src/control/bias_channel.rs
// 这是一个 "Sidecar" 控制器，通过低维偏置向量 (Bias Vector) 实时干预生成器的 Logits。
// 它利用 VAPO 算法在约束空间 (STP/p-adic) 中搜索最优的控制量 b。

use crate::dsl::schema::ProofAction;
use crate::dsl::stp_bridge::STPContext;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// 假设词表大小或动作空间大小
const ACTION_SPACE_SIZE: usize = 1024; 
// 控制向量的维度 (低维控制，例如 16 或 32)
const BIAS_DIM: usize = 16;

/// 偏置向量 (Bias Vector)
/// 使用整数向量模拟 p-进整数的位数操作，符合 VAPO 的离散优化特性。
/// 增加了 Commitment 字段，对应 THEORY.md 中的 ProofBundle 要求。
#[derive(Debug, Clone, Hash)]
pub struct BiasVector {
    pub data: Vec<i32>, // 每一位对应一个控制维度的值
    // 审计承诺 (Commitment Hash)，用于 ProofBundle
    #[hash(skip)] // 不参与自身的哈希计算，避免循环
    pub commitment: Option<String>,
}

impl BiasVector {
    pub fn new() -> Self {
        // 初始化为零偏置
        BiasVector {
            data: vec![0; BIAS_DIM],
            commitment: None,
        }
    }

    /// 计算并锁定该 Bias 的承诺 (Commitment)
    /// 这对应于 "GlobalRoot_bias" 的生成过程
    pub fn seal(&mut self) -> String {
        let mut hasher = DefaultHasher::new();
        self.data.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());
        self.commitment = Some(hash.clone());
        hash
    }

    /// 将低维 Bias 投影到高维 Logits 空间 (模拟 W_proj * b)
    pub fn project_to_logits(&self) -> Vec<f64> {
        let mut logits_bias = vec![0.0; ACTION_SPACE_SIZE];
        for (i, &val) in self.data.iter().enumerate() {
            // 简单的散列投影模拟：每一维 bias 影响一部分 logits
            let start_idx = (i * ACTION_SPACE_SIZE / BIAS_DIM);
            let end_idx = ((i + 1) * ACTION_SPACE_SIZE / BIAS_DIM).min(ACTION_SPACE_SIZE);
            
            for k in start_idx..end_idx {
                // 模拟某种线性关系
                logits_bias[k] += (val as f64) * 0.1; 
            }
        }
        logits_bias
    }
}

/// 审计记录 (Audit Record)
/// 记录每一次修正的快照，用于事后验证 (Proof of Logic)
#[derive(Debug, Clone)]
pub struct BiasAuditRecord {
    pub timestamp: u64, // 逻辑时钟
    pub commitment: String, // Bias Hash
    pub bias_snapshot: Vec<i32>,
    pub final_energy: f64,
}

/// VAPO 优化器配置
pub struct VapoConfig {
    pub max_iterations: usize, // 最大搜索步数 (实时性要求高，不能太大)
    pub initial_temperature: f64,
    pub valuation_decay: f64, // 估值衰减系数
}

impl Default for VapoConfig {
    fn default() -> Self {
        VapoConfig {
            max_iterations: 50,
            initial_temperature: 1.0,
            valuation_decay: 0.9,
        }
    }
}

/// Bias Channel 控制器
pub struct BiasController {
    current_bias: BiasVector,
    config: VapoConfig,
    // 审计日志：存储所有的 ProofBundle
    pub audit_log: Vec<BiasAuditRecord>,
}

impl BiasController {
    pub fn new(config: Option<VapoConfig>) -> Self {
        BiasController {
            current_bias: BiasVector::new(),
            config: config.unwrap_or_default(),
            audit_log: Vec::new(),
        }
    }

    /// VAPO 核心循环：搜索最优 Bias 以最小化 STP 能量
    /// 
    /// # 参数
    /// - `base_logits`: 生成器原始输出的 Logits
    /// - `stp_ctx`: 代数状态上下文 (只读引用，使用纯函数计算能量)
    /// - `decoder_simulation`: 一个闭包，模拟 "Logits -> ProofAction" 的解码过程
    /// 
    /// # 返回
    /// - `BiasVector`: 修正后并已 Seal 的偏置向量 (Auditable Artifact)
    /// - `ProofAction`: 修正后的动作
    pub fn optimize<F>(
        &mut self,
        base_logits: &[f64],
        stp_ctx: &STPContext,
        decode_fn: F,
    ) -> (BiasVector, ProofAction) 
    where
        F: Fn(&[f64]) -> ProofAction, // 模拟解码器：Logits -> Action
    {
        // 从上一次的 Bias 开始搜索 (保持控制连续性)
        let mut best_bias = self.current_bias.clone();
        let mut best_action = decode_fn(&Self::apply_bias(base_logits, &best_bias));
        
        // 使用纯函数计算能量，不修改 Context
        let mut min_energy = stp_ctx.calculate_energy(&best_action);

        // 如果初始能量已经是 0 (完美逻辑)，直接 Seal 并返回
        if min_energy <= 1e-6 {
            self.record_artifact(&mut best_bias, min_energy);
            return (best_bias, best_action);
        }

        let mut rng = rand::thread_rng();
        let mut temperature = self.config.initial_temperature;

        // VAPO 搜索循环
        for _iter in 0..self.config.max_iterations {
            // 1. 生成扰动 (Perturbation)
            let mut candidate_bias = best_bias.clone();
            let dim_idx = rng.gen_range(0..BIAS_DIM);
            
            // Valuation-Adaptive: 能量越大，扰动越剧烈 (低估值位)
            let perturbation_strength = if min_energy > 1.0 {
                rng.gen_range(-5..=5)
            } else {
                rng.gen_range(-1..=1)
            };
            
            candidate_bias.data[dim_idx] += perturbation_strength;

            // 2. 应用 Bias 并解码
            let modified_logits = Self::apply_bias(base_logits, &candidate_bias);
            let candidate_action = decode_fn(&modified_logits);

            // 3. 计算新能量 (Pure)
            let new_energy = stp_ctx.calculate_energy(&candidate_action);

            // 4. Metropolis-Hastings 接受准则
            let delta_e = new_energy - min_energy;
            if delta_e < 0.0 || rng.gen::<f64>() < (-delta_e / temperature).exp() {
                best_bias = candidate_bias;
                min_energy = new_energy;
                best_action = candidate_action;
                
                if min_energy <= 1e-6 {
                    break;
                }
            }

            temperature *= self.config.valuation_decay;
        }

        // 5. 记录审计产物 (Bias Commitment)
        self.record_artifact(&mut best_bias, min_energy);
        
        // 更新内部状态
        self.current_bias = best_bias.clone();
        
        (best_bias, best_action)
    }

    /// 内部方法：Seal bias 并写入审计日志
    fn record_artifact(&mut self, bias: &mut BiasVector, energy: f64) {
        let commitment = bias.seal();
        self.audit_log.push(BiasAuditRecord {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            commitment,
            bias_snapshot: bias.data.clone(),
            final_energy: energy,
        });
    }

    /// 将 Bias 叠加到 Base Logits 上
    fn apply_bias(base: &[f64], bias: &BiasVector) -> Vec<f64> {
        let bias_proj = bias.project_to_logits();
        base.iter().zip(bias_proj.iter())
            .map(|(b, p)| b + p)
            .collect()
    }
}

// -------------------------------------------------------------------------
// Mock Test
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::schema::ProofAction;
    use crate::dsl::stp_bridge::STPContext;

    #[test]
    fn test_vapo_audit_recording() {
        // 1. 初始化环境
        let mut stp_ctx = STPContext::new();
        // 预定义 n 为 Odd, m 为 Odd. 
        // 并在 Context 中预演 Apply ModAdd -> sum_truth (Even)
        // 这样后续的 Calculate Energy 才能工作
        stp_ctx.commit_action(&ProofAction::Define { 
            symbol: "n".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        });
        stp_ctx.commit_action(&ProofAction::Define { 
            symbol: "m".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        });
        // 预演真理推导：ModAdd(n,m) -> sum_truth (Even)
        stp_ctx.commit_action(&ProofAction::Apply {
            theorem_id: "ModAdd".to_string(),
            inputs: vec!["n".to_string(), "m".to_string()],
            output_symbol: "sum_truth".to_string()
        });

        let mut controller = BiasController::new(None);
        
        // 2. 模拟 Base Logits (倾向于错误)
        // Index 0: Define sum_truth Odd (Wrong)
        // Index 1: Define sum_truth Even (Correct)
        let mut base_logits = vec![0.0; ACTION_SPACE_SIZE];
        base_logits[0] = 10.0; 
        base_logits[1] = 5.0;

        let decode_fn = |logits: &[f64]| -> ProofAction {
            let max_idx = logits.iter().enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();

            if max_idx == 0 {
                // 错误动作
                ProofAction::Define { 
                    symbol: "sum_truth".to_string(), 
                    hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
                }
            } else {
                // 正确动作
                ProofAction::Define { 
                    symbol: "sum_truth".to_string(), 
                    hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Even".to_string()] 
                }
            }
        };

        // 3. 运行优化 (传递只读引用)
        let (final_bias, _final_action) = controller.optimize(&base_logits, &stp_ctx, decode_fn);

        // 4. 验证审计
        assert!(final_bias.commitment.is_some(), "Bias should be sealed with a commitment");
        assert_eq!(controller.audit_log.len(), 1, "Audit log should have one record");
        
        let record = &controller.audit_log[0];
        assert_eq!(record.commitment, final_bias.commitment.unwrap());
        assert!(record.final_energy < 0.1, "Final energy should be near zero");
        
        println!("Audit Record Verified: Commitment={}", record.commitment);
    }
}
