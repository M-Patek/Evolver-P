// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::affine::AffineTuple;
use crate::topology::tensor::HyperTensor;
use crate::net::wire::HtpResponse; 
use crate::core::algebra::ClassGroupElement; // 需要引用以进行盲化操作
use rug::Integer;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use rand::{Rng, thread_rng}; // [ADDED] 引入随机源用于盲化

/// HTPNeuron: 仿射神经元 (The Processor)
pub struct HTPNeuron {
    pub p_weight: Integer,
    pub memory: Arc<RwLock<HyperTensor>>,
    pub discriminant: Integer,
}

impl HTPNeuron {
    pub fn new(semantic_fingerprint: Integer, dim: usize, side_len: usize, discriminant: Integer) -> Self {
        let tensor = HyperTensor::new(dim, side_len, discriminant.clone());
        HTPNeuron {
            p_weight: semantic_fingerprint,
            memory: Arc::new(RwLock::new(tensor)),
            discriminant,
        }
    }

    /// ⚡ Algebraic Activation (Side-Channel Hardened)
    pub fn activate(
        &self, 
        input_stream: Vec<AffineTuple>, 
        recursion_depth: usize 
    ) -> Result<(AffineTuple, HtpResponse), String> {
        
        // [TIMING PROTECTION]: 启动精密计时器
        let start_time = Instant::now();
        const SECURITY_LATENCY_BUDGET_MS: u64 = 50;

        let mut memory_guard = self.memory.write().map_err(|_| "Lock poisoned")?;
        let mut current_accumulator = AffineTuple::identity(&self.discriminant);
        
        // 1. [Non-Commutative Evolution Loop]
        for (t, tuple) in input_stream.iter().enumerate() {
            // (a) [SECURITY UPGRADE]: Blinding Evolution
            // 使用底数盲化 (Base Blinding) 来防御 DPA/SPA 攻击
            let weighted_tuple = self.evolve_tuple_blinded(tuple, &self.p_weight)?;

            // (b) 时空噪声注入
            let time_noise = self.generate_spacetime_noise(t)?;
            let step_tuple = weighted_tuple.compose(&time_noise, &self.discriminant)?;

            // (c) 爆炸预判与重封
            let current_bits = current_accumulator.p_factor.significant_bits();
            let step_bits = step_tuple.p_factor.significant_bits();
            
            if current_bits + step_bits > 3072 {
                let checkpoint_key = format!("chk:seal:{}", t);
                memory_guard.insert(&checkpoint_key, current_accumulator.clone(), t as u64)?;
                current_accumulator = step_tuple;
            } else {
                current_accumulator = current_accumulator.compose(&step_tuple, &self.discriminant)?;
            }
        }

        let final_t = input_stream.len();
        let final_key = format!("chk:tail:{}", final_t);
        memory_guard.insert(&final_key, current_accumulator, final_t as u64)?;

        // 2. [Fold]: 全息折叠
        let raw_output = memory_guard.calculate_global_root()?;

        // 3. [Reduce]: 代数规约
        let final_output = self.algebraic_reduction(raw_output, recursion_depth)?;

        // 4. [Proof Generation]
        let proof_coord = memory_guard.map_id_to_coord(0); 
        let proof_path = memory_guard.get_segment_tree_path(&proof_coord, 0);
        
        let proof = HtpResponse::ProofBundle {
            request_id: 0,
            primary_path: proof_path,
            orthogonal_anchors: vec![],
            epoch: recursion_depth as u64,
        };

        // [SECURITY FIX]: Busy-Wait Padding (忙等待填充)
        // 只有 'Sleep' 是不够的，因为它会暴露 CPU 的空闲状态 (Low Power State)。
        // 攻击者可以通过功耗突然下降来精确判定计算结束时间。
        // 我们用无意义的数学运算填充剩余时间，保持功耗平稳 (Iso-Power)。
        let elapsed = start_time.elapsed();
        let target_duration = Duration::from_millis(SECURITY_LATENCY_BUDGET_MS);
        
        if elapsed < target_duration {
            self.perform_busy_wait(target_duration - elapsed);
        }

        Ok((final_output, proof))
    }

    /// 🛡️ [SECURITY CORE]: Blinded Evolution
    /// 
    /// 传统的 `base.pow(exponent)` 会导致底层的 GMP 运算路径依赖于 base 的具体数值，
    /// 这容易受到 Cache 侧信道攻击。
    /// 
    /// 这里我们引入随机盲化因子 R：
    /// 1. Generate random R
    /// 2. Compute Blinded Base: T' = T * R
    /// 3. Exponentiate: Res' = (T * R)^W = T^W * R^W
    /// 4. Unblind: Res = Res' * (R^W)^(-1)
    /// 
    /// 这样 GMP 处理的数据 T' 是完全随机的，与真实输入 T 无关。
    fn evolve_tuple_blinded(&self, tuple: &AffineTuple, weight: &Integer) -> Result<AffineTuple, String> {
        let mut rng = thread_rng();
        
        // 1. 生成随机盲化因子 R (使用 Generator 的随机幂次)
        // 使用一个较小的随机指数以减少性能开销，例如 64-bit 随机数
        let blind_exp = Integer::from(rng.gen::<u64>());
        let generator = ClassGroupElement::generator(&self.discriminant);
        let r_blind = generator.pow(&blind_exp, &self.discriminant)?;
        
        // 2. 盲化输入 Q 部分 (P 部分是整数乘法，相对安全，主要保护 Q 的群幂运算)
        let q_blinded = tuple.q_shift.compose(&r_blind, &self.discriminant)?;
        
        // 3. 执行敏感的幂运算 (Exponentiation with Secret Weight)
        // 此时输入是随机化的，功耗特征与原始数据解耦
        let q_prime_blinded = q_blinded.pow(weight, &self.discriminant)?;
        
        // 4. 计算去盲因子: U = (R^W)^(-1)
        // U = R^( -W )
        let neg_weight = -weight.clone();
        // 注意：负指数可以通过求逆元实现。在类群 (a, b, c) 中，逆元是 (a, -b, c)
        // 这里为了通用性，我们计算 R^W 然后求逆
        let r_w = r_blind.pow(weight, &self.discriminant)?;
        let r_w_inv = ClassGroupElement {
            a: r_w.a,
            b: -r_w.b, // Inverse: negate b
            c: r_w.c,
        };
        
        // 5. 去除盲化: Result = Blinded_Result * U
        let new_q = q_prime_blinded.compose(&r_w_inv, &self.discriminant)?;
        
        // P 部分直接计算 (Information leakage on P-multiplication is minimal compared to Group Pow)
        let new_p = Integer::from(&tuple.p_factor * weight);

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    /// 🛡️ [SECURITY HELPER]: 忙等待填充
    /// 执行无意义的类群运算以消耗时间，抚平功耗波动。
    fn perform_busy_wait(&self, duration: Duration) {
        let start = Instant::now();
        let mut trash = ClassGroupElement::generator(&self.discriminant);
        
        // 循环直到时间耗尽
        while start.elapsed() < duration {
            // 执行真实的数学运算，产生与正常逻辑相似的功耗特征
            // 不使用结果，防止编译器优化 (black_box 机制)
            if let Ok(res) = trash.square(&self.discriminant) {
                trash = res;
            }
            // 避免紧密循环导致的流水线停顿，插入极短的自旋提示
            std::hint::spin_loop(); 
        }
        
        // 防止编译器优化掉整个循环
        std::hint::black_box(trash);
    }

    // ... (generate_spacetime_noise, algebraic_reduction 保持不变) ...
    fn generate_spacetime_noise(&self, t: usize) -> Result<AffineTuple, String> {
        let g = ClassGroupElement::generator(&self.discriminant);
        let h_t = Integer::from(t + 1);
        let q_noise = g.pow(&h_t, &self.discriminant)?;
        Ok(AffineTuple {
            p_factor: Integer::from(1),
            q_shift: q_noise,
        })
    }

    fn algebraic_reduction(&self, tuple: AffineTuple, depth: usize) -> Result<AffineTuple, String> {
        let identity = AffineTuple::identity(&self.discriminant);
        if depth > 10 {
             return tuple.compose(&identity, &self.discriminant);
        }
        Ok(tuple)
    }
}
