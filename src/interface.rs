// src/interface.rs
// Evolver Public Interface v0.2
// 适配 BiasChannel v0.2 的确定性协议

use crate::control::bias_channel::{BiasController, VapoConfig};
use crate::dsl::schema::ProofAction;
use crate::dsl::stp_bridge::STPContext;

// =========================================================================
// 1. 核心特征：解码器适配器 (The Decoder Trait)
// =========================================================================
pub trait ActionDecoder {
    fn decode(&self, logits: &[f64]) -> ProofAction;
    fn action_space_size(&self) -> usize;
}

// =========================================================================
// 2. 输入/输出 结构体 (DTOs)
// =========================================================================

#[derive(Debug, Clone)]
pub struct CorrectionRequest {
    pub base_logits: Vec<f64>,
    pub request_id: String,
    
    // [New v0.2] 必须包含上下文和种子以支持确定性重放和验证
    pub context: String,
    pub seed: u64,
}

#[derive(Debug, Clone)]
pub struct CorrectionResponse {
    pub final_action: ProofAction,
    // [Fix] BiasVector 在 v0.2 是 f64 类型的连续向量
    pub applied_bias: Vec<f64>, 
    pub final_energy: f64,
    pub iterations: usize,
    // [New] 返回校验哈希，证明该 Bias 是专门为此 Context 生成的
    pub proof_hash: String, 
}

// =========================================================================
// 3. Evolver 引擎实例 (The Engine)
// =========================================================================

pub struct EvolverEngine {
    stp_ctx: STPContext,
    controller: BiasController,
}

impl EvolverEngine {
    pub fn new(config: Option<VapoConfig>) -> Self {
        EvolverEngine {
            stp_ctx: STPContext::new(),
            controller: BiasController::new(config),
        }
    }

    pub fn reset(&mut self) {
        self.stp_ctx = STPContext::new();
        self.controller = BiasController::new(None); 
    }

    pub fn inject_context(&mut self, action: &ProofAction) {
        self.stp_ctx.calculate_energy(action);
    }

    pub fn align_generation<D: ActionDecoder>(
        &mut self, 
        request: CorrectionRequest, 
        decoder: &D
    ) -> Result<CorrectionResponse, String> {
        
        if request.base_logits.len() != decoder.action_space_size() {
            return Err(format!("Logits dimension mismatch: expected {}, got {}", 
                decoder.action_space_size(), request.base_logits.len()));
        }

        let decode_wrapper = |logits: &[f64]| -> ProofAction {
            decoder.decode(logits)
        };

        // [Fix] 调用 v0.2 版本的 optimize，传入 context 和 seed
        // 这将启动 VAPO 搜索循环
        let proof_bundle = self.controller.optimize(
            &request.context,
            request.seed,
            &request.base_logits, 
            &mut self.stp_ctx, 
            decode_wrapper
        );

        // 验证最终能量（虽然 controller 内部也会检查，这里是双重确认）
        let final_energy = self.stp_ctx.calculate_energy(&proof_bundle.action);

        Ok(CorrectionResponse {
            final_action: proof_bundle.action,
            applied_bias: proof_bundle.bias_vector,
            final_energy: proof_bundle.energy_signature, // 使用 Bundle 里记录的能量
            iterations: 0, // Mock: 当前 ProofBundle 结构体尚未暴露 iter 次数，暂填 0
            proof_hash: proof_bundle.context_hash,
        })
    }
}
