// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::affine::AffineTuple;
use crate::core::algebra::ClassGroupElement;
use crate::topology::tensor::HyperTensor;
use crate::net::wire::HtpResponse; // 复用 ProofBundle 结构
use rug::Integer;
use std::sync::{Arc, RwLock};

/// 🧠 HTPNeuron: 仿射神经元
/// 不再处理浮点数，而是吞吐代数元组，进行逻辑演化。
pub struct HTPNeuron {
    /// [Semantic Fingerprint]: 神经元的“权重”，一个代表特定语义（如“科技”）的大素数
    pub p_weight: Integer,
    
    /// [Internal Memory]: 微型超张量，用于短期记忆和上下文折叠
    pub memory: Arc<RwLock<HyperTensor>>,
    
    /// [System Params]: 用于群运算的判别式
    pub discriminant: Integer,
}

impl HTPNeuron {
    /// 创建一个新的神经元，分配其独特的语义指纹
    pub fn new(semantic_fingerprint: Integer, dim: usize, side_len: usize, discriminant: Integer) -> Self {
        let tensor = HyperTensor::new(dim, side_len, discriminant.clone());
        HTPNeuron {
            p_weight: semantic_fingerprint,
            memory: Arc::new(RwLock::new(tensor)),
            discriminant,
        }
    }

    /// ⚡ Algebraic Activation: 代数激活函数
    /// 输入流 -> 非交换演化 -> 注入记忆 -> 折叠 -> 规约 -> 输出 + 证明
    pub fn activate(
        &self, 
        input_stream: Vec<AffineTuple>, 
        recursion_depth: usize // [Residual Management]
    ) -> Result<(AffineTuple, HtpResponse), String> {
        
        let mut memory_guard = self.memory.write().map_err(|_| "Lock poisoned")?;
        
        // 1. [Non-Commutative Evolution]: S_in ^ P_weight * G ^ H(t)
        // 这里的 P_weight 取代了传统神经网络的 W
        for (t, tuple) in input_stream.iter().enumerate() {
            // (a) 加权: Tuple ^ P_weight (类似 x * w)
            // 注意：AffineTuple 的幂运算意味着 repeated composition
            let weighted_tuple = self.evolve_tuple(tuple, &self.p_weight)?;

            // (b) 注入时空噪声: * G ^ H(t)
            // 每一个输入的位置 t 都会产生唯一的代数影响
            let time_noise = self.generate_spacetime_noise(t)?;
            let evolved = weighted_tuple.compose(&time_noise, &self.discriminant)?;

            // (c) 写入内部记忆张量
            // 简单的映射逻辑：将时序 t 映射到张量坐标
            let coord_str = format!("seq:{}", t);
            memory_guard.insert(&coord_str, evolved)?;
        }

        // 2. [Fold]: 坍缩多维状态
        // 使用 HyperTensor 的稀疏折叠算法获取当前的全息状态 (Global Root)
        // recursion_depth 可以在这里控制折叠的层级（如果 API 支持），这里模拟为完全折叠
        let raw_output = memory_guard.calculate_global_root()?;

        // 3. [Reduce]: 规约防止系数爆炸
        // 在 HTP 中，compose 操作通常自带 reduce，但为了显式符合“代数激活”定义：
        let final_output = self.algebraic_reduction(raw_output, recursion_depth)?;

        // 4. [Proof Generation]: 生成推理证明
        // 随机抽取一个维度的路径作为“解释性证明”
        // 在真实 AI 场景中，这代表模型“为什么”得出这个结论的逻辑链
        let proof_coord = memory_guard.map_id_to_coord(0); // 示例：取 0 号位的解释
        let proof_path = memory_guard.get_segment_tree_path(&proof_coord, 0);
        
        // 构造 ProofBundle (复用 wire 中的定义)
        let proof = HtpResponse::ProofBundle {
            request_id: 0, // 内部调用无 ID
            primary_path: proof_path,
            orthogonal_anchors: vec![], // 简化
            epoch: recursion_depth as u64,
        };

        Ok((final_output, proof))
    }

    /// 内部助手：对单个元组应用权重 P
    fn evolve_tuple(&self, tuple: &AffineTuple, weight: &Integer) -> Result<AffineTuple, String> {
        // 逻辑：AffineTuple (P, Q) ^ W => (P^W, Q_new)
        // 这是一个递归组合过程，如果 W 很大，这里就是深度的非线性变换
        // 为简化演示，我们只对 Q 部分做幂运算，P 部分做乘法 (同态性质)
        
        // S_{out} = S_{in}^W
        // 实际实现应调用 tuple.pow(weight) 如果 AffineTuple 实现了 pow
        // 这里手动模拟：
        let new_p = Integer::from(&tuple.p_factor * weight);
        let new_q = tuple.q_shift.pow(weight, &self.discriminant)?;
        
        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    /// 内部助手：生成 G^H(t)
    fn generate_spacetime_noise(&self, t: usize) -> Result<AffineTuple, String> {
        let g = ClassGroupElement::generator(&self.discriminant);
        // H(t) = hash(t)
        let h_t = Integer::from(t + 1); // 简化哈希
        let q_noise = g.pow(&h_t, &self.discriminant)?;
        
        // 噪声项的 P 通常为 1 (Identity)
        Ok(AffineTuple {
            p_factor: Integer::from(1),
            q_shift: q_noise,
        })
    }

    /// [Residual Management]: 模拟代数规约与噪声过滤
    fn algebraic_reduction(&self, tuple: AffineTuple, depth: usize) -> Result<AffineTuple, String> {
        // 如果递归深度太深，我们可能会丢弃一部分精度或者强制规约
        // 这里调用底层的 reduce_form (通过 compose 触发)
        let identity = AffineTuple::identity(&self.discriminant);
        
        // "Residual Cutoff": 如果深度超过阈值，引入额外的平滑项
        if depth > 10 {
             // 模拟：强行规约
             return tuple.compose(&identity, &self.discriminant);
        }
        
        Ok(tuple)
    }
}
