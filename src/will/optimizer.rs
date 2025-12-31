use num_bigint::BigInt;
use num_traits::{Zero, Signed};
use crate::soul::algebra::ClassGroupElement;
use crate::will::perturber::{self, EnergyEvaluator};

/// VAPO (Valuation-Adaptive Perturbation Optimization) 核心循环
///
/// 该函数执行在代数 Cayley 图上的离散局部搜索 (Discrete Local Search on Cayley Graph)。
/// 
/// # 数学修正
/// - 这里的 "State Space" 是理想类群 Cl(Delta) 的 Cayley 图。
/// - "Perturbations" 实际上是图的生成元集合 (Generators) \mathcal{P}。
/// - 优化过程不再是流形上的梯度下降，而是图上的贪婪游走 (Greedy Walk)。
/// 
/// # 逻辑流程
/// 1. 从 `start_state` 提取判别式 $\Delta$，确定所在的图组件。
/// 2. 生成生成元集合 $\{\epsilon_i\}$ (即 Edge Set)。
/// 3. 进入图搜索循环：
///    - **动态邻域调度 (Dynamic Neighborhood Schedule)**: 
///      根据当前的搜索进度，动态调整使用的生成元子集。
///      初期使用 "Coarse" 生成元进行大步幅跳跃（在投影空间中造成剧烈变化）。
///      后期使用 "Fine" 生成元进行局部调整。
///    - 扩展当前节点的所有邻居。
///    - 计算邻居的 STP 能量。
///    - 贪婪地移动到能量最低的邻居节点。
/// 4. 如果发现能量 $E=0$ 的节点，搜索结束。
pub fn optimize(
    start_state: &ClassGroupElement,
    evaluator: &impl EnergyEvaluator
) -> ClassGroupElement {
    // 1. 自动提取判别式
    let four = BigInt::from(4);
    let delta = (&start_state.b * &start_state.b) - (&four * &start_state.a * &start_state.c);

    // 2. 准备生成元集 (The Generator Set / Perturbation Set)
    // 这些对应于 Cayley 图中与当前节点相连的边
    let perturbation_count = 50;
    let perturbations = perturber::generate_perturbations(&delta, perturbation_count);

    let mut current_state = start_state.clone();
    let mut current_energy = evaluate_state(&current_state, evaluator);
    
    if current_energy.abs() < 1e-6 {
        return current_state;
    }

    let max_iterations = 100;

    // 3. 搜索循环 (The Graph Walk)
    for _iter in 0..max_iterations {
        // [Task 3.4] 生成元子集调度 (Generator Subset Scheduling)
        // -------------------------------------------------------------
        // 原理：并非所有生成元都是平等的。
        // 在投影映射 \Psi 下，某些生成元会导致逻辑路径剧烈变化（Coarse），
        // 而另一些只会导致微小的叶节点翻转（Fine）。
        // 我们通过收缩生成元窗口来模拟 "从粗到细" 的搜索。
        let progress = _iter as f64 / max_iterations as f64;
        
        // 窗口收缩：随着迭代，减少可用的生成元数量，
        // 强迫搜索集中在 "Fine" 生成元（通常是列表前部的那些小素数）上。
        let window_ratio = 1.0 - (0.9 * progress); 
        let active_count = (perturbations.len() as f64 * window_ratio).ceil() as usize;
        let active_count = active_count.max(3).min(perturbations.len());
        
        let active_perturbations = &perturbations[0..active_count];
        // -------------------------------------------------------------

        let mut best_neighbor = current_state.clone();
        let mut min_energy = current_energy;
        let mut found_better = false;

        // 遍历邻域 (Explore Neighborhood)
        for eps in active_perturbations {
            // 正向边: S' = S * eps
            let neighbor_pos = current_state.compose(eps);
            let energy_pos = evaluate_state(&neighbor_pos, evaluator);

            if energy_pos < min_energy {
                min_energy = energy_pos;
                best_neighbor = neighbor_pos;
                found_better = true;
            }

            // 逆向边: S' = S * eps^-1
            // Cayley 图是无向的（或双向的），我们需要检查逆元方向
            let inverse_eps = eps.inverse();
            let neighbor_neg = current_state.compose(&inverse_eps);
            let energy_neg = evaluate_state(&neighbor_neg, evaluator);

            if energy_neg < min_energy {
                min_energy = energy_neg;
                best_neighbor = neighbor_neg; 
                found_better = true;
            }
        }

        // 决策时刻
        if min_energy.abs() < 1e-6 {
            return best_neighbor;
        }

        if found_better {
            // 移动到更好的邻居
            current_state = best_neighbor;
            current_energy = min_energy;
        } else {
            // 陷入局部最优 (Local Minima)
            // 在图搜索中，这通常意味着我们所在的盆地没有向下的出口。
            // 实际工程中可能需要重启 (Restart) 或模拟退火 (accept bad moves)，
            // 这里暂且保持简单的贪婪停止。
            break;
        }
    }

    current_state
}

fn evaluate_state(state: &ClassGroupElement, evaluator: &impl EnergyEvaluator) -> f64 {
    let path = materialize_path(state);
    evaluator.evaluate(&path)
}

fn materialize_path(state: &ClassGroupElement) -> Vec<u64> {
    let mut digits = Vec::new();
    let extract_u64 = |n: &BigInt| -> u64 {
        let (_sign, bytes) = n.to_bytes_le();
        if bytes.is_empty() {
            0
        } else {
            let mut buf = [0u8; 8];
            let len = std::cmp::min(bytes.len(), 8);
            buf[..len].copy_from_slice(&bytes[..len]);
            u64::from_le_bytes(buf)
        }
    };

    digits.push(extract_u64(&state.a));
    digits.push(extract_u64(&state.b));
    digits.push(extract_u64(&state.c));

    digits
}
