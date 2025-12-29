// Evolver STP Bridge
// 这个模块实现了代数状态空间 (Algebraic State Space) 与证明 DSL 的对接。
// 它负责计算每个证明步骤的 "能量值" (Energy/Cost)，用于 VAPO 优化。

use std::collections::HashMap;
use crate::dsl::schema::ProofAction;

// -------------------------------------------------------------------------
// 1. 轻量级 Tensor 实现 (用于模拟 STP 运算)
// -------------------------------------------------------------------------
// 在实际项目中，这应该引用 crate::topology::tensor::Tensor
#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    pub data: Vec<f64>,
    pub shape: Vec<usize>, // [rows, cols] 通常用于矩阵
}

impl Tensor {
    /// 创建一个新的张量 (矩阵)
    pub fn new(data: Vec<f64>, shape: Vec<usize>) -> Self {
        assert_eq!(data.len(), shape.iter().product(), "Data length mismatch shape");
        Tensor { data, shape }
    }

    /// 创建一个逻辑状态向量 \delta_k^i
    /// k: 维度 (例如 2 代表二进制状态)
    /// i: 索引 (1-based, 1..=k)
    pub fn delta(k: usize, i: usize) -> Self {
        let mut data = vec![0.0; k];
        if i > 0 && i <= k {
            data[i - 1] = 1.0;
        }
        Tensor { data, shape: vec![k, 1] } // 列向量
    }

    /// 计算范数 (用于能量计算)
    pub fn norm(&self) -> f64 {
        self.data.iter().map(|x| x.powi(2)).sum::<f64>().sqrt()
    }

    /// 向量/矩阵减法
    pub fn sub(&self, other: &Tensor) -> Tensor {
        assert_eq!(self.shape, other.shape, "Shape mismatch in sub");
        let new_data = self.data.iter().zip(&other.data).map(|(a, b)| a - b).collect();
        Tensor { data: new_data, shape: self.shape.clone() }
    }

    /// 半张量积 (Semi-Tensor Product, STP) \ltimes
    /// 这是 STP 理论的核心：推广了普通矩阵乘法，允许维度不匹配的矩阵相乘。
    /// A (m x n) \ltimes B (p x q) -> C ( (m*t/n) x (q*t/p) ) 其中 t = lcm(n, p)
    pub fn semi_tensor_product(&self, other: &Tensor) -> Tensor {
        let (m, n) = (self.shape[0], self.shape[1]);
        let (p, q) = (other.shape[0], other.shape[1]);

        let t = num_integer::lcm(n, p);
        
        // Kronecker Product with Identity matrices to match dimensions
        // A \otimes I_(t/n)
        let a_expanded = self.kron_identity(t / n);
        // B \otimes I_(t/p)
        // 注意：如果是向量乘法通常是左乘，这里简化为标准 STP 定义
        // 为了实现 x(t+1) = L \ltimes u，我们需要处理矩阵乘向量的情况
        
        // 简化版实现：假设标准的 STP 矩阵乘法逻辑
        // 在逻辑网络中，通常 n=p 或者我们通过 Kronecker 积扩展
        // 这里为了演示，我们实现一个 naive 的 Kronecker 积作为 STP (当 n=1 或 p=1 时退化)
        // 更通用的 STP 需要重塑矩阵，这里模拟 A \ltimes B
        
        // 计算结果维度
        let new_rows = m * (t / n);
        let new_cols = q * (t / p);
        let mut new_data = vec![0.0; new_rows * new_cols];

        // 这里的完整 STP 实现比较复杂，我们用一个简化假设：
        // 假设是标准矩阵乘法扩展 (Kronecker Product 风格)
        // 如果是 M \ltimes x，且 x 是列向量，这通常等价于 (M \otimes I) * x 
        // 但在逻辑推演中，往往是 M \ltimes (x1 \otimes x2)
        
        // 为确保代码可运行，这里实现 Kronecker Product (A \otimes B)
        // 这在 u = x1 \ltimes x2 时是正确的
        for r in 0..m {
            for c in 0..n {
                let val_a = self.data[r * n + c];
                for i in 0..p {
                    for j in 0..q {
                        let val_b = other.data[i * q + j];
                        // 目标索引
                        let target_r = r * p + i;
                        let target_c = c * q + j;
                        new_data[target_r * (n * q) + target_c] = val_a * val_b;
                    }
                }
            }
        }
        
        // 注意：真正的 STP 还需要处理中间维度的约简 (Contracting)，
        // 但对于生成状态向量 u = x1 \ltimes x2 ... \ltimes xn，Kronecker 积是完全正确的。
        // 对于 y = M \ltimes u，如果 M 维度匹配，就是标准矩阵乘法。
        
        if n == p {
            // 维度匹配，回退到标准矩阵乘法
            self.matmul(other)
        } else {
             // 暂时返回 Kronecker 积结果作为 u 的构建
             Tensor { data: new_data, shape: vec![m * p, n * q] }
        }
    }

    // 标准矩阵乘法
    fn matmul(&self, other: &Tensor) -> Tensor {
         let (m, n) = (self.shape[0], self.shape[1]);
         let (p, q) = (other.shape[0], other.shape[1]);
         assert_eq!(n, p, "Matmul dimension mismatch");
         
         let mut new_data = vec![0.0; m * q];
         for i in 0..m {
             for k in 0..n {
                 let val_self = self.data[i * n + k];
                 for j in 0..q {
                     new_data[i * q + j] += val_self * other.data[k * q + j];
                 }
             }
         }
         Tensor { data: new_data, shape: vec![m, q] }
    }
    
    // Kronecker with Identity (简化辅助)
    fn kron_identity(&self, size: usize) -> Tensor {
        // ... (省略具体实现，使用上面的 semi_tensor_product 逻辑覆盖)
        self.clone() 
    }
}

// -------------------------------------------------------------------------
// 2. STP Context 与 能量计算
// -------------------------------------------------------------------------

pub struct STPContext {
    // 符号表：存储变量名到状态向量的映射
    // String -> \delta_k^i (例如 "n" -> [0, 1]^T)
    variables: HashMap<String, Tensor>,
    
    // 规则库：存储定理名到结构矩阵的映射
    // "ModAdd" -> M_add
    theorems: HashMap<String, Tensor>,

    // p-进树映射表 (Mock)
    // 路径字符串 -> 预期的 Tensor 状态 (例如 ["Odd"] -> [0, 1]^T)
    path_to_state: HashMap<String, Tensor>,
}

impl STPContext {
    pub fn new() -> Self {
        let mut ctx = STPContext {
            variables: HashMap::new(),
            theorems: HashMap::new(),
            path_to_state: HashMap::new(),
        };
        ctx.init_standard_library();
        ctx
    }

    /// 初始化一些基础的数学定理和状态定义
    fn init_standard_library(&mut self) {
        // 定义基向量
        let delta_even = Tensor::delta(2, 1); // [1, 0]
        let delta_odd = Tensor::delta(2, 2);  // [0, 1]
        
        // 注册路径映射
        self.path_to_state.insert("Number/Integer/Even".to_string(), delta_even.clone());
        self.path_to_state.insert("Number/Integer/Odd".to_string(), delta_odd.clone());
        
        // 注册定理矩阵: 奇偶加法
        // Even(1) + Even(1) = Even(1)
        // Even(1) + Odd(2)  = Odd(2)
        // Odd(2)  + Even(1) = Odd(2)
        // Odd(2)  + Odd(2)  = Even(1)
        // 对应列向量顺序: 11, 12, 21, 22 -> 输出 1, 2, 2, 1
        // 矩阵 M = [ \delta_1, \delta_2, \delta_2, \delta_1 ]
        //      = [ 1, 0, 0, 1 ]
        //        [ 0, 1, 1, 0 ]
        let m_add_data = vec![
            1.0, 0.0, 0.0, 1.0,
            0.0, 1.0, 1.0, 0.0
        ];
        let m_add = Tensor::new(m_add_data, vec![2, 4]);
        self.theorems.insert("ModAdd".to_string(), m_add);
        
        // 注册逻辑关系: Equals (2x4 matrix for boolean output)
        // True=[1,0], False=[0,1]
        // 1==1 -> T, 1==2 -> F, 2==1 -> F, 2==2 -> T
        let m_eq_data = vec![
            1.0, 0.0, 0.0, 1.0, // T row
            0.0, 1.0, 1.0, 0.0  // F row
        ];
        let m_eq = Tensor::new(m_eq_data, vec![2, 4]);
        self.theorems.insert("Equals".to_string(), m_eq);
    }

    /// 获取变量状态，如果不存在返回 None
    pub fn get_var(&self, name: &str) -> Option<&Tensor> {
        self.variables.get(name)
    }

    /// 获取逻辑真值 \delta_True
    pub fn delta_true(&self) -> Tensor {
        Tensor::delta(2, 1) // [1, 0] defined as True
    }

    /// 计算一个 DSL 动作的 "能量值" (Energy/Cost)
    /// Energy = 0.0 表示完全符合逻辑/定义
    /// Energy > 0.0 表示存在逻辑违规或状态不匹配
    pub fn calculate_energy(&mut self, action: &ProofAction) -> f64 {
        match action {
            ProofAction::Define { symbol, hierarchy_path } => {
                // 将路径转换为字符串键 (例如 "Number/Integer/Odd")
                let path_key = hierarchy_path.join("/");
                
                // 1. 检查定义是否在允许的路径表中
                if let Some(target_state) = self.path_to_state.get(&path_key) {
                    // 2. 如果该符号已经存在，检查是否发生了 "状态漂移"
                    // (在严格证明中，Define 通常只定义一次，但在生成修正中可能用于重定义)
                    if let Some(current_state) = self.variables.get(symbol) {
                        return current_state.sub(target_state).norm();
                    } else {
                        // 新定义，注册变量
                        self.variables.insert(symbol.clone(), target_state.clone());
                        return 0.0; // 合法定义
                    }
                } else {
                    // 路径未知，能量极大 (VAPO 应该避免这种情况)
                    return 100.0; 
                }
            },

            ProofAction::Apply { theorem_id, inputs, output_symbol } => {
                // 1. 获取输入向量 u = x1 \ltimes x2 ...
                // 必须保证所有输入变量都已定义
                if inputs.is_empty() { return 10.0; } // 错误：无输入
                
                let mut u = match self.get_var(&inputs[0]) {
                    Some(v) => v.clone(),
                    None => return 5.0, // 输入未定义
                };
                
                for i in 1..inputs.len() {
                    let next_var = match self.get_var(&inputs[i]) {
                        Some(v) => v,
                        None => return 5.0,
                    };
                    u = u.semi_tensor_product(next_var);
                }
                
                // 2. 获取结构矩阵 L
                let l_matrix = match self.theorems.get(theorem_id) {
                    Some(m) => m,
                    None => return 20.0, // 定理不存在
                };
                
                // 3. 计算理论预期 x_expected = L \ltimes u
                // 这里可能需要调整 STP 实现以处理 L 与 u 的乘法
                // 在我们的简化 Tensor 中，如果 u 是列向量，直接 matmul
                let x_expected = l_matrix.matmul(&u);
                
                // 4. 处理 output_symbol
                // 在生成流中，Apply 实际上是 *推导* 出了 output_symbol。
                // 我们应该更新 Context 中的 output_symbol 状态。
                // 如果 output_symbol 已经存在（例如之前的 Define 给了它一个 conflicting 的状态），则计算能量。
                
                if let Some(existing_state) = self.variables.get(output_symbol) {
                    // 计算冲突能量
                    return existing_state.sub(&x_expected).norm();
                } else {
                    // 这是一个新的推导，接受它并更新状态
                    self.variables.insert(output_symbol.clone(), x_expected);
                    return 0.0;
                }
            },
            
            ProofAction::Assert { subject, relation, object } => {
                let val_subject = match self.get_var(subject) { Some(v) => v, None => return 5.0 };
                // Object 可能是变量名，也可能是常量名 (如 "True")，这里简化处理假定是变量
                // 实际应该检查 path_to_state 或 variables
                let val_object = match self.get_var(object) { 
                    Some(v) => v, 
                    None => match self.path_to_state.get(object) { // 尝试作为常量查找
                        Some(v) => v,
                        None => return 5.0 
                    }
                };

                let m_rel = match self.theorems.get(relation) { Some(m) => m, None => return 20.0 };
                
                // 构造 u = subject \ltimes object
                let u = val_subject.semi_tensor_product(val_object);
                
                // y_truth = M_rel \ltimes u
                let truth = m_rel.matmul(&u);
                
                // 目标是 \delta_True
                return truth.sub(&self.delta_true()).norm();
            },

            ProofAction::Branch { case_id: _, sub_proof } => {
                // 递归计算子证明的能量总和
                let mut total_energy = 0.0;
                // 注意：这里应该 Clone 一个 context 副本进入子分支，
                // 因为子分支的变量定义不应该污染主分支（除非是全局变量）
                // 为了简单，暂时用当前 context
                for step in sub_proof {
                    total_energy += self.calculate_energy(step);
                }
                total_energy
            },

            ProofAction::QED => 0.0, // 总是完美的
        }
    }
}
