// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::{Integer, complete::Complete};
use blake3::Hasher;

pub struct SystemParameters {
    pub discriminant: Integer,
}

impl SystemParameters {
    /// 运行时生成：根据随机种子生成判别式 Delta
    /// [SECURITY FIX]: Added loop limit to prevent infinite hang during setup.
    pub fn from_random_seed(seed_bytes: &[u8], bit_size: u32) -> Self {
        println!("[System] Generating Trustless Parameters from seed...");
        
        let mut attempt = 0;
        let max_attempts = 10_000; // 防止无限死循环

        loop {
            if attempt > max_attempts {
                panic!("❌ Failed to generate System Parameters. Seed entropy insufficient or bad luck.");
            }

            // 1. 确定性地不断改变 Hash 输入 (Nonce)
            let mut hasher = Hasher::new();
            hasher.update(seed_bytes);
            hasher.update(&attempt.to_le_bytes());
            let hash_output = hasher.finalize();

            // 2. 将 Hash 扩展为大整数
            let mut candidate = Integer::from_digits(hash_output.as_bytes(), rug::integer::Order::Lsf);
            
            // 确保位数足够
            candidate.set_bit(bit_size - 1, true);
            
            // 3. 强制 M = 3 mod 4 (为了让 Delta = 1 mod 4)
            let rem = candidate.mod_u(4);
            if rem != 3 {
                attempt += 1;
                continue;
            }

            // 4. 素性测试 (Miller-Rabin)
            if candidate.is_probably_prime(30) != rug::integer::IsPrime::No {
                let discriminant = -candidate;
                println!("[System] Found Discriminant: {}", discriminant);
                return SystemParameters { discriminant };
            }

            attempt += 1;
        }
    }
}
