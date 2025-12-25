// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::Integer;
use blake3::Hasher;

/// 🛡️ Hash-to-Prime Map (With Safety Fallback)
/// 将任意字符串确定性地映射为一个大素数。
/// 
/// # 算法改进
/// 1. **Phase 1 (Probabilistic)**: 尝试 `max_attempts` 次 Nonce 变换，寻找符合哈希分布的“完美素数”。
/// 2. **Phase 2 (Deterministic)**: 如果运气极差（Lucky Number Deadlock），切换到“扫描模式”。
///    从一个确定性的种子开始，线性向后搜索最近的素数 (`next_prime`)。
///    这保证了**可用性 (Availability)** 为 100%。
pub fn hash_to_prime(user_id: &str, bit_size: u32) -> Result<Integer, String> {
    let mut nonce = 0u64;
    
    // [Config]: 优先尝试保持哈希均匀分布的次数
    // 1000 次尝试覆盖了绝大多数情况 (99.9999%+)
    let optimal_search_limit = 1000; 
    
    // --- Phase 1: 概率性哈希试探 (The "Good" Distribution) ---
    while nonce < optimal_search_limit {
        let mut hasher = Hasher::new();
        // [SECURITY FIX]: 长度前缀，防止哈希拼接攻击
        hasher.update(&(user_id.len() as u64).to_le_bytes());
        hasher.update(user_id.as_bytes());
        hasher.update(&nonce.to_le_bytes());
        let hash = hasher.finalize();

        let mut candidate = Integer::from_digits(hash.as_bytes(), rug::integer::Order::Lsf);
        // 强制设置最高位和最低位，确保位宽和奇数性质
        candidate.set_bit(bit_size - 1, true);
        candidate.set_bit(0, true);

        // 快速筛选：排除明显被 3 或 5 整除的数 (小素数筛)
        if candidate.mod_u(3) == 0 || candidate.mod_u(5) == 0 {
            nonce += 1;
            continue;
        }

        // Miller-Rabin 素性测试
        if candidate.is_probably_prime(25) != rug::integer::IsPrime::No {
            return Ok(candidate);
        }

        nonce += 1;
    }
    
    // --- Phase 2: 确定性保底扫描 (The "Safe" Fallback) ---
    // 如果程序运行到这里，说明该 user_id 是个数学上的“倒霉蛋”。
    // 我们不再随机哈希，而是使用确定性扫描找到最近的素数。
    // 这解决了 "Lucky Number Deadlock" 问题。
    
    // 1. 生成一个用于 Fallback 的基准种子 (Domain Separation)
    let mut hasher = Hasher::new();
    hasher.update(b"HTP_PRIME_FALLBACK_V1::");
    hasher.update(user_id.as_bytes());
    let hash = hasher.finalize();
    
    let mut fallback_candidate = Integer::from_digits(hash.as_bytes(), rug::integer::Order::Lsf);
    fallback_candidate.set_bit(bit_size - 1, true);
    fallback_candidate.set_bit(0, true);

    // 2. 使用 GMP/Rug 的优化算法寻找“下一个素数”
    // next_prime() 是确定性的，且根据黎曼猜想，素数间隙不会太大，必定能找到。
    fallback_candidate.next_prime_mut();

    // [Optional]: 记录警告日志，以便监控这种罕见情况
    // 在生产环境中，可以将此日志级别设为 Warn 或 Info
    // println!("⚠️ [Primes] Warning: '{}' triggered fallback scan. (Entropy Exhaustion)", user_id);

    Ok(fallback_candidate)
}
