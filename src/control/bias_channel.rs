use rand::prelude::*;
use rand_distr::{Distribution, Normal};
use std::f64::consts::PI;

/// -------------------------------------------------------------------
/// THEORY PATCH IMPLEMENTATION:
/// Subspace Reachability & Matrix Rotation
///
/// Problem: Bias vector (dim=16) cannot control Logits (dim=32000) deterministically.
/// Solution: Dynamic Basis Rotation (J-L Lemma Application).
///
/// If optimization stalls (local minimum in the current subspace),
/// we verify the "Blind Spot Hypothesis" and rotate the projection matrix
/// to scan a new slice of the high-dimensional manifold.
/// -------------------------------------------------------------------

const BIAS_DIM: usize = 16;
// In a real scenario, this matches the LLM vocabulary size.
// For the demo, we assume a projected embedding space or a simplified vocab.
const EMBEDDING_DIM: usize = 128; 

/// The algebraic control signal sitting on the Torus.
#[derive(Clone, Debug)]
pub struct BiasVector {
    pub components: [f64; BIAS_DIM],
}

impl BiasVector {
    pub fn new_zero() -> Self {
        BiasVector {
            components: [0.0; BIAS_DIM],
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let mut comps = [0.0; BIAS_DIM];
        for i in 0..BIAS_DIM {
            comps[i] = rng.gen_range(-1.0..1.0);
        }
        BiasVector { components: comps }
    }

    /// Perturbs the vector locally (Fine-tuning within current subspace)
    pub fn perturb(&self, intensity: f64) -> Self {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();
        let mut new_comps = self.components;
        
        // Select a random dimension to tweak (Coordinate Descent style)
        let idx = rng.gen_range(0..BIAS_DIM);
        new_comps[idx] += normal.sample(&mut rng) * intensity;
        
        // Tanh activation to keep it bounded (soft-clipping)
        new_comps[idx] = new_comps[idx].tanh();

        BiasVector {
            components: new_comps,
        }
    }
}

/// The Projector bridging the Control Space (16D) and Semantic Space (128D+).
/// Represents 'W_proj' in the theory documents.
#[derive(Clone)]
struct ProjectionMatrix {
    // Dimensions: EMBEDDING_DIM x BIAS_DIM
    weights: Vec<Vec<f64>>,
}

impl ProjectionMatrix {
    /// Initialize a random orthogonal-ish projection matrix.
    /// Uses Gaussian initialization to satisfy J-L Lemma properties.
    fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, (1.0 / BIAS_DIM as f64).sqrt()).unwrap(); // Xavier-like init

        let weights = (0..EMBEDDING_DIM)
            .map(|_| {
                (0..BIAS_DIM)
                    .map(|_| normal.sample(&mut rng))
                    .collect()
            })
            .collect();

        ProjectionMatrix { weights }
    }

    /// Projects the low-dim bias into the high-dim logit space.
    /// z = W * b
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

/// The main controller that runs VAPO (Valuation-Adaptive Perturbation Optimization).
pub struct BiasChannel {
    current_bias: BiasVector,
    projector: ProjectionMatrix,
    temperature: f64,
    rotation_count: usize,
}

impl BiasChannel {
    pub fn new() -> Self {
        println!("🔧 [BiasChannel] Initializing Control Manifold (Dim: {} -> {})", BIAS_DIM, EMBEDDING_DIM);
        BiasChannel {
            current_bias: BiasVector::new_zero(),
            projector: ProjectionMatrix::new_random(),
            temperature: 1.0,
            rotation_count: 0,
        }
    }

    /// THEORY PATCH IMPLEMENTATION: Matrix Rotation
    /// Rotates the basis to cover a new random subspace of the logit manifold.
    /// This resolves the "Subspace Reachability" issue probabilistically.
    pub fn rotate_basis(&mut self) {
        self.rotation_count += 1;
        println!("🔄 [VAPO] Stagnation detected. Triggering MATRIX ROTATION (Sequence #{})", self.rotation_count);
        println!("   -> Scanning new subspace...");
        
        // Re-initialize W_proj
        self.projector = ProjectionMatrix::new_random();
        
        // Reset bias to zero relative to the new basis to avoid shock
        self.current_bias = BiasVector::new_zero();
    }

    /// Projects the current control state to an additive logit mask.
    pub fn get_logit_bias(&self) -> Vec<f64> {
        self.projector.project(&self.current_bias)
    }

    /// The Core VAPO Loop (Mock Implementation)
    /// optimizing J(b) = Energy(STP(Generator(b)))
    pub fn optimize<F>(&mut self, mut energy_evaluator: F) -> f64
    where
        F: FnMut(&Vec<f64>) -> f64, // Callback: Returns Energy given a Logit Bias
    {
        let max_steps = 50;
        let stagnation_limit = 10;
        let mut best_energy = f64::MAX;
        let mut stagnation_counter = 0;

        println!("🛡️ [VAPO] Starting discrete optimization loop...");

        for step in 0..max_steps {
            // 1. Generate Candidate (Perturbation in current subspace)
            let candidate_bias = self.current_bias.perturb(0.5 * self.temperature);
            let candidate_logits = self.projector.project(&candidate_bias);

            // 2. Evaluate Energy (Oracle Call to STP Engine)
            let energy = energy_evaluator(&candidate_logits);

            // 3. Selection Logic (Greedy + Simulated Annealing)
            if energy < best_energy {
                best_energy = energy;
                self.current_bias = candidate_bias;
                stagnation_counter = 0; // Reset stagnation
                
                // println!("   -> [Step {}] Improvement! Energy: {:.4}", step, best_energy);
                
                if best_energy <= 1e-6 {
                    println!("✨ [VAPO] Convergence achieved at Step {}.", step);
                    return 0.0;
                }
            } else {
                stagnation_counter += 1;
            }

            // 4. CHECK FOR TRAPS (Blind Spot Detection)
            if stagnation_counter >= stagnation_limit {
                // The optimizer is stuck. 
                // Hypothesis: The solution is orthogonal to the current projection subspace.
                // Action: Rotate the Matrix.
                self.rotate_basis();
                stagnation_counter = 0;
                
                // Heat up temperature to escape local wells
                self.temperature = 1.5; 
            }

            // Decay temperature
            self.temperature *= 0.95;
        }

        println!("⚠️ [VAPO] Loop finished. Final Energy: {:.4} (Rotations: {})", best_energy, self.rotation_count);
        best_energy
    }
}

// ------------------------------------------------------------------
// Unit Tests ensuring the Rotation Logic works
// ------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_consistency() {
        let channel = BiasChannel::new();
        let vec = BiasVector::new_zero();
        let proj = channel.projector.project(&vec);
        assert_eq!(proj.len(), EMBEDDING_DIM);
        assert_eq!(proj[0], 0.0); // Zero bias should project to zero
    }

    #[test]
    fn test_rotation_resets_basis() {
        let mut channel = BiasChannel::new();
        let original_weights = channel.projector.weights.clone();
        
        channel.rotate_basis();
        
        let new_weights = channel.projector.weights.clone();
        assert_ne!(original_weights[0][0], new_weights[0][0]);
    }
}
