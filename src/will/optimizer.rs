use crate::will::evaluator::Evaluator;
use crate::will::perturber::Perturber; // [Modified] Perturber is now a concrete struct or trait we use directly
use crate::will::ricci::RicciFlow;    // [New] Import Ricci Flow
use crate::soul::algebra::IdealClass;

/// Valuation-Adaptive Perturbation Optimizer (VAPO)
/// 
/// Updated with Topological Amendment (v2.3):
/// Now traverses the Conformal Manifold (V, d_Ricci) instead of the raw Cayley Graph.
pub struct VapoOptimizer {
    evaluator: Box<dyn Evaluator>,
    perturber: Perturber, // [Explicit] We need direct access to perturber for lookahead
    search_steps: usize,
    
    // [New] The Geometry Engine
    ricci_engine: RicciFlow,
}

impl VapoOptimizer {
    pub fn new(evaluator: Box<dyn Evaluator>, search_steps: usize, p: u64) -> Self {
        // Initialize Perturber with standard generating set (assumed logic)
        let perturber = Perturber::new_standard(p);
        
        Self {
            evaluator,
            perturber,
            search_steps,
            // Sensitivity = 2.0 意味着对负曲率非常敏感
            ricci_engine: RicciFlow::new(2.0, p), 
        }
    }

    /// 执行搜索 (The Walk of Will)
    /// 
    /// Returns: (Best State, Trace Path)
    pub fn search(&self, seed: &IdealClass) -> (IdealClass, Vec<String>) {
        let mut current_state = seed.clone();
        let mut current_energy = self.evaluator.evaluate(&current_state);
        let mut trace = Vec::new();

        println!("Starting VAPO Search on Ricci-Flowed Manifold...");
        println!("Initial Energy: {:.4}", current_energy);

        for step in 0..self.search_steps {
            if current_energy.abs() < 1e-6 {
                println!("Convergence reached at step {}", step);
                break;
            }

            // 1. 获取候选邻居 (Perturbations)
            let candidates = self.perturber.generate_candidates(&current_state);
            
            // 2. 评估候选者 (Evaluation Loop)
            // 这里我们不仅计算能量 E，还计算有效梯度 \nabla_{eff}
            let mut best_candidate = None;
            let mut min_effective_energy = f64::MAX;
            let mut selected_move_name = String::new();

            for (move_name, candidate_state) in candidates {
                // A. 原始能量 (Truth)
                let raw_energy = self.evaluator.evaluate(&candidate_state);
                
                // B. 曲率计算 (Stability)
                // 计算从 current 到 candidate 这条边的曲率
                let kappa = self.ricci_engine.calculate_curvature(
                    &current_state, 
                    &candidate_state, 
                    &self.perturber
                );

                // C. 度量修正 (The Amendment)
                // E_eff = E_raw + Penalty(\kappa)
                let curvature_penalty = self.ricci_engine.compute_penalty(kappa);
                let effective_energy = raw_energy + curvature_penalty;

                // Debug log for critical decisions
                if step % 10 == 0 && effective_energy < current_energy {
                    println!("  Option [{}]: E={:.4}, k={:.4}, E_eff={:.4}", 
                        move_name, raw_energy, kappa, effective_energy);
                }

                if effective_energy < min_effective_energy {
                    min_effective_energy = effective_energy;
                    best_candidate = Some(candidate_state);
                    selected_move_name = move_name;
                }
            }

            // 3. 状态更新 (Transition)
            // 只有当有效能量下降时才移动 (Hill Climbing on Ricci Manifold)
            // 注意：这里我们比较的是 effective_energy，即使 raw_energy 降低了，
            // 如果曲率是极负的（陷阱），effective_energy 可能会很高，从而阻止移动。
            if let Some(next_state) = best_candidate {
                // 计算当前状态的有效能量作为基准（近似）
                let current_kappa_est = 0.0; // 原地不动曲率为0
                let current_eff = current_energy + self.ricci_engine.compute_penalty(current_kappa_est);

                if min_effective_energy < current_eff {
                    current_state = next_state;
                    // 更新 current_energy 为真实的原始能量，用于下一次迭代判断收敛
                    current_energy = self.evaluator.evaluate(&current_state);
                    trace.push(selected_move_name.clone());
                } else {
                    // Local Minima on Ricci Manifold
                    // 可能需要模拟退火或随机重启 (未实现)
                    break;
                }
            } else {
                break;
            }
        }

        (current_state, trace)
    }
}
