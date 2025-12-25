// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

#[cfg(test)]
mod tests {
    use crate::phase3::core::algebra::ClassGroupElement;
    use crate::phase3::core::affine::AffineTuple;
    // 假设 SystemParameters 在 param 模块中，这里直接模拟环境
    use rug::Integer;

    fn setup_env() -> Integer {
        // 使用一个固定的测试用判别式 (Small, for validation speed)
        // M = 3 mod 4 => Delta = -M = 1 mod 4
        let m = Integer::from(1000003); // Prime, 3 mod 4
        let discriminant = -m;
        discriminant
    }

    #[test]
    fn test_state_streaming_constant_size() {
        let discriminant = setup_env();
        let mut state = ClassGroupElement::identity(&discriminant);
        
        println!("🌊 [Test] Starting State Streaming Evolution...");
        println!("   Initial State Size: {} bits", state.a.significant_bits());

        // 模拟 100 步演化
        // 如果是旧的累积模式，100 步足以让 P 变得巨大
        for i in 0..100 {
            // 构造随机算子 (p, q)
            // 这里的 p 模拟 Token Prime
            let p = Integer::from(1009); 
            let q = ClassGroupElement::generator(&discriminant); // 模拟 Shift
            
            // Apply: S_new = S_old^p * q
            // 关键点：这里 p 被立即消耗掉了，不参与后续累积
            state = state.apply_affine(&p, &q, &discriminant).unwrap();
            
            if i % 20 == 0 {
                let size = state.a.significant_bits();
                println!("   Step {}: State Size = {} bits (Should remain const)", i, size);
                
                // 断言：状态大小不应超过判别式的位宽太多 (Class Group 元素的紧凑性)
                // 实际上归约后的元素大小由判别式决定
                assert!(size < discriminant.significant_bits() + 100);
            }
        }
        println!("✅ State Streaming test passed. No explosion detected.");
    }

    #[test]
    #[should_panic(expected = "Security Halt")]
    fn test_legacy_accumulation_overflow() {
        let discriminant = setup_env();
        let mut accumulator = AffineTuple::identity(&discriminant);
        
        println!("💥 [Test] Testing Legacy Accumulation Fuse...");

        // 模拟旧模式：不断累积 P (试图构造全局 AffineTuple)
        // 每次 P 增加 ~10 bits，循环 1000 次将达到 10000 bits，超过 8192 限制
        for _ in 0..1000 {
            let p = Integer::from(1009); 
            let q = ClassGroupElement::identity(&discriminant);
            let op = AffineTuple { p_factor: p, q_shift: q };
            
            // 这里会因为 P 因子爆炸而触发 Panic
            // 这证明了我们的安全熔断机制是生效的
            accumulator = accumulator.compose(&op, &discriminant).unwrap();
        }
    }
}
