// src/dsl/stp_bridge.rs
use std::collections::HashMap;
use crate::dsl::schema::ProofAction;
use crate::crypto::algebra::Matrix; 

/// STP 上下文环境
/// 负责维护当前的代数状态，并将高层的 ProofAction 编译为底层的矩阵算子。
#[derive(Debug, Clone)]
pub struct STPContext {
    // 状态矩阵：代表当前系统的全局状态 x(t)
    pub state_matrix: Matrix,
    
    // 能量值：E(t)，0.0 表示自洽，>0 表示存在矛盾
    pub energy: f64,
    
    // 符号表：存储变量名及其对应的代数定义 (Valuation)
    // 在 Mock 阶段，我们用简单的 Label 矩阵来表示 (Odd/Even)
    pub variables: HashMap<String, Matrix>,
}

impl STPContext {
    pub fn new() -> Self {
        STPContext {
            state_matrix: Matrix::identity(1),
            energy: 0.0,
            variables: HashMap::new(),
        }
    }

    /// 核心接口：计算给定动作的能量
    /// 修复：必须是 &mut self，因为需要更新内部状态
    pub fn calculate_energy(&mut self, action: &ProofAction) -> f64 {
        let energy = match action {
            ProofAction::Define { symbol, hierarchy_path } => {
                // 定义动作：注册变量到符号表
                let val = self.mock_valuation_from_path(hierarchy_path);
                self.variables.insert(symbol.clone(), val);
                0.0
            },
            
            ProofAction::Apply { theorem_id, inputs, output_symbol } => {
                // 推导动作：检查逻辑一致性
                self.check_inference_consistency(theorem_id, inputs, output_symbol)
            },

            _ => 0.0,
        };

        self.energy = energy;
        energy
    }

    // --- 内部辅助逻辑 ---

    fn mock_valuation_from_path(&self, path: &[String]) -> Matrix {
        // 简单 Mock：如果是 "Odd" 返回 [1, 0], "Even" 返回 [0, 1]
        // 这里只是为了演示，实际应解析完整路径
        if path.iter().any(|s| s == "Odd") {
            Matrix::new(vec![vec![1.0], vec![0.0]]) // Vector [1, 0]
        } else {
            Matrix::new(vec![vec![0.0], vec![1.0]]) // Vector [0, 1]
        }
    }

    fn check_inference_consistency(&self, theorem: &str, inputs: &[String], output_sym: &str) -> f64 {
        if theorem == "ModAdd" && inputs.len() == 2 {
            let val_a = self.variables.get(&inputs[0]);
            let val_b = self.variables.get(&inputs[1]);
            // 注意：这里检查的是 output_sym *当前* 的定义是否与推导结果冲突
            // 如果 output_sym 还没定义，通常意味着 Apply 会生成它（能量为0）
            // 如果 output_sym 已经定义了（比如 Generator 瞎猜了一个值），我们就检查冲突
            let val_out = self.variables.get(output_sym);

            if let (Some(a), Some(b), Some(out)) = (val_a, val_b, val_out) {
                // 逻辑：Odd(1,0) + Odd(1,0) should be Even(0,1)
                let is_a_odd = a[0][0] > 0.5;
                let is_b_odd = b[0][0] > 0.5;
                let is_out_odd = out[0][0] > 0.5;

                // 奇数加法规则：Odd + Odd = Even (False)
                let expected_odd = is_a_odd ^ is_b_odd; 
                
                if is_out_odd == expected_odd {
                    return 0.0; // 符合逻辑
                } else {
                    return 10.0; // 逻辑矛盾！
                }
            }
        }
        0.0 // 默认宽容
    }
}
