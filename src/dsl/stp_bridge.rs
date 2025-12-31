use std::collections::HashMap;
use crate::dsl::schema::ProofAction;
use crate::dsl::math_kernel::Matrix;

/// STP (Semi-Tensor Product) Context
/// 逻辑物理引擎 v2.0 (Strict Mode)
///
/// [Security Update]: 修复了“未定义输入返回 0 能量”的漏洞。
/// 现在，任何试图访问未定义状态的行为都会触发高能惩罚 (Energy = 100.0)。
pub struct STPContext {
    // 状态空间：符号 -> 逻辑向量 (e.g., "n" -> [1, 0]^T)
    pub state: HashMap<String, Matrix>,
    
    // 规则空间：定理ID -> 结构矩阵 (e.g., "ModAdd" -> M_add)
    pub operators: HashMap<String, Matrix>,
}

impl STPContext {
    pub fn new() -> Self {
        let mut ctx = STPContext {
            state: HashMap::new(),
            operators: HashMap::new(),
        };
        ctx.init_operators();
        ctx
    }

    fn init_operators(&mut self) {
        // Matrix M_add (2 x 4):
        // [ 1.0, 0.0, 0.0, 1.0 ] (Even 行)
        // [ 0.0, 1.0, 1.0, 0.0 ] (Odd 行)
        let m_add = Matrix::new(2, 4, vec![
            1.0, 0.0, 0.0, 1.0, 
            0.0, 1.0, 1.0, 0.0  
        ]);
        
        self.operators.insert("ModAdd".to_string(), m_add);
    }

    /// 核心能量计算函数 (The Strict Violation Function)
    /// 
    /// 返回值定义：
    /// - 0.0: 逻辑完美自洽 (Truth)
    /// - 1.0: 逻辑推导错误 (Falsehood)
    /// - 100.0: 逻辑断裂/未定义 (Chaos/Void) - 严厉惩罚！
    pub fn calculate_energy(&mut self, action: &ProofAction) -> f64 {
        match action {
            ProofAction::Define { symbol, hierarchy_path } => {
                let val_type = hierarchy_path.last().map(|s| s.as_str()).unwrap_or("");
                
                let vector = if val_type == "Odd" {
                    Matrix::new(2, 1, vec![0.0, 1.0])
                } else if val_type == "Even" {
                    Matrix::new(2, 1, vec![1.0, 0.0])
                } else {
                    // [Fix]: 定义未知类型也是一种风险，给予轻微惩罚或默认向量
                    // 这里我们暂时允许通过，但在严格模式下可能需要 Panic
                    Matrix::new(2, 1, vec![0.5, 0.5]) 
                };
                
                self.state.insert(symbol.clone(), vector);
                0.0 
            },
            
            ProofAction::Apply { theorem_id, inputs, output_symbol } => {
                // [Critical Fix]: 严厉检查输入是否存在
                if inputs.is_empty() {
                    return 100.0; // 空输入惩罚
                }

                // 1. 获取并检查第一个输入
                let v1 = match self.state.get(&inputs[0]) {
                    Some(v) => v,
                    None => {
                        // println!("DEBUG: Input symbol '{}' not found!", inputs[0]);
                        return 100.0; // [Penalty] 引用未定义符号
                    }
                };
                
                // 2. 处理二元运算张量积
                let v_input_tensor = if inputs.len() > 1 {
                    let v2 = match self.state.get(&inputs[1]) {
                        Some(v) => v,
                        None => {
                            // println!("DEBUG: Input symbol '{}' not found!", inputs[1]);
                            return 100.0; // [Penalty] 引用未定义符号
                        }
                    };
                    v1.kron(v2) 
                } else {
                    v1.clone()
                };

                // 3. 获取算子
                let m_op = match self.operators.get(theorem_id) {
                    Some(m) => m,
                    None => return 100.0, // [Penalty] 调用不存在的定理
                };

                // 4. 物理推演 (V_truth)
                let v_truth = match m_op.matmul(&v_input_tensor) {
                    Ok(v) => v,
                    Err(_) => return 100.0, // [Penalty] 维度崩塌
                };

                // 5. 获取声明结果 (V_claim)
                let v_claim = match self.state.get(output_symbol) {
                    Some(v) => v,
                    None => {
                        // 这是一个微妙的情况：如果 Apply 的目的是生成 output_symbol，
                        // 那么它此时可能还不存在。但在 Evolver 的逻辑里，
                        // 通常是先 Define (猜想)，再 Apply (验证)。
                        // 如果 output_symbol 没被 Define 过，就没有参照物来计算 Energy。
                        // 所以这里找不到 claim 也是一种错误。
                        return 100.0; 
                    }
                };

                // 6. 计算距离
                let dist = self.vector_distance(&v_truth, v_claim);
                
                if dist > 1e-6 { 
                    1.0 // 逻辑错误 (例如 Even != Odd)
                } else { 
                    0.0 // 逻辑正确
                }
            },
            
            // 对于未知的动作类型，不要默认返回 0.0！
            _ => 100.0, 
        }
    }

    fn vector_distance(&self, v1: &Matrix, v2: &Matrix) -> f64 {
        if v1.rows != v2.rows || v1.cols != v2.cols {
            return 100.0; // 维度不匹配惩罚
        }
        
        let mut sum_sq = 0.0;
        for i in 0..v1.data.len() {
            let diff = v1.data[i] - v2.data[i];
            sum_sq += diff * diff;
        }
        sum_sq
    }
}
