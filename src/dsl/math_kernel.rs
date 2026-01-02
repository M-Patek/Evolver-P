// src/dsl/math_kernel.rs

use super::schema::{Constraint, LogicMatrix, Predicate, AggregationStrategy};
use std::collections::HashMap;

/// The MathKernel is responsible for evaluating the "Energy" (Truth/Falsity) 
/// of constraints against the current system state.
///
/// In Evolver, Energy = 0.0 means "Logically Sound".
/// Energy > 0.0 means "Contradiction Detected".
pub struct MathKernel {
    // In a full implementation, these would map to algebraic objects in the Class Group.
    // For this kernel representation, we map them to continuous probabilities [0.0, 1.0]
    // or raw numeric values.
    variables: HashMap<String, f64>, 
    
    // Static data collections available in the context (e.g., [2, 3, 5, 7])
    collections: HashMap<String, Vec<f64>>,
}

impl MathKernel {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            collections: HashMap::new(),
        }
    }

    /// Register a variable with a current probability/value.
    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }

    /// Register a collection of values (for aggregators).
    pub fn set_collection(&mut self, name: &str, values: Vec<f64>) {
        self.collections.insert(name.to_string(), values);
    }

    fn get_val(&self, var_name: &str) -> f64 {
        *self.variables.get(var_name).unwrap_or(&0.0)
    }

    /// The core evaluation loop.
    /// Computes the total energy of a constraint given the current state.
    pub fn compute_energy(&self, constraint: &Constraint) -> f64 {
        match constraint {
            // 1. Basic Assertion
            Constraint::Assert(predicate, var_name) => {
                let val = self.get_val(var_name);
                self.evaluate_predicate(predicate, val)
            },

            // 2. Logical Implication (The new Implies operator)
            Constraint::AssertImplies(var_a, var_b) => {
                let prob_a = self.get_val(var_a);
                let prob_b = self.get_val(var_b);
                
                // Delegate to the LogicMatrix schema we defined
                LogicMatrix::implies_energy(prob_a, prob_b)
            },

            // 3. Universal Quantifier (The new Aggregator)
            // "Unrolls" the check over the collection without an SMT solver.
            Constraint::AssertForAll { collection, predicate, strategy } => {
                if let Some(items) = self.collections.get(collection) {
                    // Map every item to its individual violation energy
                    let energies: Vec<f64> = items.iter()
                        .map(|&item| self.evaluate_predicate(predicate, item))
                        .collect();

                    if energies.is_empty() {
                        return 0.0; // Vacuously true for empty set
                    }

                    match strategy {
                        AggregationStrategy::Sum => {
                            // E_total = Sum(E_i)
                            energies.iter().sum()
                        },
                        AggregationStrategy::LogSumExp => {
                            // E_total = LogSumExp(E_i) = max(E) + ln(sum(exp(E_i - max(E))))
                            // This functions as a "Soft Max", focusing the optimizer on the worst violation.
                            let max_e = energies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                            if max_e == f64::NEG_INFINITY { return 0.0; }
                            
                            let sum_exp = energies.iter().map(|e| (e - max_e).exp()).sum::<f64>();
                            max_e + sum_exp.ln()
                        }
                    }
                } else {
                    // Critical Error: Collection not found. Return high barrier energy.
                    100.0 
                }
            }
        }
    }

    /// Helper function to calculate energy for basic predicates.
    fn evaluate_predicate(&self, predicate: &Predicate, val: f64) -> f64 {
        // Note: In the real STP algebra, this would involve checking class group properties.
        // Here we use simplified numeric checks for the logic kernel.
        match predicate {
            Predicate::IsOdd => {
                // Soft penalty: distance from the nearest odd integer
                // 3.0 -> 0.0, 2.0 -> 1.0, 2.1 -> 0.9
                let nearest = val.round();
                if (nearest as i64) % 2 != 0 {
                    (val - nearest).abs() // Close to odd is good
                } else {
                    1.0 - (val - nearest).abs() // Close to even is bad
                }
            },
            Predicate::IsEven => {
                 let nearest = val.round();
                if (nearest as i64) % 2 == 0 {
                    (val - nearest).abs()
                } else {
                    1.0 - (val - nearest).abs()
                }
            },
            Predicate::IsPositive => {
                // ReLU-style penalty: if x > 0, E=0. If x <= 0, E = -x
                if val > 0.0 { 0.0 } else { -val + 0.1 }
            },
            Predicate::IsPrime => {
                // Hard check for demo purposes (Logic doesn't usually do primality tests continuously!)
                // Acknowledging the "SmallPrimes" context from the prompt.
                let i_val = val.round() as i64;
                if is_prime_basic(i_val) { 0.0 } else { 1.0 }
            }
        }
    }
}

// Simple helper for the mock predicate
fn is_prime_basic(n: i64) -> bool {
    if n <= 1 { return false; }
    for i in 2..=(n as f64).sqrt() as i64 {
        if n % i == 0 { return false; }
    }
    true
}
