// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::core::affine::AffineTuple;
use crate::phase3::topology::tensor::HyperTensor;
use crate::phase3::net::wire::HtpResponse; 
use crate::phase3::core::algebra::ClassGroupElement; // 需要引用以进行盲化操作
use rug::Integer;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use rand::{Rng, thread_rng};

/// HTPNeuron: 仿射神经元 (The Processor)
/// 
/// [Phase 2 Refactor]: 实现双轨制 (Dual-Track)
/// 1. Semantic Track: 使用 ClassGroupElement 进行流式演化，无 P 因子累积。
/// 2. Commitment Track: 使用 Buffer 暂存操作，批量生成 Proof Checkpoints。
pub struct HTPNeuron {
    pub p_weight: Integer,
    pub memory: Arc<RwLock<HyperTensor>>,
    pub discriminant: Integer,
    
    /// 🧠 [Semantic Root]: 当前语义状态 $S$
    /// 它是 ClassGroupElement (一等公民)，大小恒定，永不爆炸。
    /// 使用 RwLock 支持即使在只读引用 (&self) 下也能更新内部状态 (Internal Mutability)。
    pub semantic_root: RwLock<ClassGroupElement>,

    /// 📝 [Commitment Buffer]: 待提交的局部算子缓冲区
    /// 用于暂存 Chunk 内的操作，每 K 步刷写一次到持久化存储。
    pub commitment_buffer: RwLock<Vec<AffineTuple>>,
}

impl HTPNeuron {
    pub fn new(semantic_fingerprint: Integer, dim: usize, side_len: usize, discriminant: Integer) -> Self {
        let tensor = HyperTensor::new(dim, side_len, discriminant.clone());
        HTPNeuron {
            p_weight: semantic_fingerprint,
            memory: Arc::new(RwLock::new(tensor)),
            discriminant: discriminant.clone(),
            // 初始化为单位元
            semantic_root: RwLock::new(ClassGroupElement::identity(&discriminant)),
            commitment_buffer: RwLock::new(Vec::new()),
        }
    }

    /// ⚡ Algebraic Activation (Streamlined)
    /// 
    /// 重写后的激活函数，剥离了计算与存储。
    pub fn activate(
        &self, 
        input_stream: Vec<AffineTuple>, 
        recursion_depth: usize 
    ) -> Result<(AffineTuple, HtpResponse), String> {
        
        // [TIMING PROTECTION]
        let start_time = Instant::now();
        const SECURITY_LATENCY_BUDGET_MS: u64 = 50;
        const CHUNK_SIZE: usize = 64; // 每 64 步生成一个 Checkpoint

        // 获取锁
        let mut memory_guard = self.memory.write().map_err(|_| "Memory Lock poisoned")?;
        let mut s_guard = self.semantic_root.write().map_err(|_| "Semantic Root Lock poisoned")?;
        let mut buffer_guard = self.commitment_buffer.write().map_err(|_| "Buffer Lock poisoned")?;

        // 重置当前推理的状态 (如果是新的推理请求)
        // 在流式服务中，可能希望保持状态，但这里假设每次 activate 是独立的 Sequence
        *s_guard = ClassGroupElement::identity(&self.discriminant);
        buffer_guard.clear();
        
        // 1. [Dual-Track Evolution Loop]
        for (t, tuple) in input_stream.iter().enumerate() {
            // (a) [Blinded Evolution]: Local Op Generation
            // Op = Tuple ^ P_weight
            let weighted_tuple = self.evolve_tuple_blinded(tuple, &self.p_weight)?;

            // (b) [SpaceTime Noise]
            let time_noise = self.generate_spacetime_noise(t)?;
            
            // Combine: Op_final = Weighted * Noise
            // 这里的 compose 是局部算子聚合，受到 MAX_CHUNK_P_BITS 保护，是安全的
            let step_op = weighted_tuple.compose(&time_noise, &self.discriminant)?;

            // === Track A: Semantic Stream (Computation) ===
            // S_new = S_old.apply(p, q)
            // P 因子在这里被立即消耗，转化为群元素的变换。
            // 这一步保证了状态 S 永远不会膨胀。
            *s_guard = s_guard.apply_affine(&step_op.p_factor, &step_op.q_shift, &self.discriminant)?;

            // === Track B: Commitment Buffer (Storage) ===
            // 将操作推入缓冲区
            buffer_guard.push(step_op);

            // (c) [Chunking & Checkpoint]
            // 每 K 步，或者在流的末尾，我们需要生成一个 Checkpoint
            if buffer_guard.len() >= CHUNK_SIZE || t == input_stream.len() - 1 {
                // 生成 Checkpoint：当前语义根 $S$ 的快照
                // 这里的 P 设为 1，因为状态已经包含在 Q (ClassGroupElement) 中了
                let checkpoint = AffineTuple {
                    p_factor: Integer::from(1),
                    q_shift: s_guard.clone(),
                };

                let checkpoint_key = format!("chk:seq:{}", t);
                
                // 写入 HyperTensor (Commitment)
                // 注意：这里我们存的是“状态快照”，而不是累计的算子。
                // 这允许验证者直接验证某个时间点的状态。
                memory_guard.insert(&checkpoint_key, checkpoint, t as u64)?;
                
                // 清空缓冲区 (在更复杂的实现中，可能还需要对 buffer 内的 op 进行 Merkle 聚合)
                buffer_guard.clear();
            }
        }

        // 2. [Fold]: 全息折叠 (Holographic Collapse)
        // 从 Memory 中获取全局根 (这部分逻辑保持不变，依然基于 Tensor 结构)
        let raw_output = memory_guard.calculate_global_root()?;

        // 3. [Output Formatting]
        // 最终输出需要是 AffineTuple 格式以兼容接口
        // 我们返回 (1, S_final)
        let final_output = AffineTuple {
            p_factor: Integer::from(1),
            q_shift: s_guard.clone(), 
        };
        // 注意：calculate_global_root 返回的可能是基于 Checkpoint 聚合的结果
        // 在强一致性要求下，我们可以直接使用 *s_guard 作为最新状态。
        // 这里为了兼容性，我们优先使用 s_guard (最新内存状态)

        // 4. [Proof Generation]
        let proof_coord = memory_guard.map_id_to_coord(0); 
        let proof_path = memory_guard.get_segment_tree_path(&proof_coord, 0);
        
        let proof = HtpResponse::ProofBundle {
            request_id: 0,
            primary_path: proof_path,
            orthogonal_anchors: vec![],
            epoch: recursion_depth as u64,
        };

        // [SECURITY]: Busy-Wait Padding
        let elapsed = start_time.elapsed();
        let target_duration = Duration::from_millis(SECURITY_LATENCY_BUDGET_MS);
        if elapsed < target_duration {
            self.perform_busy_wait(target_duration - elapsed);
        }

        Ok((final_output, proof))
    }

    /// 🛡️ [SECURITY CORE]: Blinded Evolution
    fn evolve_tuple_blinded(&self, tuple: &AffineTuple, weight: &Integer) -> Result<AffineTuple, String> {
        let mut rng = thread_rng();
        
        // 1. Generate random R
        let blind_exp = Integer::from(rng.gen::<u64>());
        let generator = ClassGroupElement::generator(&self.discriminant);
        let r_blind = generator.pow(&blind_exp, &self.discriminant)?;
        
        // 2. Compute Blinded Base: T' = T * R
        let q_blinded = tuple.q_shift.compose(&r_blind, &self.discriminant)?;
        
        // 3. Exponentiate: Res' = (T * R)^W
        let q_prime_blinded = q_blinded.pow(weight, &self.discriminant)?;
        
        // 4. Unblind: Res = Res' * (R^W)^(-1)
        let neg_weight = -weight.clone();
        let r_w = r_blind.pow(weight, &self.discriminant)?;
        let r_w_inv = ClassGroupElement {
            a: r_w.a,
            b: -r_w.b, 
            c: r_w.c,
        };
        
        let new_q = q_prime_blinded.compose(&r_w_inv, &self.discriminant)?;
        
        // P 部分直接计算 
        let new_p = Integer::from(&tuple.p_factor * weight);

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    /// 🛡️ [SECURITY HELPER]: 忙等待填充
    fn perform_busy_wait(&self, duration: Duration) {
        let start = Instant::now();
        let mut trash = ClassGroupElement::generator(&self.discriminant);
        while start.elapsed() < duration {
            if let Ok(res) = trash.square(&self.discriminant) {
                trash = res;
            }
            std::hint::spin_loop(); 
        }
        std::hint::black_box(trash);
    }

    fn generate_spacetime_noise(&self, t: usize) -> Result<AffineTuple, String> {
        let g = ClassGroupElement::generator(&self.discriminant);
        let h_t = Integer::from(t + 1);
        let q_noise = g.pow(&h_t, &self.discriminant)?;
        Ok(AffineTuple {
            p_factor: Integer::from(1),
            q_shift: q_noise,
        })
    }

    // 依然保留，虽然现在流式演化天然规约，但为了接口完整性
    fn algebraic_reduction(&self, tuple: AffineTuple, _depth: usize) -> Result<AffineTuple, String> {
        Ok(tuple)
    }
}
