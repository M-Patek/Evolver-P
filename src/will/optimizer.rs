use crate::soul::algebra::IdealClass;
use crate::will::evaluator::Evaluator;
use crate::will::perturber::Perturber;
use crate::will::tracer::OptimizationTrace;

/// VAPO: Valuation-Adaptive Perturbation Optimization.
/// 
/// 现在它是一个“有记忆”的搜索者，会生成可验证的 Trace。
pub struct VapoOptimizer {
    evaluator: Box<dyn Evaluator>,
    max_steps: usize,
    perturbation_count: usize,
}

impl VapoOptimizer {
    pub fn new(evaluator: Box<dyn Evaluator>, max_steps: usize) -> Self {
        Self {
            evaluator,
            max_steps,
            perturbation_count: 32,
        }
    }

    /// 执行可验证的搜索 (Search with Proof)
    /// 
    /// 返回:
    /// 1. 最优状态 (用于生成最终逻辑)
    /// 2. 优化轨迹 (用于生成 Proof Bundle)
    pub fn search(&self, start_seed: &IdealClass) -> (IdealClass, OptimizationTrace) {
        let mut current_state = start_seed.clone();
        let mut current_energy = self.evaluator.evaluate(&current_state);
        
        // 初始化 Trace
        let mut trace = OptimizationTrace::new(start_seed.clone());

        // 0. 如果初始状态即完美
        if current_energy.abs() < 1e-6 {
            trace.finalize(current_energy);
            return (current_state, trace);
        }

        // 1. 初始化扰动器
        let discriminant = start_seed.discriminant();
        let perturber = Perturber::new(&discriminant, self.perturbation_count);

        // 2. 离散梯度下降
        for _step in 0..self.max_steps {
            // 这里 VAPO 需要做一个权衡：
            // 我们在寻找最优解的过程中会尝试很多路径，
            // 但 Trace 只记录 **最终被采纳** 的那条有效路径。
            // 这是一个 "贪婪路径" 的记录。
            
            // A. Generate Neighbors & Select Best
            // 这是一个局部搜索，我们要在所有邻居中选最好的一个跳过去
            
            let mut best_neighbor = None;
            let mut best_perturbation = None;
            let mut min_local_energy = current_energy;

            // 尝试所有生成元 (这是一个简单的 Steepest Descent)
            // 也可以用随机采样 (Stochastic Hill Climbing)
            for _ in 0..10 { // 采样 10 次作为示例
                 let (candidate, perturbation) = perturber.perturb_with_source(&current_state);
                 let energy = self.evaluator.evaluate(&candidate);
                 
                 if energy < min_local_energy {
                     min_local_energy = energy;
                     best_neighbor = Some(candidate);
                     best_perturbation = Some(perturbation);
                 }
            }

            // B. Transition & Record
            if let (Some(next_state), Some(op)) = (best_neighbor, best_perturbation) {
                // 只有真正移动了，才记录进 Trace
                trace.record_step(op);
                
                current_state = next_state;
                current_energy = min_local_energy;

                // Found Truth?
                if current_energy.abs() < 1e-6 {
                    break;
                }
            } else {
                // Local Optima reached (or bad luck sampling)
                // 在更复杂的实现中，这里会触发 Metropolis 准则接受坏解
                // 或者增大扰动半径 (Valuation Adaptation)
            }
        }

        trace.finalize(current_energy);
        (current_state, trace)
    }
}
