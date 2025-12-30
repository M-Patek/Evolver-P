// Copyright (C) 2025 M-Patek. All Rights Reserved.
// Integrated from HTP Core (Hyper-Tensor Protocol)

pub mod algebra;
pub mod primes;

// Re-export core types for easier access
pub use algebra::ClassGroupElement;
pub use primes::hash_to_prime;
