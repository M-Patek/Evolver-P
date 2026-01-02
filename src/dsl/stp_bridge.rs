use ndarray::{Array1, Array2};
use std::collections::HashMap;

/// StpBridge: The Semantic Auditor
/// 
/// Responsible for calculating the structural and semantic energy of a state.
/// [Update]: Now includes `calculate_axiom_penalty` to enforce static mathematical truths.

pub struct StpContext {
    // Maps semantic concepts (e.g., "Even", "Odd") to their index in the state vector
    pub concept_map: HashMap<String, usize>,
    
    // Cached indices for fast penalty calculation
    idx_even: usize,
    idx_odd: usize,
    idx_prime: usize,
}

impl StpContext {
    pub fn new(concepts: Vec<String>) -> Self {
        let mut map = HashMap::new();
        for (i, c) in concepts.iter().enumerate() {
            map.insert(c.clone(), i);
        }

        // Pre-fetch indices for Axiom Logic
        // In a real system, these would be dynamic or configured via config file
        let idx_even = *map.get("Even").unwrap_or(&0); // Default to 0 if missing (unsafe in prod)
        let idx_odd = *map.get("Odd").unwrap_or(&1);
        let idx_prime = *map.get("Prime").unwrap_or(&2);

        Self {
            concept_map: map,
            idx_even,
            idx_odd,
            idx_prime,
        }
    }

    /// Calculates the Total Logical Energy
    /// J(S) = E_syntax + E_stp_struct + E_axiom
    pub fn calculate_energy(&self, state_vec: &Array1<f64>, syntax_valid: bool) -> f64 {
        if !syntax_valid {
            return 10.0; // Syntax Error Barrier
        }

        let struct_energy = self.calculate_structure_violation(state_vec);
        let axiom_energy = self.calculate_axiom_penalty(state_vec);

        // If logical contradiction exists, return massive energy
        // otherwise return 0.0 (Truth)
        if struct_energy > 0.0 || axiom_energy > 0.0 {
            return 100.0 + struct_energy + axiom_energy;
        }

        0.0
    }

    /// Checks if matrix dimensions match (Traditional STP check)
    fn calculate_structure_violation(&self, _state_vec: &Array1<f64>) -> f64 {
        // ... (Existing logic for dimension checking) ...
        0.0 // Placeholder
    }

    /// [NEW FEATURE] Axiom Injection
    /// Penalizes states that are structurally valid but semantically absurd.
    /// Energy += X^T * M_axiom * X (simplified here to element-wise ops)
    pub fn calculate_axiom_penalty(&self, state_vec: &Array1<f64>) -> f64 {
        let mut penalty = 0.0;
        
        // Axiom 1: Mutual Exclusion (互斥律)
        // A number cannot be strongly Even AND strongly Odd.
        // P(Even) * P(Odd) should be 0.
        let p_even = state_vec[self.idx_even];
        let p_odd = state_vec[self.idx_odd];
        
        // Penalty is proportional to the product of conflicting probabilities.
        // Multiplied by a large constant to create a steep gradient (energy cliff).
        if p_even > 0.1 && p_odd > 0.1 {
            penalty += p_even * p_odd * 10_000.0;
        }

        // Axiom 2: Primes > 2 are Odd (素数蕴含奇数律)
        // P(Prime) * P(Even) should be 0 (ignoring 2 for this simplified model).
        let p_prime = state_vec[self.idx_prime];
        if p_prime > 0.1 && p_even > 0.1 {
            penalty += p_prime * p_even * 1_000.0;
        }

        // Axiom 3: Identity Law (同一律 violation detection)
        // If state represents A != A, panic.
        // (Implementation omitted for brevity)

        penalty
    }
}
