// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::core::affine::AffineTuple;
use crate::phase3::topology::tensor::HyperTensor;
use crate::phase3::net::wire::HtpResponse; 
use crate::phase3::core::algebra::ClassGroupElement;
use rug::Integer;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use rand::{Rng, thread_rng};

pub struct HTPNeuron {
    pub p_weight: Integer,
    pub memory: Arc<RwLock<HyperTensor>>,
    pub discriminant: Integer,
    pub semantic_root: RwLock<ClassGroupElement>,
    pub commitment_buffer: RwLock<Vec<AffineTuple>>,
}

impl HTPNeuron {
    pub fn new(semantic_fingerprint: Integer, dim: usize, side_len: usize, discriminant: Integer) -> Self {
        let tensor = HyperTensor::new(dim, side_len, discriminant.clone());
        HTPNeuron {
            p_weight: semantic_fingerprint,
            memory: Arc::new(RwLock::new(tensor)),
            discriminant: discriminant.clone(),
            semantic_root: RwLock::new(ClassGroupElement::identity(&discriminant)),
            commitment_buffer: RwLock::new(Vec::new()),
        }
    }

    pub fn activate(
        &self, 
        input_stream: Vec<AffineTuple>, 
        recursion_depth: usize 
    ) -> Result<(AffineTuple, HtpResponse), String> {
        
        let start_time = Instant::now();
        const SECURITY_LATENCY_BUDGET_MS: u64 = 50;
        const CHUNK_SIZE: usize = 64; 

        let mut memory_guard = self.memory.write().map_err(|_| "Memory Lock poisoned")?;
        let mut s_guard = self.semantic_root.write().map_err(|_| "Semantic Root Lock poisoned")?;
        let mut buffer_guard = self.commitment_buffer.write().map_err(|_| "Buffer Lock poisoned")?;

        *s_guard = ClassGroupElement::identity(&self.discriminant);
        buffer_guard.clear();
        
        for (t, tuple) in input_stream.iter().enumerate() {
            // (a) Blinded Evolution
            let weighted_tuple = self.evolve_tuple_blinded(tuple, &self.p_weight)?;

            // (b) SpaceTime Noise
            let time_noise = self.generate_spacetime_noise(t)?;
            let step_op = weighted_tuple.compose(&time_noise, &self.discriminant)?;

            // Track A: Semantic Stream
            *s_guard = s_guard.apply_affine(&step_op.p_factor, &step_op.q_shift, &self.discriminant)?;

            // Track B: Commitment Buffer
            buffer_guard.push(step_op);

            // (c) Chunking & Checkpoint
            if buffer_guard.len() >= CHUNK_SIZE || t == input_stream.len() - 1 {
                // [CRITICAL]: 生成快照时，P 因子固定为 1。
                // 所有的语义信息都在 s_guard (Q) 中。
                // 配合 tensor.rs 的修复，Q 现在会被完整哈希，从而绑定住状态。
                let checkpoint = AffineTuple {
                    p_factor: Integer::from(1),
                    q_shift: s_guard.clone(),
                };

                let checkpoint_key = format!("chk:seq:{}", t);
                
                // 这里的 key 已不再关键，因为现在是 Log Append 模式
                memory_guard.insert(&checkpoint_key, checkpoint, t as u64)?;
                
                buffer_guard.clear();
            }
        }

        let raw_output = memory_guard.calculate_global_root()?;

        // 返回最新的语义状态
        let final_output = AffineTuple {
            p_factor: Integer::from(1),
            q_shift: s_guard.clone(), 
        };

        let proof_coord = memory_guard.map_id_to_coord(0); 
        let proof_path = memory_guard.get_segment_tree_path(&proof_coord, 0);
        
        let proof = HtpResponse::ProofBundle {
            request_id: 0,
            primary_path: proof_path,
            orthogonal_anchors: vec![],
            epoch: recursion_depth as u64,
        };

        let elapsed = start_time.elapsed();
        let target_duration = Duration::from_millis(SECURITY_LATENCY_BUDGET_MS);
        if elapsed < target_duration {
            self.perform_busy_wait(target_duration - elapsed);
        }

        Ok((final_output, proof))
    }

    fn evolve_tuple_blinded(&self, tuple: &AffineTuple, weight: &Integer) -> Result<AffineTuple, String> {
        let mut rng = thread_rng();
        let blind_exp = Integer::from(rng.gen::<u64>());
        let generator = ClassGroupElement::generator(&self.discriminant);
        let r_blind = generator.pow(&blind_exp, &self.discriminant)?;
        
        let q_blinded = tuple.q_shift.compose(&r_blind, &self.discriminant)?;
        let q_prime_blinded = q_blinded.pow(weight, &self.discriminant)?;
        
        let neg_weight = -weight.clone();
        let r_w = r_blind.pow(weight, &self.discriminant)?;
        let r_w_inv = ClassGroupElement {
            a: r_w.a,
            b: -r_w.b, 
            c: r_w.c,
        };
        
        let new_q = q_prime_blinded.compose(&r_w_inv, &self.discriminant)?;
        let new_p = Integer::from(&tuple.p_factor * weight);

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

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
}
