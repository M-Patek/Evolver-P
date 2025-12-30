use rand::prelude::*;
use rand_distr::{Distribution, Normal};
use rand::rngs::StdRng; // Use Standard RNG for seeding
// use std::f64::consts::PI; 
use crate::dsl::schema::{ProofAction, ProofBundle};
use crate::dsl::stp_bridge::STPContext;

/// -------------------------------------------------------------------
/// BIAS CHANNEL v0.2 (Deterministic Edition)
/// 
/// Security Patch: "Verifiable Binding"
/// All randomness (Initialization, Projection, Perturbation) is now
/// derived from a master `seed`. This allows a remote Verifier to 
/// reconstruct the exact same Projection Matrix W_proj and verify
/// the search trajectory.
/// -------------------------------------------------------------------

const BIAS_DIM: usize = 16;
const EMBEDDING_DIM: usize = 128; 

/// The algebraic control signal.
#[derive(Clone, Debug)]
pub struct BiasVector {
    pub components: [f64; BIAS_DIM],
}

impl BiasVector {
    pub fn new_zero() -> Self {
        BiasVector { components: [0.0; BIAS_DIM] }
    }

    /// Perturbs the vector locally using a deterministic RNG.
    pub fn perturb(&self, rng: &mut StdRng, intensity: f64) -> Self {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let mut new_comps = self.components;
        
        // Select a random dimension to tweak
        let idx = rng.gen_range(0..BIAS_DIM);
        new_comps[idx] += normal.sample(rng) * intensity;
        
        // Tanh activation (soft-clipping)
        new_comps[idx] = new_comps[idx].tanh();

        BiasVector { components: new_comps }
    }
}

/// The Projector bridging Control Space -> Semantic Space.
/// MUST be deterministic based on the seed.
#[derive(Clone)]
struct ProjectionMatrix {
    weights: Vec<Vec<f64>>,
}

impl ProjectionMatrix {
    /// Initialize matrix using a specific seed.
    /// This ensures W_proj is identical on Server and Verifier.
    fn new_from_seed(seed: u64) -> Self {
        // We use the seed + a constant to separate Matrix gen from other randomness
        let mut rng = StdRng::seed_from_u64(seed + 0xDEADBEEF); 
        let normal = Normal::new(0.0, (1.0 / BIAS_DIM as f64).sqrt()).unwrap();

        let weights = (0..EMBEDDING_DIM)
            .map(|_| {
                (0..BIAS_DIM)
                    .map(|_| normal.sample(&mut rng))
                    .collect()
            })
            .collect();

        ProjectionMatrix { weights }
    }

    fn project(&self, bias: &BiasVector) -> Vec<f64> {
        let mut output = vec![0.0; EMBEDDING_DIM];
        for i in 0..EMBEDDING_DIM {
            let mut sum = 0.0;
            for j in 0..BIAS_DIM {
                sum += self.weights[i][j] * bias.components[j];
            }
            output[i] = sum;
        }
        output
    }
}

pub struct VapoConfig {
    pub max_iterations: usize,
    pub initial_temperature: f64,
    pub valuation_decay: f64,
}

/// The main controller that runs VAPO.
pub struct BiasController {
    config: VapoConfig,
    // We no longer store persistent state that carries over between Contexts
    // to ensure pure determinism.
}

impl BiasController {
    pub fn new(config: Option<VapoConfig>) -> Self {
        BiasController {
            config: config.unwrap_or(VapoConfig {
                max_iterations: 50,
                initial_temperature: 1.0,
                valuation_decay: 0.95,
            }),
        }
    }

    /// The Core Optimization Loop (Now strictly bound to a Seed/Context)
    pub fn optimize<F>(
        &self,
        context_str: &str, // The "Prompt"
        seed: u64,         // The "Commitment"
        raw_logits: &[f64], // Simulated output from Generator(seed, context)
        stp_ctx: &mut STPContext, 
        decode_fn: F
    ) -> ProofBundle
    where
        F: Fn(&[f64]) -> ProofAction,
    {
        println!("🛡️ [VAPO] Init: Seed={}, ContextHash={}", seed, md5::compute(context_str));

        // 1. Deterministic Initialization
        let mut rng = StdRng::seed_from_u64(seed);
        let mut projector = ProjectionMatrix::new_from_seed(seed);
        let mut current_bias = BiasVector::new_zero();
        
        let mut temperature = self.config.initial_temperature;
        let mut best_energy = f64::MAX;
        let mut best_bias = current_bias.clone();
        let mut best_action = decode_fn(raw_logits); // Initial guess

        // 2. Optimization Loop
        for step in 0..self.config.max_iterations {
            // Generate Candidate
            let candidate_bias = current_bias.perturb(&mut rng, 0.5 * temperature);
            let bias_logits = projector.project(&candidate_bias);
            
            // Combine: Logits_Final = Logits_Raw + Logits_Bias
            // Note: In real world, dimensions match. Here we assume logic handles mapping.
            // For mock: we just look at the bias effect or assume raw_logits is adaptable.
            let mut mixed_logits = raw_logits.to_vec();
            for i in 0..mixed_logits.len().min(bias_logits.len()) {
                mixed_logits[i] += bias_logits[i]; 
            }

            // Decode & Check Energy
            let candidate_action = decode_fn(&mixed_logits);
            let energy = stp_ctx.calculate_energy(&candidate_action);

            // Selection Logic
            if energy < best_energy {
                best_energy = energy;
                current_bias = candidate_bias.clone();
                best_bias = current_bias.clone();
                best_action = candidate_action.clone();

                if best_energy <= 1e-6 {
                    println!("✨ [VAPO] Convergence at Step {}. Energy=0.0", step);
                    break;
                }
            } else {
                // Stochastic Accept (Metropolis-like) could go here
            }

            // Blind Spot Rotation Logic (Simulated)
            // If stuck, we can rotate the projector. 
            // Crucial: The rotation must ALSO be deterministic based on the RNG state!
            if step % 15 == 14 && best_energy > 0.1 {
                println!("🔄 [VAPO] Rotating Basis (Deterministic)...");
                // Re-init projector with current RNG state (which flows from seed)
                // In this mock, we just scramble weights deterministically
                let new_sub_seed = rng.next_u64(); 
                projector = ProjectionMatrix::new_from_seed(new_sub_seed);
                current_bias = BiasVector::new_zero(); // Reset bias
            }

            temperature *= self.config.valuation_decay;
        }

        // 3. Construct the Proof Bundle
        ProofBundle {
            bias_vector: best_bias.components.to_vec(),
            action: best_action,
            energy_signature: best_energy,
            context_hash: format!("{:x}", md5::compute(context_str)),
            generator_seed: seed,
        }
    }
}
