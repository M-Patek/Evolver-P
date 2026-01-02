//! The Soul Module
//! 
//! Defines the Ontology of the Evolver system.
//! 
//! # The Ontological Amendment
//! We have transitioned from the Commutative Ideal Class Group Cl(D)
//! to the Non-Commutative Arithmetic Lattice of Definite Quaternions B_{p, \infty}.
//! 
//! This structure supports:
//! 1. Intrinsic Causality (Order of operations matters).
//! 2. Ramanujan Graph Spectral Gap (Optimal search mixing).

pub mod algebra;
pub mod dynamics;

// Re-export core types for easy access
pub use algebra::{IdealClass, Quaternion};
pub use dynamics::{TimeEvolution, IdentityDynamics, HeckeDynamics, VDFDynamics};
