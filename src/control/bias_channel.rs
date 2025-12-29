// src/control/bias_channel.rs
// 这是一个 "Sidecar" 控制器，通过低维偏置向量 (Bias Vector) 实时干预生成器的 Logits。
// 它利用 VAPO 算法在约束空间 (STP/p-adic) 中搜索最优的控制量 b。

use crate::dsl::schema::ProofAction;
use crate::dsl::stp_bridge::STPContext;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::f64::consts::PI;

// 假设词表大小或动作空间大小
const ACTION_SPACE_SIZE: usize = 1024; 
// 控制向量的维度 (低维控制，例如 16 或 32)
const BIAS_DIM: usize = 16;
// 环面模数 L (Bias Ring Size)
// 这定义了坐标环的大小 Z/LZ。这也决定了 VAPO 的搜索边界。
const BIAS_RING_SIZE: i32 = 256; 

/// 偏置向量 (Bias Vector)
/// 现在，这是一个定义在环面 T^n = (Z/LZ)^n 上的向量。
/// 所有的运算都必须是在模 L 意义下的。
#[derive(Debug, Clone, Hash)]
pub struct BiasVector {
    pub data: Vec<i32>, // 存储值范围应始终在 [0, BIAS_RING_SIZE)
    // 审计承诺 (Commitment Hash)，用于 ProofBundle
    #[hash(skip)] // 不参与自身的哈希计算，避免循环
    pub commitment: Option<String>,
}

impl BiasVector {
    pub fn new() -> Self {
        // 初始化为零偏置 (通常选择 L/2 作为中心可能更好，但在环面上 0 就是原点)
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

    /// 在环面上计算两个 Bias 向量之间的“曼哈顿环面距离”
    /// d(x, y) = sum( min(|x_i - y_i|, L - |x_i - y_i|) )
    /// 这是验证 Lipschitz 稳定性的关键指标。
    pub fn torus_distance(&self, other: &BiasVector) -> i32 {
        self.data.iter().zip(other.data.iter())
            .map(|(&a, &b)| {
                let raw_diff = (a - b).abs();
                std::cmp::min(raw_diff, BIAS_RING_SIZE - raw_diff)
            })
            .sum()
    }

    /// 应用扰动 (Perturbation) 并保持在环面上 (Wrap-around)
    /// val_new = (val_old + delta) mod L
    pub fn apply_perturbation(&mut self, idx: usize, delta: i32) {
        if idx < self.data.len() {
            // 使用 rem_euclid 确保结果总是正数 [0, L)
            self.data[idx] = (self.data[idx] + delta).rem_euclid(BIAS_RING_SIZE);
        }
    }

    /// 将环面坐标投影到 Logits 线性空间
    /// 
    /// # ⚠️ 几何连续性警告
    /// 如果直接线性投影 (w * b)，当 b 从 L-1 变为 0 时，Logits 会发生剧烈跳变。
    /// 为了保持 Lipschitz 连续性，我们必须使用循环嵌入 (Cyclic Embedding):
    /// b_i -> [sin(2*pi*b_i/L), cos(2*pi*b_i/L)]
    /// 这样，0 和 L 在 Logits 空间中是同一个点，消除了边界断裂。
    pub fn project_to_logits(&self) -> Vec<f64> {
        let mut logits_bias = vec![0.0; ACTION_SPACE_SIZE];
        
        for (i, &val) in self.data.iter().enumerate() {
            // 计算角度 theta = 2 * pi * val / L
            let theta = 2.0 * PI * (val as f64) / (BIAS_RING_SIZE as f64);
            let sin_component = theta.sin();
            let cos_component = theta.cos();

            // 每一个 Bias 维度控制一部分 Logits
            // 这里的投影矩阵 W 现在隐含地包含了 sin/cos 变换
            let start_idx = i * ACTION_SPACE_SIZE / BIAS_DIM;
            let end_idx = ((i + 1) * ACTION_SPACE_SIZE / BIAS_DIM).min(ACTION_SPACE_SIZE);
            
            for k in start_idx..end_idx {
                // 模拟投影：混合 sin 和 cos 分量
                // 这种映射保证了当 val 跨越边界 (255 -> 0) 时，输出是平滑过渡的
                logits_bias[k] += sin_component * 0.5 + cos_component * 0.5;
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
            
            // Valuation-Adaptive: 能量越大，扰动越剧烈
            // 注意：在环面上，"剧烈" 意味着步长较大，但仍然是模运算
            let perturbation_strength = if min_energy > 1.0 {
                rng.gen_range(-10..=10) // 粗调
            } else {
                rng.gen_range(-2..=2)   // 微调
            };
            
            // 使用环面加法应用扰动
            candidate_bias.apply_perturbation(dim_idx, perturbation_strength);

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
    fn test_bias_ring_homomorphism() {
        // 测试环面几何性质：Wrapping 和 Distance
        let mut b1 = BiasVector::new();
        // 设置为 L-1 (边界)
        b1.apply_perturbation(0, -1); 
        assert_eq!(b1.data[0], BIAS_RING_SIZE - 1);

        let mut b2 = BiasVector::new();
        // 设置为 1 (另一侧)
        b2.apply_perturbation(0, 1);
        assert_eq!(b2.data[0], 1);

        // 线性距离应该是 (L-1) - 1 = L-2 (很大)
        // 环面距离应该是 2 (很小)
        let dist = b1.torus_distance(&b2);
        assert_eq!(dist, 2, "Torus distance failed! Expected 2 (neighboring across boundary), got {}", dist);
    }

    #[test]
    fn test_vapo_optimization_loop() {
        // 1. 初始化环境
        let mut stp_ctx = STPContext::new();
        // 预定义环境...
        stp_ctx.commit_action(&ProofAction::Define { 
            symbol: "n".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        });
        stp_ctx.commit_action(&ProofAction::Define { 
            symbol: "m".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        });
        stp_ctx.commit_action(&ProofAction::Apply {
            theorem_id: "ModAdd".to_string(),
            inputs: vec!["n".to_string(), "m".to_string()],
            output_symbol: "sum_truth".to_string()
        });

        let mut controller = BiasController::new(None);
        
        // 2. 模拟 Base Logits
        let mut base_logits = vec![0.0; ACTION_SPACE_SIZE];
        base_logits[0] = 10.0; // Wrong
        base_logits[1] = 5.0;  // Correct

        let decode_fn = |logits: &[f64]| -> ProofAction {
            let max_idx = logits.iter().enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();

            if max_idx == 0 {
                 ProofAction::Define { 
                    symbol: "sum_truth".to_string(), 
                    hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
                }
            } else {
                 ProofAction::Define { 
                    symbol: "sum_truth".to_string(), 
                    hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Even".to_string()] 
                }
            }
        };

        // 3. 运行优化
        let (final_bias, _) = controller.optimize(&base_logits, &stp_ctx, decode_fn);

        // 4. 验证
        assert!(final_bias.commitment.is_some());
    }
}
