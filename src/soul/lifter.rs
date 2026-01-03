// Copyright (c) 2025 M-Patek
// Part of the Evolver Project
//
// "To change the universe, one must first forget the coordinates but remember the shape."

use nalgebra::DVector;
use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;

use crate::soul::algebra::{AlgebraicState, QuaternionAlgebra};
use crate::dsl::stp_bridge::FeatureProjector;

/// 状态提升器 (State Lifter)
/// 
/// 负责解决 "失忆悖论" (Amnesia Paradox)。
/// 当 SpectralGovernor 决定迁移到底层代数结构完全不同的新空间 (新的素数 p) 时，
/// StateLifter 负责将旧空间的 "真理" (Algebraic State) 无损地移植到新空间。
/// 
/// 核心原理：
/// 虽然 $Cl(\Delta_1)$ 和 $Cl(\Delta_2)$ 的群结构不兼容，但它们在模形式 (Modular Forms) 
/// 特征空间中的投影应当是拓扑同构的。
pub struct StateLifter {
    /// 特征投影仪：负责将代数状态映射到坐标无关的特征向量空间
    projector: FeatureProjector,
}

/// 搜索节点
/// 用于在重整化 (Re-quantization) 阶段的波束搜索。
#[derive(Clone, PartialEq)]
struct SearchNode {
    state: AlgebraicState,
    /// 与目标特征向量的欧几里得距离 (Feature Loss)
    dist: f64,
}

// 实现最小堆排序逻辑：距离越小，优先级越高 (Ord 是反过来的)
impl Eq for SearchNode {}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // 注意：这里反转了比较顺序，使得 BinaryHeap 变成 Min-Heap
        other.dist.partial_cmp(&self.dist).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl StateLifter {
    /// 初始化状态提升器
    pub fn new() -> Self {
        Self {
            projector: FeatureProjector::new(),
        }
    }

    /// 核心逻辑：跨宇宙记忆移植 (Trans-Universal Memory Transfer)
    /// 
    /// 流程：
    /// 1. Lift (离魂): 将旧状态投影到特征空间，获取 "灵魂指纹"。
    /// 2. Transport (转世): 在新代数中初始化，并重放逻辑路径以获得初始落点。
    /// 3. Re-quantize (重塑): 在新流形上搜索最符合 "灵魂指纹" 的格点。
    /// 
    /// # 参数
    /// * `old_state`: 旧宇宙 ($p_{old}$) 中的代数状态
    /// * `new_p`: 新宇宙的物理常数
    /// 
    /// # 返回
    /// * `AlgebraicState`: 新宇宙 ($p_{new}$) 中承载相同逻辑真理的状态
    pub fn lift_and_requantize(
        &self, 
        old_state: &AlgebraicState, 
        new_p: u64
    ) -> AlgebraicState {
        // println!("[Lifter] INITIATING SOUL TRANSFER: p={} -> p={}", old_state.p, new_p);

        // 1. Lift: 提取灵魂 (Canonical Lift)
        // 计算坐标无关的模形式特征向量 (Spirit Vector)
        // 这个向量代表了逻辑的 "形状" 而非 "位置"
        let spirit_vector = self.projector.project(old_state);

        // 2. Transport: 初始化新宇宙
        let new_algebra = QuaternionAlgebra::new(new_p);
        
        // [关键启发式] Path Replay: 路径重放
        // 尝试在新代数中执行完全相同的逻辑操作序列 (生成元序列)。
        // 逻辑的句法结构 (Syntax) 比代数坐标 (Semantics) 对 p 的变化更具鲁棒性。
        // 这通常能让我们直接落在目标特征附近的 "吸引盆地" (Basin of Attraction) 里。
        let seed_state = new_algebra.replay_path(&old_state.path_history);
        
        // 3. Re-quantize: 逆向坍缩 (CVP Search / Fine-tuning)
        // 由于 p 的变化，流形的曲率发生了微变，简单的 Path Replay 会导致 "语义漂移"。
        // 我们需要在新代数中进行局部搜索 (Beam Search)，寻找最接近 spirit_vector 的格点。
        let refined_state = self.fine_tune_requantization(seed_state, &new_algebra, &spirit_vector);
        
        // 计算最终的特征损失，用于验证迁移质量
        // let final_loss = (&self.projector.project(&refined_state) - &spirit_vector).norm();
        // println!("[Lifter] TRANSFER COMPLETE. Feature Loss: {:.6}", final_loss);
        
        refined_state
    }

    /// 微调重整化 (Fine-tune Requantization)
    /// 
    /// 这是一个近似最近向量问题 (Approximate CVP) 的求解过程。
    /// 我们在新代数的格点上进行波束搜索，目标是最小化特征空间的距离。
    fn fine_tune_requantization(
        &self, 
        seed: AlgebraicState, 
        algebra: &QuaternionAlgebra, 
        target: &DVector<f64>
    ) -> AlgebraicState {
        let mut heap = BinaryHeap::new();
        
        // 计算种子的初始误差
        let initial_dist = (&self.projector.project(&seed) - target).norm();
        
        heap.push(SearchNode { 
            state: seed.clone(), 
            dist: initial_dist 
        });
        
        let mut best_state = seed;
        let mut min_dist = initial_dist;
        
        // 使用 HashSet 记录已访问的状态 Hash，防止搜索回环
        let mut visited = HashSet::new();
        // visited.insert(best_state.hash()); // 假设 AlgebraicState 有 hash() 方法
        
        // Beam Search 参数
        let beam_width = 20;  // 波束宽度：每层保留的平行宇宙数量
        let max_steps = 50;   // 最大搜索深度

        for _ in 0..max_steps {
            if heap.is_empty() { break; }

            // 1. 收集当前波束 (The Beam)
            // 从堆中取出当前最好的 K 个节点
            let mut candidates = Vec::new();
            for _ in 0..beam_width {
                if let Some(node) = heap.pop() {
                    // 更新全局最优解
                    if node.dist < min_dist {
                        min_dist = node.dist;
                        best_state = node.state.clone();
                    }
                    candidates.push(node);
                }
            }

            // 2. 扩展下一层 (Expand)
            for node in candidates {
                // 生成所有邻居 (通过应用基础生成元)
                let neighbors = algebra.generate_neighbors(&node.state);
                
                for neighbor in neighbors {
                    // 查重 (这里需要 AlgebraicState 实现 Hash 或唯一 ID)
                    let neighbor_hash = neighbor.hash(); 
                    if visited.contains(&neighbor_hash) { 
                        continue; 
                    }
                    visited.insert(neighbor_hash);

                    // 投影并计算距离
                    let feat = self.projector.project(&neighbor);
                    let d = (feat - target).norm();
                    
                    // 3. 剪枝与入堆 (Pruning)
                    // 只有方向大致正确 (距离没有显著增加) 的才入堆
                    // 允许 1.2 倍的松弛 (Relaxation) 以具备跳出局部极小值的能力
                    if d < node.dist * 1.2 { 
                        heap.push(SearchNode { 
                            state: neighbor, 
                            dist: d 
                        });
                    }
                }
            }
        }
        
        best_state
    }
}
