// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::evolution::gene::ProbeGene;
use rug::Integer;

/// 🧪 MutagenOracle: 诱变剂预言机 (Transformer Interface)
/// 这是一个 Trait，用于抽象化那个“上宽下窄”的 Transformer 探针网。
/// 它负责给出进化的“建议”，而不是“决定”。
pub trait MutagenOracle {
    /// [Macro-Mutation]: 宏观突变建议 (Hyper-Jump)
    /// 当探针陷入死胡同时，Transformer 建议跳跃到一个全新的素数域。
    /// 返回 Top-K 个建议的素数。
    fn suggest_macro_mutations(&self, context: &ProbeGene, k: usize) -> Vec<Integer>;

    /// [Micro-Mutation]: 微观突变建议 (Adaptive Radiation)
    /// 当探针方向正确时，Transformer 建议微调 Bias 向量。
    /// 返回 Top-K 个建议的偏差调整量。
    fn suggest_micro_mutations(&self, context: &ProbeGene, k: usize) -> Vec<Vec<usize>>;

    /// [Entropy]: 获取随机种子
    /// 用于产生“叛逆者”探针 (The Mutants)。
    fn get_entropy(&self) -> [u8; 32];
}

// --- Mock Implementation (for testing without a GPU model) ---

pub struct MockTransformer;

impl MutagenOracle for MockTransformer {
    fn suggest_macro_mutations(&self, _context: &ProbeGene, _k: usize) -> Vec<Integer> {
        // 在真实系统中，这里会运行神经网络 Beam Search
        // 这里返回一些固定的素数作为模拟
        vec![Integer::from(1009), Integer::from(1013), Integer::from(1019)] 
    }

    fn suggest_micro_mutations(&self, context: &ProbeGene, k: usize) -> Vec<Vec<usize>> {
        let mut suggestions = Vec::new();
        // 模拟：在当前 Bias 基础上做微小的随机扰动
        for i in 0..k {
             let mut new_bias = context.bias_vector.clone();
             if new_bias.is_empty() {
                 new_bias = vec![0; 4];
             }
             if !new_bias.is_empty() {
                 new_bias[0] = (new_bias[0] + i) % 100; // 简单的线性偏移模拟
             }
             suggestions.push(new_bias);
        }
        suggestions
    }

    fn get_entropy(&self) -> [u8; 32] {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
}
