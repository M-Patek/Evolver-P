// src/interface.rs
// Evolver Public Interface
// 这是一个最小化的对外接口，用于将 Evolver 集成到更大的系统中。
// 它封装了 VAPO 循环和 STP 状态管理。

use crate::control::bias_channel::{BiasController, BiasVector, VapoConfig};
use crate::dsl::schema::ProofAction;
use crate::dsl::stp_bridge::STPContext;
use std::sync::{Arc, Mutex};

// =========================================================================
// 1. 核心特征：解码器适配器 (The Decoder Trait)
// =========================================================================
// 外部系统（如 LLM）可能有不同的词表或解码策略。
// 必须实现此特征，告诉 Evolver 如何把 "Logits" 变成 "Action"。
pub trait ActionDecoder {
    /// 给定当前的 logits（可能已经被 Bias 修正过），解码出一个 ProofAction
    fn decode(&self, logits: &[f64]) -> ProofAction;
    
    /// 获取 Logits 的维度 (Action Space Size)
    fn action_space_size(&self) -> usize;
}

// =========================================================================
// 2. 输入/输出 结构体 (DTOs)
// =========================================================================

#[derive(Debug, Clone)]
pub struct CorrectionRequest {
    /// 原始生成器输出的 Logits (Base Logits)
    pub base_logits: Vec<f64>,
    
    /// 上下文元数据 (可选，用于日志或特定的 STP 状态注入)
    pub request_id: String,
}

#[derive(Debug, Clone)]
pub struct CorrectionResponse {
    /// 最终修正后的动作 (Safe Action)
    pub final_action: ProofAction,
    
    /// 最终施加的 Bias (用于训练反馈/RLHF)
    pub applied_bias: Vec<i32>,
    
    /// 最终的能量值 (0.0 表示逻辑完美，>0 表示仍有瑕疵)
    pub final_energy: f64,
    
    /// 优化迭代次数
    pub iterations: usize,
}

// =========================================================================
// 3. Evolver 引擎实例 (The Engine)
// =========================================================================

pub class EvolverEngine {
    // 内部状态保持
    stp_ctx: STPContext,
    controller: BiasController,
}
// Rust struct implementation below
pub struct EvolverEngine {
    stp_ctx: STPContext,
    controller: BiasController,
}

impl EvolverEngine {
    /// 初始化一个新的 Evolver 引擎
    /// 通常一个 Session 或一个 Proof 任务对应一个实例
    pub fn new(config: Option<VapoConfig>) -> Self {
        EvolverEngine {
            stp_ctx: STPContext::new(), // 加载标准定理库
            controller: BiasController::new(config),
        }
    }

    /// 重置状态 (例如开始一个新的证明任务)
    pub fn reset(&mut self) {
        self.stp_ctx = STPContext::new();
        // Controller 状态可能需要保留 Bias 作为一个先验，或者也重置
        self.controller = BiasController::new(None); 
    }

    /// 手动注入外部状态 (Context Injection)
    /// 如果外部系统想预设一些变量 (例如 "Goal" 或历史定义)
    pub fn inject_context(&mut self, action: &ProofAction) {
        // 直接在 STP 中执行而不进行 Bias 修正（假设外部注入是可信的）
        self.stp_ctx.calculate_energy(action);
    }

    /// 核心方法：对齐生成 (Align Generation)
    /// 接收 Raw Logits -> 运行 VAPO 循环 -> 返回 Correct Action
    pub fn align_generation<D: ActionDecoder>(
        &mut self, 
        request: CorrectionRequest, 
        decoder: &D
    ) -> Result<CorrectionResponse, String> {
        
        // 1. 维度检查
        if request.base_logits.len() != decoder.action_space_size() {
            return Err(format!("Logits dimension mismatch: expected {}, got {}", 
                decoder.action_space_size(), request.base_logits.len()));
        }

        // 2. 定义解码闭包，适配 VAPO 的接口要求
        // VAPO 需要 Fn(&[f64]) -> ProofAction
        let decode_wrapper = |logits: &[f64]| -> ProofAction {
            decoder.decode(logits)
        };

        // 3. 运行 VAPO 优化循环
        // 注意：这里我们需要修改 bias_channel.rs 的 optimize 签名以返回更多元数据，
        // 或者在这里重新封装一下。为了最小改动，我们假设 optimize 返回 (Bias, Action)。
        let (final_bias, final_action) = self.controller.optimize(
            &request.base_logits, 
            &mut self.stp_ctx, 
            decode_wrapper
        );

        // 4. 计算最终能量 (用于报告)
        // 注意：这里需要再次计算，或者让 optimize 返回 energy
        // 为了简单，我们再次调用（calculate_energy 是幂等的或低副作用的）
        let final_energy = self.stp_ctx.calculate_energy(&final_action);

        Ok(CorrectionResponse {
            final_action,
            applied_bias: final_bias.data,
            final_energy,
            iterations: 0, // 暂时没从 optimize 返回，实际应暴露
        })
    }
}
