//! The Body Module (躯体模块)
//! 
//! 负责系统的显化 (Materialization)。
//! 它将 Soul 层非交换的四元数状态投影为两个维度：
//! 
//! 1. Projection (投影): 
//!    - 连续投影：提取 R^4 空间中的方向向量，为优化器提供梯度。
//!    - 精确投影：提取代数结构的混沌哈希，作为因果证明。
//! 
//! 2. Adapter (适配):
//!    - 将精确投影的熵转化为具体的逻辑门操作 (ProofAction)。

pub mod adapter;
pub mod projection;

// Re-export for easier access
pub use adapter::{Adapter, ProofAction, LogicOp};
pub use projection::Projector;
