use num_bigint::BigInt;
use num_traits::{Zero, Signed};
use crate::soul::algebra::ClassGroupElement;
use crate::will::perturber::{self, EnergyEvaluator};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// VAPO 优化器结构体
/// 负责在凯莱图上进行有状态的搜索，并记录“意志的轨迹”。
pub struct VapoOptimizer {
    pub current_seed: ClassGroupElement,
    pub trace: Vec<usize>, // 记录选择的 generator 索引，用于 Proof 重放
    generators: Vec<ClassGroupElement>, // 缓存生成元
    iteration_count: usize,
}

impl VapoOptimizer {
    /// 初始化优化器
    pub fn new(start_seed: ClassGroupElement) -> Self {
        // 在初始化时预生成所有扰动算子 (Generators)
        // 生产环境 VAPO 会使用更复杂的生成策略
        let delta = start_seed.discriminant();
        // 生成足够多的扰动算子以覆盖搜索空间
        let generators = perturber::generate_perturbations(&delta, 50);

        Self {
            current_seed: start_seed,
            trace: Vec::new(),
            generators,
            iteration_count: 0,
        }
    }

    /// 扰动 (Perturb): 意志的决策
    /// 
    /// [Hash Strategy Alignment]:
    /// 这里实现了 "Chaotic Determinism" (混沌决定论)。
    /// 意志的下一步不仅仅是随机的，而是由当前的代数状态(Soul)决定的。
    /// 我们对当前的 (a, b) 进行哈希，作为 RNG 的种子。
    /// 
    /// 这样做的目的是：
    /// 1. **可重放性**: 如果验证者重放 Trace，每一步的状态都确定，虽然验证不需要重放 RNG，但调试需要。
    /// 2. **结构耦合**: 搜索方向内在于代数结构本身。
    pub fn perturb(&mut self) -> (ClassGroupElement, usize) {
        let gen_count = self.generators.len();
        if gen_count == 0 {
            return (self.current_seed.clone(), 0);
        }

        // 1. 从当前状态提取熵 (State Hash)
        // 注意：这里使用 DefaultHasher 做内部状态哈希是可以的，
        // 因为这不涉及 Proof 的抗碰撞性，只涉及搜索路径的选择策略。
        // 但为了严谨，我们混合 iteration_count 防止死循环。
        let mut hasher = DefaultHasher::new();
        self.current_seed.a.hash(&mut hasher);
        self.current_seed.b.hash(&mut hasher);
        self.iteration_count.hash(&mut hasher);
        let state_hash = hasher.finish();

        // 2. 播种 RNG (Seed PRNG)
        let mut rng = StdRng::seed_from_u64(state_hash);

        // 3. 选择扰动 (Will's Choice)
        // VAPO 策略：在 active window 内选择。
        // 这里简化为全域加权选择。
        let idx = rng.gen_range(0..gen_count);
        let perturbation = &self.generators[idx];

        let candidate = self.current_seed.compose(perturbation);
        
        (candidate, idx)
    }

    /// 接受 (Accept)：确认这一步有效，更新状态并记录轨迹
    pub fn accept(&mut self, new_state: ClassGroupElement, generator_idx: usize) {
        self.current_seed = new_state;
        self.trace.push(generator_idx);
        self.iteration_count += 1;
    }
    
    // 增加计数器，即使拒绝也算一步，用于模拟退火的温度控制和随机数混合
    pub fn reject(&mut self) {
        self.iteration_count += 1;
    }
}
