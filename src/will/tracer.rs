use serde::{Deserialize, Serialize};
use crate::soul::algebra::{IdealClass, EvolverError};
use crate::will::perturber::Perturber;

pub type Energy = f64;

/// 验证结果
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    Verified { energy: Energy, steps: usize },
    InvalidUniverse { details: String },
    IllegalMove { step: usize, generator: String },
    ContextMismatch { expected_seed: String, actual_seed: String },
    FinalStateMismatch { claimed: String, calculated: String },
    EnergyMismatch { claimed: Energy, calculated: Energy },
    AlgebraicFailure { details: String }, // [New] 处理计算过程中的异常
}

/// 优化轨迹 (Proof of Will Certificate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationTrace {
    pub id: String,
    pub timestamp: u64,
    pub context: String, 
    pub initial_state: IdealClass,
    pub perturbations: Vec<IdealClass>,
    pub final_state: IdealClass,
    pub claimed_energy: Energy,
}

impl OptimizationTrace {
    pub fn new(initial_state: IdealClass, context: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            context,
            initial_state: initial_state.clone(),
            perturbations: Vec::new(),
            final_state: initial_state, 
            claimed_energy: f64::MAX,
        }
    }

    pub fn record_step(&mut self, perturbation: IdealClass) {
        // [Note] 这里是在生成 Proof 阶段，如果出错了 panic 也是合理的（因为是自己的 Bug），
        // 但为了健壮性，我们尽量 unwrap。生产环境应处理 Result。
        if let Ok(new_state) = self.final_state.compose(&perturbation) {
             self.final_state = new_state;
             self.perturbations.push(perturbation);
        } else {
            // 如果记录步骤时出错，我们可以选择忽略或者记录一个错误标记
            // 这里选择不做任何状态更新，相当于这一步没发生
            eprintln!("Warning: Failed to record step due to algebraic error");
        }
    }

    pub fn finalize(&mut self, final_energy: Energy) {
        self.claimed_energy = final_energy;
    }
}

/// 验证器 (The Verifier)
pub struct TraceVerifier;

impl TraceVerifier {
    pub fn verify<E>(
        trace: &OptimizationTrace, 
        energy_fn: E,
        perturbation_count: usize 
    ) -> VerificationResult
    where 
        E: Fn(&IdealClass) -> Energy,
    {
        // --- 1. Anchor Check ---
        let (expected_seed, _) = IdealClass::spawn_universe(&trace.context);
        
        if expected_seed != trace.initial_state {
            return VerificationResult::ContextMismatch {
                expected_seed: format!("{}", expected_seed),
                actual_seed: format!("{}", trace.initial_state),
            };
        }

        // --- 2. Graph Topology Setup ---
        let discriminant = trace.initial_state.discriminant();
        let perturber = Perturber::new(&discriminant, perturbation_count);
        let allowed_generators = perturber.get_generators(); 

        // --- 3. Path Replay (Robust Replay) ---
        let mut calculated_state = trace.initial_state.clone();
        
        for (i, u) in trace.perturbations.iter().enumerate() {
            // A. Check Membership
            let is_valid_generator = allowed_generators.contains(u);
            let is_valid_inverse = if !is_valid_generator {
                let inverse = u.inverse();
                allowed_generators.contains(&inverse)
            } else {
                true
            };

            if !is_valid_generator && !is_valid_inverse {
                return VerificationResult::IllegalMove { 
                    step: i, 
                    generator: format!("{}", u) 
                };
            }

            // B. Execute Algebra (Safely!)
            match calculated_state.compose(u) {
                Ok(new_state) => {
                    calculated_state = new_state;
                },
                Err(EvolverError::CosmicMismatch(s, o)) => {
                    return VerificationResult::InvalidUniverse { 
                        details: format!("Step {}: Mismatch {} vs {}", i, s, o) 
                    };
                },
                Err(e) => {
                    return VerificationResult::AlgebraicFailure {
                        details: format!("Step {}: {}", i, e)
                    };
                }
            }
        }

        // --- 4. Final Consistency Check ---
        if calculated_state != trace.final_state {
            return VerificationResult::FinalStateMismatch {
                claimed: format!("{}", trace.final_state),
                calculated: format!("{}", calculated_state),
            };
        }

        // --- 5. Energy Audit ---
        let calculated_energy = energy_fn(&calculated_state);
        let epsilon = 1e-6;
        if (calculated_energy - trace.claimed_energy).abs() > epsilon {
            return VerificationResult::EnergyMismatch { 
                claimed: trace.claimed_energy, 
                calculated: calculated_energy 
            };
        }

        VerificationResult::Verified { 
            energy: calculated_energy, 
            steps: trace.perturbations.len() 
        }
    }
}
