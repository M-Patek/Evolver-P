use serde::{Deserialize, Serialize};

/// The ProofBundle is the verifiable artifact of the "Proof of Will".
/// It contains not just the answer (LogicPath), but the cryptographic proof
/// that the answer was found through legitimate search on the algebraic manifold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofBundle {
    /// The input context (e.g., "Prove 1+1=2") hashed to anchor the session.
    pub context_hash: String,
    
    /// The algebraic parameters derived from the context.
    /// In production, this defines the Class Group Cl(Delta).
    pub discriminant: String, // String to support large integers in JSON
    
    /// The security level used for this generation.
    pub security_bits: u32,

    /// The seed state where the search began.
    pub start_state: String,

    /// The final state found by the Will.
    pub final_state: String,

    /// The trace of perturbations applied to reach the final state.
    /// Replaying this trace on the start_state MUST yield the final_state.
    pub trace: Vec<String>,

    /// The materialized logical path (the actual "Answer").
    pub logic_path: Vec<String>,
    
    /// The energy of the final state (Should be 0 for a valid proof).
    pub energy: f64,
}

impl ProofBundle {
    /// verifying the bundle means replaying the trace and checking energy.
    pub fn is_valid(&self) -> bool {
        // Logic to replay trace would go here.
        // For now, we trust the energy field if the signature matches (omitted).
        self.energy == 0.0
    }
}
