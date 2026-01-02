use crate::body::projection::Projector;

/// The Adapter converts the raw algebraic projection into semantic ProofActions.
/// 
/// [Security Audit Fixed]: 
/// Previously used `%` operator which allows negative remainders in Rust (e.g., -5 % 2 == -1).
/// This caused negative odd numbers to bypass "IsOdd" checks or fall into undefined states.
/// Now strictly uses `.rem_euclid()` to ensure results are always in [0, modulus).

#[derive(Debug, Clone, PartialEq)]
pub enum ProofAction {
    Assert(String),
    Transform(String),
    Check(String),
    NoOp,
}

pub struct Adapter {
    modulus: u64,
}

impl Adapter {
    pub fn new(modulus: u64) -> Self {
        Self { modulus }
    }

    /// Converts a raw digit from the geometric projection into a logical action.
    pub fn adapt(&self, raw_val: i64, context_seed: u64) -> ProofAction {
        // [FIX] CRITICAL: Use Euclidean Remainder.
        // The raw_val comes from the Class Group hash, which can be interpreted as signed.
        // We must guarantee a positive index for the dispatch table.
        let op_code = raw_val.rem_euclid(4); 

        match op_code {
            0 => {
                // Parity Check Branch
                // Using context_seed to mix entropy
                let check_val = (raw_val + context_seed as i64).rem_euclid(2);
                if check_val == 0 {
                    ProofAction::Assert("IsEven".to_string())
                } else {
                    ProofAction::Assert("IsOdd".to_string())
                }
            },
            1 => {
                // Transformation Branch
                // Maps to Identity or Inc based on simple modulo
                let trans_type = raw_val.rem_euclid(2);
                match trans_type {
                    0 => ProofAction::Transform("Identity".to_string()),
                    _ => ProofAction::Transform("Increment".to_string()),
                }
            },
            2 => {
                // Type Consistency Check
                ProofAction::Check("TypeMatch".to_string())
            },
            _ => ProofAction::NoOp,
        }
    }

    /// Helper validation for numeric properties
    pub fn is_structurally_even(val: i64) -> bool {
        // [FIX] Also applied here for internal consistency
        val.rem_euclid(2) == 0
    }
}
