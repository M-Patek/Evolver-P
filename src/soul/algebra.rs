use sha2::{Digest, Sha256};
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

// ============================================================================
// Constants defining the Definite Quaternion Algebra B_{p, \infty}
// We choose p = 37 (a safe prime) for this implementation.
// Algebra: i^2 = A, j^2 = B, ij = k, ji = -k
// Parameters: A = -1, B = -p
// ============================================================================

const ALGEBRA_P: i64 = 37;
const PARAM_A: i64 = -1;
const PARAM_B: i64 = -ALGEBRA_P;

/// A Quaternion q = a + bi + cj + dk in the algebra B_{p, \infty}.
/// This is the atomic "word" of our causal language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Quaternion {
    pub a: i64, // Scalar part
    pub b: i64, // i coeff
    pub c: i64, // j coeff
    pub d: i64, // k coeff
}

impl Quaternion {
    pub fn new(a: i64, b: i64, c: i64, d: i64) -> Self {
        Self { a, b, c, d }
    }

    pub fn zero() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub fn identity() -> Self {
        Self::new(1, 0, 0, 0)
    }

    /// The reduced norm: N(q) = a^2 - A*b^2 - B*c^2 + A*B*d^2
    /// Note: Since A<0 and B<0, this is a positive definite quadratic form.
    pub fn norm(&self) -> i128 {
        let a = self.a as i128;
        let b = self.b as i128;
        let c = self.c as i128;
        let d = self.d as i128;

        let term1 = a * a;
        let term2 = -(PARAM_A as i128) * b * b;
        let term3 = -(PARAM_B as i128) * c * c;
        let term4 = (PARAM_A as i128) * (PARAM_B as i128) * d * d;

        term1 + term2 + term3 + term4
    }

    /// Quaternion Conjugate: q_bar = a - bi - cj - dk
    pub fn conjugate(&self) -> Self {
        Self::new(self.a, -self.b, -self.c, -self.d)
    }
}

// ----------------------------------------------------------------------------
// Operator Overloading for Non-Commutative Arithmetic
// ----------------------------------------------------------------------------

impl Add for Quaternion {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(
            self.a + other.a,
            self.b + other.b,
            self.c + other.c,
            self.d + other.d,
        )
    }
}

impl Sub for Quaternion {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(
            self.a - other.a,
            self.b - other.b,
            self.c - other.c,
            self.d - other.d,
        )
    }
}

impl Mul for Quaternion {
    type Output = Self;

    /// Non-commutative multiplication in B_{p, \infty}
    /// (a1 + b1i + c1j + d1k)(a2 + b2i + c2j + d2k)
    /// Using multiplication table:
    /// i^2 = A, j^2 = B, k^2 = -AB
    /// ij = k, ji = -k
    /// jk = -Bi, kj = Bi
    /// ki = -Aj, ik = Aj
    fn mul(self, rhs: Self) -> Self {
        let a1 = self.a; let b1 = self.b; let c1 = self.c; let d1 = self.d;
        let a2 = rhs.a; let b2 = rhs.b; let c2 = rhs.c; let d2 = rhs.d;

        let A = PARAM_A;
        let B = PARAM_B;

        // Real part
        let ra = a1*a2 + A*b1*b2 + B*c1*c2 - A*B*d1*d2;
        
        // i part
        let rb = a1*b2 + b1*a2 - B*c1*d2 + B*d1*c2;

        // j part
        let rc = a1*c2 + A*b1*d2 + c1*a2 - A*d1*b2;

        // k part
        let rd = a1*d2 - b1*c2 + c1*b2 + d1*a2;

        Self::new(ra, rb, rc, rd)
    }
}

impl fmt::Display for Quaternion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}, {}]", self.a, self.b, self.c, self.d)
    }
}

// ============================================================================
// The Soul: Arithmetic Lattice State
// Replaces the old IdealClass (Binary Quadratic Form)
// ============================================================================

/// Represents a node in the Pizer Graph (Ramanujan Graph).
/// Physically, it is a Right Ideal in the Maximal Order of the Quaternion Algebra.
/// Simplification: We represent the state by the accumulated path of quaternions
/// acting on the origin.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdealClass {
    /// The current value of the accumulator quaternion.
    /// This represents the "Position" in the non-commutative lattice.
    pub value: Quaternion,
    
    /// The 'context' prime p used for seeding (kept for compatibility).
    pub discriminator: u64,
}

impl IdealClass {
    /// Creates a new Identity State (The Origin).
    pub fn identity(discriminator: u64) -> Self {
        Self {
            value: Quaternion::identity(),
            discriminator,
        }
    }

    /// Seeds the Soul from a linguistic context.
    /// Hashes the context to generate 4 integers (a, b, c, d) to form the initial Quaternion.
    pub fn from_hash(context: &str, discriminator: u64) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(context.as_bytes());
        let result = hasher.finalize();

        // Slice the hash into 4 i64s
        let bytes = result.as_slice();
        let a = i64::from_be_bytes(bytes[0..8].try_into().unwrap_or([0; 8])) % 1000;
        let b = i64::from_be_bytes(bytes[8..16].try_into().unwrap_or([0; 8])) % 1000;
        let c = i64::from_be_bytes(bytes[16..24].try_into().unwrap_or([0; 8])) % 1000;
        let d = i64::from_be_bytes(bytes[24..32].try_into().unwrap_or([0; 8])) % 1000;

        // Ensure we start with a non-zero quaternion
        let q = if a == 0 && b == 0 && c == 0 && d == 0 {
            Quaternion::identity()
        } else {
            Quaternion::new(a, b, c, d)
        };

        Self {
            value: q,
            discriminator,
        }
    }

    /// Apply a Hecke Operator action (Right Multiplication by a Generator).
    /// This is the fundamental mechanism of Causality.
    /// S_next = S_current * G
    pub fn apply_hecke(&self, generator: &Quaternion) -> Self {
        // Non-commutative state transition
        let new_value = self.value * (*generator);
        
        // Note: In a full implementation, we would perform lattice reduction here 
        // (Right Ideal normalization) to keep coefficients small.
        // For this verified-search version, we allow the coefficients to grow 
        // as they carry the full path history (The Trace).
        
        Self {
            value: new_value,
            discriminator: self.discriminator,
        }
    }

    /// Generates a set of "Hecke Neighbors" (The Spectral Gap guarantee).
    /// Returns a list of valid moves from the current state.
    pub fn neighbors(&self) -> Vec<Self> {
        // In the Pizer graph for p=37, we look for elements of norm p.
        // Hardcoded simplified generators for B_{37, \infty}
        // These are quaternions with Norm = ALGEBRA_P (37).
        // Since i^2 = -1, j^2 = -37, k^2 = -37.
        // Norm = a^2 + b^2 + 37c^2 + 37d^2
        
        let mut moves = Vec::new();
        
        // 1. Trivial generators (if they exist in this algebra structure)
        // For demonstration, we use a deterministic set of small perturbations
        // that represent the 'directions' in the Cayley graph.
        
        // Generator 1: 6^2 + 1^2 = 37. (a=6, b=1, c=0, d=0) -> Norm = 36 + 1 = 37.
        moves.push(self.apply_hecke(&Quaternion::new(6, 1, 0, 0)));
        moves.push(self.apply_hecke(&Quaternion::new(6, -1, 0, 0)));
        
        // Generator 2: 1^2 + 6^2 = 37.
        moves.push(self.apply_hecke(&Quaternion::new(1, 6, 0, 0)));
        moves.push(self.apply_hecke(&Quaternion::new(1, -6, 0, 0)));

        // In a real Pizer graph, there are p+1 neighbors for T_p.
        // We include "Identity-like" perturbations for fine-tuning.
        moves.push(self.apply_hecke(&Quaternion::new(1, 0, 0, 0))); // Self-loop (Stay)

        moves
    }
}
