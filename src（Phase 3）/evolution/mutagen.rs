// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::Integer;
use rand::{Rng, thread_rng};
use std::collections::HashMap;

/// 🎯 Track A: Bias VAPO (Valuation-Adaptive Perturbation)
/// 专精于在整数格点上的多尺度微调。
/// 
/// 这一层负责解决“连续性陷阱”中的可控性问题。
/// 通过对 Bias 向量的不同位（Digit）进行敏感度分析，实现“手术刀”式的微调。
pub struct BiasVapo {
    /// 敏感度分数 (Per-Dimension Scores)
    /// 这里的 index 对应 Bias 向量的 index。
    /// 根据 Coordinate 定义，index 越大 = 维度越高 = 越 Coarse。
    /// index 0 = LSD (Fine Detail), index N = MSD (Global Structure).
    pub scores: Vec<f64>,
}

impl BiasVapo {
    pub fn new(dims: usize) -> Self {
        // 初始时对所有层级一视同仁
        BiasVapo { scores: vec![1.0; dims] } 
    }

    /// 执行突变并返回被修改的层级 index (用于后续反馈)
    /// 
    /// 策略：
    /// 1. 轮盘赌选择修改哪一位（基于敏感度）。
    /// 2. 施加 +/- 1 的最小步长扰动（符合 VAPO 思想，尺度由位的位置决定，而非步长大小）。
    pub fn mutate(&self, bias: &mut Vec<usize>, side_len: usize) -> usize {
        let mut rng = thread_rng();
        
        // 1. 轮盘赌选择层级 (Softmax-like selection)
        let total_score: f64 = self.scores.iter().sum();
        // 防止全零
        let effective_total = if total_score <= 0.0 { 1.0 } else { total_score };
        
        let mut pick = rng.gen_range(0.0..effective_total);
        let mut level = 0;
        
        for (i, &s) in self.scores.iter().enumerate() {
            pick -= s;
            if pick <= 0.0 {
                level = i;
                break;
            }
        }
        // 边界保护
        if level >= bias.len() { level = bias.len() - 1; }
        
        // 2. VAPO 核心：步长固定为 +/- 1 (Mod P)
        // 这里的 level 对应 Coordinate 的 index，实际上就是 p-adic 展开的第 level 位
        let delta = if rng.gen_bool(0.5) { 1 } else { side_len - 1 }; // +1 or -1 (mod side_len)
        
        // 确保 bias 向量长度足够
        if bias.len() < self.scores.len() {
            bias.resize(self.scores.len(), 0);
        }

        if level < bias.len() {
            bias[level] = (bias[level] + delta) % side_len;
        }
        
        level
    }

    /// 反馈更新：动量法
    /// 如果某一层级的修改带来了 Fitness 的提升，增加其被选中的概率。
    pub fn update_feedback(&mut self, level: usize, reward: f64) {
        if level < self.scores.len() {
            // 动量更新：保留 90% 历史，吸收 10% 新知识
            self.scores[level] = 0.9 * self.scores[level] + 0.1 * reward;
        }
    }
}

/// 🎲 Track B: Prime Adaptive Search
/// 专精于在混沌类群上的经验探索。
/// 
/// 这一层负责解决“连续性陷阱”中的遍历性问题。
/// 由于 Prime 映射是混沌的，我们无法计算梯度，只能依赖统计学习（Thompson Sampling）
/// 来决定是“就近搜索”还是“远程跳跃”。
pub struct PrimeAdaptive {
    /// 候选策略池
    /// 0: Next Prime (微小步 - 试图利用偶发的局部连续性)
    /// 1: Small Jump (+1..100 - 局部搜索)
    /// 2: Re-Hash (重置 - 全局搜索)
    pub strategies: Vec<u8>,
    /// 成功率统计 (Success, Total)
    /// 存储每个策略的 Alpha, Beta 参数，用于 Beta 分布采样
    pub stats: HashMap<u8, (u64, u64)>,
}

impl PrimeAdaptive {
    pub fn new() -> Self {
        PrimeAdaptive {
            strategies: vec![0, 1, 2],
            stats: HashMap::new(),
        }
    }

    /// Thompson Sampling 选择策略
    /// 从 Beta 分布中采样，以此平衡 Exploration 和 Exploitation。
    pub fn select_strategy(&self) -> u8 {
        let mut best_score = -1.0;
        let mut best_strat = 0;
        let mut rng = thread_rng();

        for &strat in &self.strategies {
            // 默认为 Beta(1, 2) 先验，略微悲观，鼓励尝试
            let (success, total) = self.stats.get(&strat).unwrap_or(&(1, 2)); 
            
            // Beta 分布采样 (模拟)
            let sample = self.beta_sample(*success as f64, (*total - *success) as f64);
            
            if sample > best_score {
                best_score = sample;
                best_strat = strat;
            }
        }
        
        // Epsilon-Greedy Exploration (10% 强制随机探索)
        if rng.gen_bool(0.1) {
            return self.strategies[rng.gen_range(0..self.strategies.len())];
        }
        
        best_strat
    }

    pub fn generate(&self, strategy: u8, current_p: &Integer) -> Integer {
        let mut rng = thread_rng();
        match strategy {
            0 => current_p.next_prime(),
            1 => {
                let offset = rng.gen_range(1..100);
                (current_p.clone() + offset).next_prime()
            },
            _ => {
                // Hyper Jump: 使用高熵源重新生成
                crate::phase3::core::primes::hash_to_prime("hyper_jump_adaptive", 64)
                    .unwrap_or_else(|_| Integer::from(3))
            }
        }
    }

    /// 更新统计数据
    pub fn update_stats(&mut self, strategy: u8, success: bool) {
        let entry = self.stats.entry(strategy).or_insert((1, 2)); 
        entry.1 += 1; // Total + 1
        if success {
            entry.0 += 1; // Success + 1
        }
    }

    fn beta_sample(&self, alpha: f64, beta: f64) -> f64 {
        // 简化的 Beta 采样模拟: X / (X + Y) where X~Gamma(a,1), Y~Gamma(b,1)
        // 这里用 powf(1/a) 近似 Gamma 分布的形状特征用于比较
        let mut rng = thread_rng();
        let x = rng.gen::<f64>().powf(1.0/alpha);
        let y = rng.gen::<f64>().powf(1.0/beta);
        x / (x + y)
    }
}
