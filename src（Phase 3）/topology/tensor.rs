// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::Integer;
use crate::phase3::core::affine::AffineTuple;
use crate::phase3::topology::merkle::IncrementalMerkleTree;
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use blake3::Hasher;

// [CONFIG]: Log Policy
const HOT_LAYER_SIZE: usize = 1024; // 内存只保留最近 1024 个 Chunk

/// 📜 LogEntry: 不可变的历史单元
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub index: u64,
    pub checkpoint_hash: [u8; 32], // 状态的数字指纹
    pub op_snapshot: AffineTuple,  // 当时的算子快照
    pub timestamp: u64,
}

/// 🗄️ EventLog: 冷热分层存储
#[derive(Serialize, Deserialize)]
pub struct EventLog {
    /// Hot Layer: 内存中的最近记录 (Ring Buffer 逻辑)
    pub hot_layer: Vec<LogEntry>,
    
    /// Merkle Accumulator: 全局状态承诺树
    pub commitment_tree: IncrementalMerkleTree,

    /// Cold Layer Path: 磁盘追加路径
    #[serde(skip)]
    pub cold_file_path: String,
}

impl EventLog {
    pub fn new(cold_path: String) -> Self {
        EventLog {
            hot_layer: Vec::new(),
            commitment_tree: IncrementalMerkleTree::new(),
            cold_file_path: cold_path,
        }
    }

    /// 📝 Append: 追加日志并更新 Merkle Tree
    pub fn append(&mut self, entry: LogEntry) -> Result<(), String> {
        // 1. Update Merkle Tree (Commitment)
        self.commitment_tree.append(entry.checkpoint_hash);

        // 2. Write to Disk (Cold Layer - Persistence)
        self.persist_to_cold(&entry)?;

        // 3. Update Memory (Hot Layer)
        if self.hot_layer.len() >= HOT_LAYER_SIZE {
            self.hot_layer.remove(0); // 简单的 FIFO 驱逐
        }
        self.hot_layer.push(entry);

        Ok(())
    }

    fn persist_to_cold(&self, entry: &LogEntry) -> Result<(), String> {
        // 使用追加模式打开文件
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.cold_file_path)
            .map_err(|e| e.to_string())?;
        
        let mut writer = BufWriter::new(file);
        
        // 使用 Bincode 或 JSON 序列化一行
        bincode::serialize_into(&mut writer, entry).map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        
        Ok(())
    }
}

/// 🧊 HyperTensor (Log Wrapper)
#[derive(Serialize, Deserialize)]
pub struct HyperTensor {
    pub dimensions: usize,
    pub side_length: usize,
    pub discriminant: Integer,
    pub event_log: EventLog,
}

impl HyperTensor {
    pub fn new(dim: usize, len: usize, discriminant: Integer) -> Self {
        HyperTensor {
            dimensions: dim,
            side_length: len,
            discriminant,
            // 默认使用临时文件，生产环境应传入真实路径
            event_log: EventLog::new("/tmp/htp_event_log.bin".to_string()),
        }
    }

    pub fn map_id_to_coord(&self, numeric_id: u64) -> Vec<usize> {
        vec![numeric_id as usize] 
    }

    /// 🖊️ Insert -> Append (Security Patched)
    pub fn insert(&mut self, _key: &str, checkpoint: AffineTuple, timestamp: u64) -> Result<(), String> {
        // 1. Calculate Hash of the Checkpoint (Comprehensive Hashing)
        // [SECURITY FIX]: 必须对语义状态 Q (ClassGroupElement) 进行完整哈希
        // 以前只哈希 P (通常为 1) 导致承诺为空。
        
        let mut hasher = Hasher::new();
        // [Fix 1]: Domain Separation Tag
        hasher.update(b"HTP_LOG_ENTRY_V1"); 
        
        // [Fix 2]: Hash P-Factor (虽然 Checkpoint 里通常是 1，但必须包含)
        hasher.update(&checkpoint.p_factor.to_digits(rug::integer::Order::Lsf));

        // [Fix 3]: Hash Q-Shift Components (Semantic State)
        // 这是最重要的修复，锁死语义内容。
        hasher.update(&checkpoint.q_shift.a.to_digits(rug::integer::Order::Lsf));
        hasher.update(&checkpoint.q_shift.b.to_digits(rug::integer::Order::Lsf));
        hasher.update(&checkpoint.q_shift.c.to_digits(rug::integer::Order::Lsf));
        
        let hash = hasher.finalize().into();

        // 2. Create Log Entry
        let entry = LogEntry {
            index: self.event_log.commitment_tree.leaf_count,
            checkpoint_hash: hash,
            op_snapshot: checkpoint,
            timestamp,
        };

        // 3. Append to Log
        self.event_log.append(entry)?;

        Ok(())
    }

    pub fn calculate_global_root(&self) -> Result<AffineTuple, String> {
        let root_hash = self.event_log.commitment_tree.root();
        
        // Wrap Hash into Integer for API compatibility
        let root_int = Integer::from_digits(&root_hash, rug::integer::Order::Lsf);
        
        Ok(AffineTuple {
            p_factor: root_int,
            q_shift: crate::phase3::core::affine::AffineTuple::identity(&self.discriminant).q_shift,
        })
    }

    pub fn get_segment_tree_path(&self, _coord: &Vec<usize>, _axis: usize) -> Vec<AffineTuple> {
        // Placeholder for Merkle Path retrieval
        vec![AffineTuple::identity(&self.discriminant)] 
    }
}
