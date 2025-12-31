use std::collections::HashMap;
use crate::dsl::schema::ProofAction;
use crate::dsl::math_kernel::Matrix;

/// STP (Semi-Tensor Product) Context
/// 这是一个纯粹的“逻辑物理引擎”。它不关心符号的含义，只关心矩阵的运算。
/// Truth is just a zero-energy state in vector space.
pub struct STPContext {
    // 状态空间：符号 -> 逻辑向量 (e.g., "n" -> [1, 0]^T)
    state: HashMap<String, Matrix>,
    
    // 规则空间：定理ID -> 结构矩阵 (e.g., "ModAdd" -> M_add)
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

    /// 初始化逻辑公理的矩阵表示 (Matrix Encoding)
    fn init_operators(&mut self) {
        // 定义 Z2 (Odd/Even) 上的加法结构矩阵 M_add
        // 编码约定: [1, 0]^T = Even, [0, 1]^T = Odd
        // 运算表:
        // Even(1,0) + Even(1,0) -> Even(1,0) | Col 1: [1, 0]
        // Even(1,0) + Odd(0,1)  -> Odd(0,1)  | Col 2: [0, 1]
        // Odd(0,1)  + Even(1,0) -> Odd(0,1)  | Col 3: [0, 1]
        // Odd(0,1)  + Odd(0,1)  -> Even(1,0) | Col 4: [1, 0]
        //
        // Matrix M_add (2 x 4):
        // [ 1.0, 0.0, 0.0, 1.0 ]
        // [ 0.0, 1.0, 1.0, 0.0 ]
        let m_add = Matrix::new(2, 4, vec![
            1.0, 0.0, 0.0, 1.0, 
            0.0, 1.0, 1.0, 0.0  
        ]);
        
        self.operators.insert("ModAdd".to_string(), m_add);

        // 可以在此扩展更多算子，如 M_mul, M_neg 等
    }

    /// 核心能量计算函数 (The Violation Function)
    /// 完全基于矩阵运算，无硬编码逻辑。
    pub fn calculate_energy(&mut self, action: &ProofAction) -> f64 {
        match action {
            ProofAction::Define { symbol, hierarchy_path } => {
                // 1. 向量化 (Vectorization)
                // 将离散符号映射到向量空间
                let val_type = hierarchy_path.last().map(|s| s.as_str()).unwrap_or("");
                
                let vector = if val_type == "Odd" {
                    Matrix::new(2, 1, vec![0.0, 1.0]) // Odd Basis
                } else if val_type == "Even" {
                    Matrix::new(2, 1, vec![1.0, 0.0]) // Even Basis
                } else {
                    // 对于未定义类型，暂时分配零向量或随机向量
                    // 在严格模式下应 Panic 或返回高能量
                    Matrix::new(2, 1, vec![0.5, 0.5]) 
                };
                
                self.state.insert(symbol.clone(), vector);
                0.0 // 定义动作本身不产生逻辑违背
            },
            
            ProofAction::Apply { theorem_id, inputs, output_symbol } => {
                // 2. 获取输入张量
                // 如果输入不存在，视为严重逻辑断裂 (Energy = 10.0)
                let v1 = match self.state.get(&inputs[0]) {
                    Some(v) => v,
                    None => return 10.0, 
                };
                
                // 处理二元运算
                let v_input_tensor = if inputs.len() > 1 {
                    let v2 = match self.state.get(&inputs[1]) {
                        Some(v) => v,
                        None => return 10.0,
                    };
                    // STP 关键步骤：构造输入空间的张量积 x (x) y
                    // 实际上 math_kernel::stp 会处理维度，这里我们显式做 Kronecker 积
                    // 因为输入是向量，STP 退化为 Kronecker
                    v1.kron(v2) 
                } else {
                    v1.clone()
                };

                // 3. 获取算子矩阵
                let m_op = match self.operators.get(theorem_id) {
                    Some(m) => m,
                    None => return 10.0, // 调用了不存在的定理
                };

                // 4. 物理推演 (Physical Inference)
                // V_truth = M_op * (V_in1 (x) V_in2)
                // 这里使用 matmul，因为输入维度已经通过 kron 对齐了
                // 如果涉及降维，应使用 stp
                let v_truth = match m_op.matmul(&v_input_tensor) {
                    Ok(v) => v,
                    Err(_) => return 10.0, // 维度不匹配（类型错误）
                };

                // 5. 获取声明结果 (The Claim)
                let v_claim = match self.state.get(output_symbol) {
                    Some(v) => v,
                    None => return 10.0, // 输出了未定义的符号
                };

                // 6. 计算能量违背 (Violation Calculation)
                // Energy = || V_truth - V_claim ||^2
                // 在布尔逻辑中，这通常是 0.0 或 2.0 (如果正交)
                let dist = self.vector_distance(&v_truth, v_claim);
                
                // 返回二值化的能量 (0 或 1)
                if dist > 1e-6 { 1.0 } else { 0.0 }
            },
            
            _ => 0.0,
        }
    }

    /// 辅助：计算两个向量的欧几里得距离平方
    fn vector_distance(&self, v1: &Matrix, v2: &Matrix) -> f64 {
        if v1.rows != v2.rows || v1.cols != v2.cols {
            return 100.0; // 维度不同，距离无穷大
        }
        
        let mut sum_sq = 0.0;
        for i in 0..v1.data.len() {
            let diff = v1.data[i] - v2.data[i];
            sum_sq += diff * diff;
        }
        sum_sq
    }
}
