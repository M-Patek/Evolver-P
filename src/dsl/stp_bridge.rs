use std::collections::HashMap;
use crate::dsl::schema::{ProofAction}; // ProofBundle is defined in schema but not used here directly yet
// [Fix 1] 正确引入 Matrix，不再指向不存在的 crypto::algebra
use crate::dsl::math_kernel::Matrix; 

/// STP (Semi-Tensor Product) Context
/// 负责维护逻辑状态并计算“能量值”（逻辑违背程度）。
pub struct STPContext {
    // 存储符号及其对应的逻辑向量值
    // 例如: "n" -> [1.0, 0.0]^T (代表 Odd)
    state: HashMap<String, Matrix>,
    
    // 结构常数矩阵缓存 (Structure Constants)
    operators: HashMap<String, Matrix>,
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

    /// 初始化常用的逻辑算子矩阵
    fn init_operators(&mut self) {
        // [Fix 2] 修复 Matrix 构造函数调用
        // 这里的 Matrix::new 需要 (rows, cols, flat_data)
        // 以前是 vec![vec![...]] 导致了类型错误
        
        // 例如：定义 Z2 上的加法结构矩阵 (ModAdd)
        // 逻辑: Even(0)+Even(0)=Even(0), Even+Odd=Odd, Odd+Even=Odd, Odd+Odd=Even
        // 向量表示(假设 Even=[1,0], Odd=[0,1]):
        // M_add = \delta_2 [1 2 2 1] -> 
        // Row 1 (Even result): 1, 0, 0, 1
        // Row 2 (Odd result):  0, 1, 1, 0
        let m_add = Matrix::new(2, 4, vec![
            1.0, 0.0, 0.0, 1.0,
            0.0, 1.0, 1.0, 0.0
        ]);
        
        self.operators.insert("ModAdd".to_string(), m_add);
    }

    /// 核心能量计算函数
    /// Energy = 0.0 表示逻辑自洽
    /// Energy > 0.0 表示存在逻辑矛盾
    pub fn calculate_energy(&mut self, action: &ProofAction) -> f64 {
        match action {
            ProofAction::Define { symbol, hierarchy_path } => {
                // 解析定义，将符号映射为向量
                // 简化逻辑: "Odd" -> [0, 1], "Even" -> [1, 0]
                // 注意：这里需要根据具体的 hierarchy_path 解析
                let val_type = hierarchy_path.last().map(|s| s.as_str()).unwrap_or("");
                
                // [Fix 2] 同样修复这里的 Matrix 构造
                let vector = if val_type == "Odd" {
                    Matrix::new(2, 1, vec![0.0, 1.0]) // Odd: Vector [0, 1]
                } else {
                    Matrix::new(2, 1, vec![1.0, 0.0]) // Even: Vector [1, 0]
                };
                
                self.state.insert(symbol.clone(), vector);
                0.0 // 定义动作本身默认为“合法”
            },
            
            ProofAction::Apply { theorem_id, inputs, output_symbol } => {
                // 验证推理的一致性
                // 逻辑: Energy = || M * (Input1 \ltimes Input2) - Output ||
                
                // 1. 获取输入向量
                let v1 = match self.state.get(&inputs[0]) {
                    Some(v) => v,
                    None => return 0.0, // 未知输入暂不惩罚
                };
                
                // 安全获取第二个输入（如果存在）
                let empty_string = "".to_string();
                let v2_key = inputs.get(1).unwrap_or(&empty_string);
                let v2 = match self.state.get(v2_key) {
                    Some(v) => v,
                    None => return 0.0,
                };
                
                // 2. 获取预期的输出向量 (即 Generator 声称的结果)
                // 注意：main.rs 里的逻辑是先 Apply 再检查，或者 Apply 包含了对输出的定义
                // 这里我们假设 output_symbol 已经被 Define 过了
                let v_claim = match self.state.get(output_symbol) {
                    Some(v) => v,
                    None => return 0.0,
                };

                // 3. 执行 STP 运算 (模拟)
                if let Some(_op_matrix) = self.operators.get(theorem_id) {
                    // 在完整实现中，这里应该是:
                    // let input_tensor = v1.stp(v2); 
                    // let derived_result = op_matrix.multiply(&input_tensor);
                    // let diff = derived_result.sub(v_claim);
                    // return diff.norm();
                    
                    // 为了让 Demo 跑通，我们在此处硬编码 "ModAdd" 的逻辑检测
                    // 因为我们还没有完整实现 Matrix 的乘法和张量积
                    if theorem_id == "ModAdd" {
                        // 假设 data[1] > 0.5 表示是 Odd (向量为 [0, 1])
                        // 使用 .get() 并处理 Option 以防止越界
                        let is_v1_odd = v1.data.get(1).copied().unwrap_or(0.0) > 0.5;
                        let is_v2_odd = v2.data.get(1).copied().unwrap_or(0.0) > 0.5;
                        
                        // 逻辑运算: Odd + Odd = Even (即 1 ^ 1 = 0)
                        // XOR 逻辑: 结果为奇数当且仅当只有一个输入是奇数
                        let should_be_odd = is_v1_odd ^ is_v2_odd; 
                        
                        let claim_is_odd = v_claim.data.get(1).copied().unwrap_or(0.0) > 0.5;
                        
                        if should_be_odd != claim_is_odd {
                            return 1.0; // 逻辑矛盾！能量激增！
                        }
                    }
                }
                0.0
            },
            
            _ => 0.0,
        }
    }
}
