// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::Integer;
use blake3::Hasher;

pub fn hash_to_prime(user_id: &str, bit_size: u32) -> Result<Integer, String> {
    let mut nonce = 0u64;
    // [SECURITY FIX]: 降低尝试次数，防止 素数搜索 CPU DoS
    let max_attempts = 500; 
    
    while nonce < max_attempts {
        let mut hasher = Hasher::new();
        // [SECURITY FIX]: 增加长度前缀，防止 Canonicalization (哈希拼接) 攻击
        hasher.update(&(user_id.len() as u64).to_le_bytes());
        hasher.update(user_id.as_bytes());
        hasher.update(&nonce.to_le_bytes());
        let hash = hasher.finalize();

        let mut candidate = Integer::from_digits(hash.as_bytes(), rug::integer::Order::Lsf);
        candidate.set_bit(bit_size - 1, true);
        candidate.set_bit(0, true);

        if candidate.mod_u(3) == 0 || candidate.mod_u(5) == 0 {
            nonce += 1;
            continue;
        }

        if candidate.is_probably_prime(25) != rug::integer::IsPrime::No {
            return Ok(candidate);
        }

        nonce += 1;
    }
    
    Err(format!("❌ Failed to generate prime for user after {} attempts.", max_attempts))
}
