use num_bigint::BigInt;
use num_traits::{Zero, Signed};
use crate::soul::algebra::ClassGroupElement;
use crate::will::perturber::{self, EnergyEvaluator};
use crate::dsl::schema::GeneratorSpec;
use crate::will::tracer::WillTracer;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// VAPO 优化器结构体
/// 负责在凯莱图上进行有状态的搜索。
///
/// [Refactor Note]:
/// 移除了 `trace` 和 `generator_spec` 字段。
/// 轨迹记录现在通过 `WillTracer` 接口由外部注入。
/// 这使得优化器在 "原生模式" 下零开销，且更专注于搜索本身。
pub struct VapoOptimizer {
    pub current_seed: ClassGroupElement,
    // trace: Vec<usize>, // 已移除：通过 WillTracer 解耦
    generators: Vec<ClassGroupElement>, // 缓存生成元
    iteration_count: usize,
    
    // generator_spec: GeneratorSpec, // 已移除：元数据移至 ProofBundle 构建逻辑
}

impl VapoOptimizer {
    /// 默认使用的生成元数量，与 production 环境保持一致
    pub const DEFAULT_GEN_COUNT: usize = 50;

    /// 初始化优化器
    pub fn new(start_seed: ClassGroupElement) -> Self {
        // 在初始化时预生成所有扰动算子 (Generators)
        let delta = start_seed.discriminant();
        
        // 定义生成策略
        let generators = perturber::generate_perturbations(&delta, Self::DEFAULT_GEN_COUNT);

        Self {
            current_seed: start_seed,
            generators,
            iteration_count: 0,
        }
    }

    /// 获取优化器使用的默认生成元规格
    /// 这是一个辅助方法，用于外部 (PyEvolver) 构建 ProofBundle 时获取版本信息，
    /// 而不需要优化器实例持有该数据。
    pub fn default_spec() -> GeneratorSpec {
        GeneratorSpec {
            algorithm_version: perturber::ALGORITHM_VERSION.to_string(),
            count: Self::DEFAULT_GEN_COUNT,
            max_norm: None,
        }
    }

    /// 获取当前的生成元列表长度
    pub fn generator_count(&self) -> usize {
        self.generators.len()
    }

    /// 扰动 (Perturb): 意志的决策
    /// 
    /// [Hash Strategy Alignment]:
    /// 这里实现了 "Chaotic Determinism" (混沌决定论)。
    /// 意志的下一步不仅仅是随机的，而是由当前的代数状态(Soul)决定的。
    pub fn perturb(&mut self) -> (ClassGroupElement, usize) {
        let gen_count = self.generators.len();
        if gen_count == 0 {
            return (self.current_seed.clone(), 0);
        }

        // 1. 从当前状态提取熵 (State Hash)
        // 混合 iteration_count 防止死循环
        let mut hasher = DefaultHasher::new();
        self.current_seed.a.hash(&mut hasher);
        self.current_seed.b.hash(&mut hasher);
        self.iteration_count.hash(&mut hasher);
        let state_hash = hasher.finish();

        // 2. 播种 RNG (Seed PRNG)
        let mut rng = StdRng::seed_from_u64(state_hash);

        // 3. 选择扰动 (Will's Choice)
        let idx = rng.gen_range(0..gen_count);
        let perturbation = &self.generators[idx];

        let candidate = self.current_seed.compose(perturbation);
        
        (candidate, idx)
    }

    /// 接受 (Accept)：确认这一步有效，更新状态并通知 Tracer
    /// 
    /// [Decoupling]: 这里引入泛型 T: WillTracer。
    /// - 如果传入 SilentTracer，编译器会优化掉调用，零开销。
    /// - 如果传入 ProvenTracer，则记录轨迹。
    pub fn accept<T: WillTracer>(&mut self, tracer: &mut T, new_state: ClassGroupElement, generator_idx: usize) {
        self.current_seed = new_state;
        self.iteration_count += 1;
        
        // 通知外部观察者
        tracer.on_accept(generator_idx);
    }
    
    /// 拒绝 (Reject)：通知 Tracer 并更新计数器
    pub fn reject<T: WillTracer>(&mut self, tracer: &mut T) {
        self.iteration_count += 1;
        tracer.on_reject();
    }
}
