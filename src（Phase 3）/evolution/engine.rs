// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::evolution::gene::{ProbeGene, ProbeState};
use crate::phase3::evolution::mutagen::MutagenOracle;
use crate::phase3::core::neuron::HTPNeuron;
use crate::phase3::core::affine::AffineTuple;
use crate::phase3::core::algebra::ClassGroupElement;
use crate::phase3::decoder::InverseDecoder;

use std::collections::{BinaryHeap, HashSet};
use std::sync::{Arc, RwLock};
use rug::Integer;
// use rayon::prelude::*; // 建议在 Cargo.toml 中开启 rayon 以支持并行下坠

/// 🦖 EvolutionaryEngine: 达尔文引擎 (Fractal Mesh Search Core)
/// 实现了主人设计的“无限递归网探针”逻辑：
/// 1. 撒网 (Scatter)
/// 2. 撞墙 (Collision)
/// 3. 幸存者裂变 (Fission)
pub struct EvolutionaryEngine {
    /// [Environment]: 代数环境 (提供墙和法则)
    neuron_core: Arc<RwLock<HTPNeuron>>,
    
    /// [Navigator]: 用于判断是否到达真理 (坐标解码器)
    decoder: InverseDecoder,
    
    /// [Mutagen]: 诱变剂来源 (Transformer 探针)
    mutagen: Box<dyn MutagenOracle>,

    /// [Time Machine]: 优先队列 (支持时光回溯)
    /// 存储了所有“存活但暂未处理”的探针状态。
    /// 这是一个大根堆，始终优先处理适应度最高的探针。
    search_queue: BinaryHeap<ProbeState>,

    /// [History]: 已探索空间 (Tabu Search)
    /// 防止进化在同一个局部最优的死胡同里打转。
    visited_hashes: HashSet<u64>,

    /// [Parameters]: 进化参数
    precision_target: f64, // 目标精度 (epsilon)
    max_generations: usize, // 最大迭代次数 (防止无限递归)
}

impl EvolutionaryEngine {
    pub fn new(
        neuron: Arc<RwLock<HTPNeuron>>, 
        vocab_size: u32,
        mutagen: Box<dyn MutagenOracle>
    ) -> Self {
        EvolutionaryEngine {
            neuron_core: neuron,
            decoder: InverseDecoder::new(vocab_size),
            mutagen,
            search_queue: BinaryHeap::new(),
            visited_hashes: HashSet::new(),
            precision_target: 0.0, // 0 drift = Absolute Truth
            max_generations: 1000,
        }
    }

    /// 🌪️ 主要进化循环：寻找真理
    /// 这就是“递归网”的主循环。
    pub fn evolve_until_optimality(&mut self, initial_state: AffineTuple) -> Result<ProbeGene, String> {
        // 1. 初始化始祖探针 (Adam & Eve) - 撒下第一张网
        self.seed_population(initial_state);

        let mut generation = 0;

        // while let (自动时光回溯): 如果当前最优探针撞墙了，下一次循环会自动拿出次优探针
        while let Some(parent_state) = self.search_queue.pop() {
            
            // [Termination]: 超时熔断
            if generation > self.max_generations {
                return Err("Evolution Timeout: Fractal mesh exhausted without convergence.".to_string());
            }

            let parent_gene = &parent_state.gene;

            // 2. [Validation]: 撞墙检测 (The Wall)
            // 检查当前探针是否不仅“活着”，而且“活得好”(Drift 小)
            match self.decoder.decode(&parent_gene.current_state) {
                Ok(result) => {
                    // [Goal Check]: 是否达到绝对真理 (Drift <= Target)
                    if (result.drift as f64) <= self.precision_target {
                        println!("🏆 Truth Found! Generation: {}, Logic Depth: {}", generation, parent_gene.depth);
                        return Ok(parent_gene.clone());
                    }
                    
                    // 虽然没到终点，但没撞墙，可以作为裂变的种子
                },
                Err(_) => {
                    // [Collision]: 撞墙了 (Navigation Lost)
                    // 探针死亡。由于这是循环的开始，continue 意味着放弃该分支，
                    // 也就是自动回溯到优先队列中的下一个“备胎”。
                    continue; 
                }
            }

            // 3. [Reproduction]: 裂变 (Fission)
            // 基于幸存者，产生下一代探针云 (Recursive Mesh)
            let offspring = self.spawn_offspring(parent_gene);

            // 4. [Selection]: 评估子代并入队
            for child in offspring {
                // 在入队前先做一次轻量级评估，如果生下来就是死的，就不入队
                if let Some(scored_child) = self.evaluate_fitness(child) {
                    self.search_queue.push(scored_child);
                }
            }

            generation += 1;
        }

        Err("Extinction: All probes collided with logic walls. No solution found.".to_string())
    }

    /// 🌱 播种：生成初始探针群
    fn seed_population(&mut self, initial_state: AffineTuple) {
        // 初始探针是一个“空白”个体
        let seed = ProbeGene {
            p_weight: Integer::from(1), // Identity
            bias_vector: vec![0; 4],
            depth: 0,
            current_state: initial_state,
        };
        
        // 立即进行一次宏观裂变，撒出第一层网
        let first_gen = self.spawn_offspring(&seed);
        for child in first_gen {
             if let Some(scored) = self.evaluate_fitness(child) {
                 self.search_queue.push(scored);
             }
        }
    }

    /// 🧬 繁殖：生成子代 (包含三种突变策略)
    /// 这对应了“幸存者裂变变成多个探针”的过程。
    fn spawn_offspring(&self, parent: &ProbeGene) -> Vec<ProbeGene> {
        let mut offspring = Vec::new();

        // A. [Micro-Mutation]: 适应性辐射 (Focus)
        // 沿用父亲的 P_weight，微调 Bias。这是“下窄”的过程，精度收缩。
        // 我们请求 5 个微调建议
        let micro_suggestions = self.mutagen.suggest_micro_mutations(parent, 5);
        for bias in micro_suggestions {
            offspring.push(ProbeGene {
                p_weight: parent.p_weight.clone(),
                bias_vector: bias,
                depth: parent.depth + 1,
                current_state: parent.current_state.clone(), // 状态会在 evaluate_fitness 中更新
            });
        }

        // B. [Macro-Mutation]: 超时空跳跃 (Explore)
        // 改变 P_weight，跳出局部最优。这是 Transformer 的“直觉引导”。
        // 如果父亲的适应度已经很高，我们减少这种突变；如果低，增加这种突变。
        let macro_suggestions = self.mutagen.suggest_macro_mutations(parent, 2);
        for p in macro_suggestions {
            offspring.push(ProbeGene {
                p_weight: p,
                bias_vector: parent.bias_vector.clone(), // 继承 Bias
                depth: parent.depth + 1,
                current_state: parent.current_state.clone(),
            });
        }

        // C. [Entropy Injection]: 熵注入 (Chaos)
        // 产生完全随机的“疯子探针”，对应“叛逆者”策略。
        // 这是理论上保证遍历性的关键。
        let random_p = crate::phase3::core::primes::hash_to_prime("entropy_mutant", 64).unwrap_or(Integer::from(3));
        offspring.push(ProbeGene {
            p_weight: random_p,
            bias_vector: vec![0; 4], // Reset Bias
            depth: parent.depth + 1,
            current_state: parent.current_state.clone(),
        });

        offspring
    }

    /// ⚖️ 评估适应度 & 执行演化步
    /// 这里是“生存还是毁灭”的判决点 (Collision Check)。
    fn evaluate_fitness(&self, mut gene: ProbeGene) -> Option<ProbeState> {
        let neuron_guard = self.neuron_core.read().ok()?;

        // 1. [Trial Run]: 试运行一步演化
        // 模拟：应用 P_weight 和 Bias
        // 注意：这里我们简化处理，假设 Bias 影响 Q，P_weight 影响 P
        
        let p_op = AffineTuple {
            p_factor: gene.p_weight.clone(),
            // 真实的 Bias 逻辑应更复杂，这里仅作演示：Bias 影响 Shift
            q_shift: ClassGroupElement::identity(&neuron_guard.discriminant), 
        };
        
        // [Soft Wall]: 尝试合成。如果 P-Factor 溢出，compose 会返回 Err
        let new_state_res = gene.current_state.compose(&p_op, &neuron_guard.discriminant);
        
        match new_state_res {
            Ok(new_state) => {
                gene.current_state = new_state;
            },
            Err(_) => return None, // [Hard Wall]: 撞墙死亡 (溢出熔断)
        }

        // 2. [Drift Check]: 计算曼哈顿漂移 (Navigation)
        // 漂移越小，适应度越高
        let decode_res = self.decoder.decode(&gene.current_state);
        
        let drift = if let Ok(res) = decode_res {
            res.drift
        } else {
            return None; // [Soft Wall]: 迷路死亡 (Navigation Lost)
        };

        // 3. [Fitness Formula]: 适应度公式
        // Fitness = (1.0 / (1 + drift)) + (Depth * 0.1)
        // 我们不仅想要 drift 小的，也想要能走得深(逻辑链长)的
        let fitness = (1.0 / (1.0 + drift as f64)) + (gene.depth as f64 * 0.1);

        Some(ProbeState {
            gene,
            fitness_score: fitness,
        })
    }
}
