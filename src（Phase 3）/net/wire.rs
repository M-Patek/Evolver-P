// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::core::affine::AffineTuple;
use crate::phase3::core::algebra::ClassGroupElement;
use rug::Integer;
use serde::{Serialize, Deserialize};
use blake3::Hasher;

/// 🌳 Merkle Inclusion Proof
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MerkleProof {
    pub leaf_index: u64,
    pub leaf_hash: [u8; 32],
    pub siblings: Vec<[u8; 32]>,
}

impl MerkleProof {
    pub fn verify(&self, global_root: &[u8; 32]) -> bool {
        let mut current_hash = self.leaf_hash;
        let mut index = self.leaf_index;

        for sibling in &self.siblings {
            let mut hasher = Hasher::new();
            hasher.update(b"HTP_MERKLE_NODE");

            if index % 2 == 0 {
                hasher.update(&current_hash);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(&current_hash);
            }
            
            current_hash = hasher.finalize().into();
            index /= 2;
        }

        &current_hash == global_root
    }
}

/// ⏭️ State Transition Proof
#[derive(Serialize, Deserialize, Debug)]
pub struct StateTransitionProof {
    pub checkpoint_state: ClassGroupElement,
    pub log_inclusion_proof: MerkleProof,
    pub replay_ops: Vec<AffineTuple>,
    pub claimed_final_state: ClassGroupElement,
}

impl StateTransitionProof {
    /// 🛡️ 执行跳表验证 (Security Patched)
    /// 这是 HTP 的“最高法院”，审判一切状态转移的合法性。
    pub fn verify(&self, global_merkle_root: &[u8; 32], discriminant: &Integer) -> bool {
        // [Fix Step 0]: Binding Check (状态-哈希绑定检查)
        // 边界一：身份绑定。
        // 验证者必须确信：这个 checkpoint_state 生成的哈希值，
        // 确实等于 Merkle Proof 中声称的 leaf_hash。
        // 这防止了“拿着真的 Proof 验证假的 State”的攻击。
        
        let mut hasher = Hasher::new();
        hasher.update(b"HTP_LOG_ENTRY_V1"); // Tag 必须一致
        
        // 重构 Checkpoint Tuple (P 固定为 1, Q 为状态)
        let p_one = Integer::from(1);
        hasher.update(&p_one.to_digits(rug::integer::Order::Lsf));
        
        // Hash Q components (a, b, c)
        hasher.update(&self.checkpoint_state.a.to_digits(rug::integer::Order::Lsf));
        hasher.update(&self.checkpoint_state.b.to_digits(rug::integer::Order::Lsf));
        hasher.update(&self.checkpoint_state.c.to_digits(rug::integer::Order::Lsf));
        
        let computed_leaf_hash: [u8; 32] = hasher.finalize().into();

        if computed_leaf_hash != self.log_inclusion_proof.leaf_hash {
             println!("❌ Security Alert: Checkpoint State does not match the Merkle Proof.");
             return false;
        }

        // [Fix Step 1]: Audit the Log (审计日志)
        // 边界二：历史存在性。
        // 任何无法溯源到 Global Root 的状态都是“幻觉”。
        if !self.log_inclusion_proof.verify(global_merkle_root) {
            println!("❌ Verification Failed: Merkle proof invalid. Checkpoint not found in Log.");
            return false;
        }

        // [Fix Step 2]: Replay Evolution (重放演化)
        // 边界三：逻辑一致性。
        // 从起点出发，严格按照记录的步骤走，必须能走到终点。
        let mut computed_state = self.checkpoint_state.clone();
        
        for (i, op) in self.replay_ops.iter().enumerate() {
            // Apply atomic transition
            // 这里的 apply_affine 会触发底层的代数检查
            match computed_state.apply_affine(&op.p_factor, &op.q_shift, discriminant) {
                Ok(new_state) => computed_state = new_state,
                Err(e) => {
                    println!("❌ Verification Error during replay at step {}: {}", i, e);
                    return false;
                }
            }
        }

        // Step 3: 最终一致性检查
        if computed_state != self.claimed_final_state {
            println!("❌ Verification Failed: State mismatch.");
            return false;
        }

        true
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HtpResponse {
    ProofBundle {
        request_id: u64,
        proof: StateTransitionProof,
        log_epoch: u64,
    },
    Ack,
    Error(String),
}
