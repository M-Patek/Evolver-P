// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.
// Ported from HTP Core for Evolver Integration

use rug::Integer;
use blake3::Hasher;

/// Generates a cryptographically secure prime number deterministically from a user ID.
/// Uses a nonce-based search with Miller-Rabin primality testing.
pub fn hash_to_prime(user_id: &str, bit_size: u32) -> Result<Integer, String> {
    let mut nonce = 0u64;
    let max_attempts = 500; // Prevent infinite loops
    
    while nonce < max_attempts {
        let mut hasher = Hasher::new();
        // Domain separation and length prefixing to prevent canonicalization attacks
        hasher.update(b"HTP_HashToPrime_v1");
        hasher.update(&(user_id.len() as u64).to_le_bytes());
        hasher.update(user_id.as_bytes());
        hasher.update(&nonce.to_le_bytes());
        let hash = hasher.finalize();

        // Convert hash to integer
        let mut candidate = Integer::from_digits(hash.as_bytes(), rug::integer::Order::Lsf);
        
        // Ensure bit length and oddity
        candidate.set_bit(bit_size - 1, true);
        candidate.set_bit(0, true);

        // Small prime sieve (optimization)
        if candidate.mod_u(3) == 0 || candidate.mod_u(5) == 0 {
            nonce += 1;
            continue;
        }

        // Miller-Rabin test (25 rounds)
        if candidate.is_probably_prime(25) != rug::integer::IsPrime::No {
            return Ok(candidate);
        }

        nonce += 1;
    }
    
    Err(format!("❌ Failed to generate prime for identifier '{}' after {} attempts.", user_id, max_attempts))
}
