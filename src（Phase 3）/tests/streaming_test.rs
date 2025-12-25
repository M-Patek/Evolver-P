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
        // 选用稍大一点的素数以确保群阶足够大，避免小阶元素干扰测试
        let m = Integer::from(1000003); // Prime, 3 mod 4
        let discriminant = -m;
        discriminant
    }

    /// 🛡️ [NEW TEST]: 严格代数性质检查
    /// 专门用于捕捉非单位元运算中的逻辑缺陷
    #[test]
    fn test_strict_algebraic_properties() {
        let discriminant = setup_env();
        let identity = ClassGroupElement::identity(&discriminant);
        
        println!("🧪 [Test] Starting Strict Algebraic Property Checks...");

        // 1. 获取非单位元生成元 (Non-Identity Generator)
        let g = ClassGroupElement::generator(&discriminant);
        assert_ne!(g, identity, "FATAL: Generator must not be identity!");
        println!("   [1/5] Generator retrieved: Non-Identity ✅");

        // 2. Square Safety Check
        // 确保 g.square() 不会因为 reduce 逻辑错误而 Panic 或返回非法值
        let g_sq = g.square(&discriminant).expect("Squaring failed");
        assert_ne!(g_sq, g, "g^2 should not equal g (unless order is 1, which is forbidden)");
        println!("   [2/5] Squaring safety check passed ✅");
        
        // 3. Power Consistency Check
        // g^1 == g
        let p1 = g.pow(&Integer::from(1), &discriminant).expect("Pow(1) failed");
        assert_eq!(p1, g, "g.pow(1) != g");

        // g^2 == g.compose(g)
        let p2 = g.pow(&Integer::from(2), &discriminant).expect("Pow(2) failed");
        let g_comp_g = g.compose(&g, &discriminant).expect("Compose failed");
        assert_eq!(p2, g_comp_g, "g.pow(2) != g.compose(g) -> Logic inconsistency detected!");
        println!("   [3/5] Power consistency check passed ✅");

        // 4. Associativity Check (结合律)
        // (x * y) * z == x * (y * z)
        // 这是群论的基础，如果 compose 实现有误（如 reduce 不规范），结合律通常会首先崩坏
        let x = g.clone();
        // 构造另外两个“伪独立”元素用于测试
        let y = g.pow(&Integer::from(5), &discriminant).unwrap();
        let z = g.pow(&Integer::from(11), &discriminant).unwrap();

        let xy = x.compose(&y, &discriminant).unwrap();
        let xy_z = xy.compose(&z, &discriminant).unwrap(); // (x*y)*z

        let yz = y.compose(&z, &discriminant).unwrap();
        let x_yz = x.compose(&yz, &discriminant).unwrap(); // x*(y*z)

        assert_eq!(xy_z, x_yz, "❌ Associativity Violated! (x*y)*z != x*(y*z)");
        println!("   [4/5] Associativity check passed ✅");

        // 5. Inverse Property Check (逆元性质)
        // x * x^-1 == Identity
        // 在类群形式 (a, b, c) 中，逆元是 (a, -b, c)
        let x_inv = ClassGroupElement {
            a: x.a.clone(),
            b: -x.b.clone(),
            c: x.c.clone(),
        };

        let res_right = x.compose(&x_inv, &discriminant).unwrap();
        assert_eq!(res_right, identity, "❌ Right Inverse failed (x * x^-1 != I)");
        
        let res_left = x_inv.compose(&x, &discriminant).unwrap();
        assert_eq!(res_left, identity, "❌ Left Inverse failed (x^-1 * x != I)");
        
        println!("   [5/5] Inverse property check passed ✅");

        println!("✅ Strict algebraic properties verified. The algebraic engine is robust.");
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
            // [FIXED]: 即使在这里，也应该尽可能使用 Generator 避免 Identity 掩盖问题
            // 但为了触发 P 因子爆炸，Q 的值其实不重要，用 Identity 也可以
            let q = ClassGroupElement::identity(&discriminant);
            let op = AffineTuple { p_factor: p, q_shift: q };
            
            // 这里会因为 P 因子爆炸而触发 Panic
            // 这证明了我们的安全熔断机制是生效的
            accumulator = accumulator.compose(&op, &discriminant).unwrap();
        }
    }
}
