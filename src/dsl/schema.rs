use serde::{Deserialize, Serialize};

/// Represents a rigorous mathematical action in the HTP system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofAction {
    Define {
        symbol: String,
        hierarchy_path: Vec<String>,
    },
    Apply {
        theorem_id: String,
        inputs: Vec<String>,
        output_symbol: String,
    },
}

/// The Holy Grail of the Evolver system.
/// This bundle contains everything needed for a skeptical Verifier
/// to accept a piece of neuro-symbolic logic as "Truth".
///
/// UPDATED v0.2: Includes Context Binding and Seed Commitment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofBundle {
    /// Layer 0: The discrete control signal found by VAPO.
    pub bias_vector: Vec<f64>,
    
    /// Layer 1: The resulting logical action (The "Claim").
    pub action: ProofAction,
    
    /// Layer 2: STP Algebraic Proof (Mocked as Energy signature for now).
    /// In full impl, this would be the Merkle Path.
    pub energy_signature: f64,

    /// SECURITY UPDATE (v0.2): Context Integrity
    /// SHA256 Hash of the input prompt/context.
    /// Prevents "Context Splicing" attacks.
    pub context_hash: String,

    /// SECURITY UPDATE (v0.2): Generator Determinism
    /// The random seed used to initialize the Generator (and the Projection Matrix).
    /// Allows the Verifier to replay the "Chaos" and verify the "Order".
    pub generator_seed: u64,
}

impl ProofBundle {
    /// A lightweight check to see if this bundle belongs to the given context.
    pub fn verify_binding(&self, current_context_str: &str) -> bool {
        // In a real implementation, use sha2::Sha256
        // Here we use a mock hash for demonstration
        let calculated_hash = format!("{:x}", md5::compute(current_context_str));
        
        if self.context_hash != calculated_hash {
            println!("❌ [Security] Context Mismatch! Bundle bound to {}, but current is {}.", 
                self.context_hash, calculated_hash);
            return false;
        }
        true
    }
}
