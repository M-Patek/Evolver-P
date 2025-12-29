// src/dsl/stp_bridge.rs
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
        if data.len() != shape.iter().product() {
            // 简单容错或 panic，实际应返回 Result
            panic!("Data length mismatch shape: len={} vs product={}", data.len(), shape.iter().product::<usize>());
        }
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
    pub fn semi_tensor_product(&self, other: &Tensor) -> Tensor {
        let (m, n) = (self.shape[0], self.shape[1]);
        let (p, q) = (other.shape[0], other.shape[1]);

        // STP 简化逻辑: Kronecker Product
        let new_rows = m * p;
        let new_cols = n * q;
        let mut new_data = vec![0.0; new_rows * new_cols];

        for r in 0..m {
            for c in 0..n {
                let val_a = self.data[r * n + c];
                for i in 0..p {
                    for j in 0..q {
                        let val_b = other.data[i * q + j];
                        let target_r = r * p + i;
                        let target_c = c * q + j;
                        new_data[target_r * new_cols + target_c] = val_a * val_b;
                    }
                }
            }
        }
        
        // 如果维度匹配，退化为普通矩阵乘法 (用于 y = M * u)
        if n == p {
            self.matmul(other)
        } else {
             Tensor { data: new_data, shape: vec![new_rows, new_cols] }
        }
    }

    // 标准矩阵乘法
    fn matmul(&self, other: &Tensor) -> Tensor {
         let (m, n) = (self.shape[0], self.shape[1]);
         let (p, q) = (other.shape[0], other.shape[1]);
         if n != p {
             // 维度不匹配时，返回一个空张量或做某种 fallback
             // 这里为了演示稳定性，panic
             panic!("Matmul dimension mismatch: {}x{} vs {}x{}", m, n, p, q);
         }
         
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
        self.clone() // Placeholder
    }
}

// -------------------------------------------------------------------------
// 2. STP Context 与 能量计算 (Refactored for Pure Evaluation)
// -------------------------------------------------------------------------

#[derive(Clone)]
pub struct STPContext {
    // 符号表：存储变量名到状态向量的映射
    pub variables: HashMap<String, Tensor>,
    
    // 规则库：存储定理名到结构矩阵的映射
    theorems: HashMap<String, Tensor>,

    // p-进树映射表
    path_to_state: HashMap<String, Tensor>,

    // 历史栈：用于 Snapshot / Rollback
    history_stack: Vec<HashMap<String, Tensor>>,
}

impl STPContext {
    pub fn new() -> Self {
        let mut ctx = STPContext {
            variables: HashMap::new(),
            theorems: HashMap::new(),
            path_to_state: HashMap::new(),
            history_stack: Vec::new(),
        };
        ctx.init_standard_library();
        ctx
    }

    /// 初始化一些基础的数学定理和状态定义
    fn init_standard_library(&mut self) {
        // 定义基向量
        let delta_even = Tensor::delta(2, 1); // [1, 0]
        let delta_odd = Tensor::delta(2, 2);  // [0, 1]
        
        self.path_to_state.insert("Number/Integer/Even".to_string(), delta_even.clone());
        self.path_to_state.insert("Number/Integer/Odd".to_string(), delta_odd.clone());
        
        // ModAdd: Even(1)+Even(1)=1, Even(1)+Odd(2)=2, Odd(2)+Even(1)=2, Odd(2)+Odd(2)=1
        let m_add_data = vec![
            1.0, 0.0, 0.0, 1.0,
            0.0, 1.0, 1.0, 0.0
        ];
        let m_add = Tensor::new(m_add_data, vec![2, 4]);
        self.theorems.insert("ModAdd".to_string(), m_add);
        
        // Equals
        let m_eq_data = vec![
            1.0, 0.0, 0.0, 1.0, // T row
            0.0, 1.0, 1.0, 0.0  // F row
        ];
        let m_eq = Tensor::new(m_eq_data, vec![2, 4]);
        self.theorems.insert("Equals".to_string(), m_eq);
    }

    pub fn get_var(&self, name: &str) -> Option<&Tensor> {
        self.variables.get(name)
    }

    pub fn delta_true(&self) -> Tensor {
        Tensor::delta(2, 1) // [1, 0] defined as True
    }

    // =====================================================================
    // State Management (Rollback / Snapshot)
    // =====================================================================

    /// 创建当前状态的快照
    pub fn snapshot(&mut self) {
        self.history_stack.push(self.variables.clone());
    }

    /// 回滚到上一个快照
    pub fn rollback(&mut self) {
        if let Some(prev_vars) = self.history_stack.pop() {
            self.variables = prev_vars;
        }
    }

    /// 确认状态变更 (如果需要显式清理历史，可以在这里做)
    pub fn commit_history(&mut self) {
        self.history_stack.clear();
    }

    // =====================================================================
    // Pure Evaluation (无副作用)
    // =====================================================================

    /// 计算动作能量（纯函数，不修改状态）
    /// 如果需要模拟连续的多步动作，建议先 clone 或 snapshot 上下文
    pub fn calculate_energy(&self, action: &ProofAction) -> f64 {
        self.evaluate_internal(action, &self.variables)
    }

    /// 内部计算逻辑，接收一个变量表引用，以便处理递归分支的模拟
    fn evaluate_internal(&self, action: &ProofAction, current_vars: &HashMap<String, Tensor>) -> f64 {
        match action {
            ProofAction::Define { symbol, hierarchy_path } => {
                let path_key = hierarchy_path.join("/");
                if let Some(target_state) = self.path_to_state.get(&path_key) {
                    if let Some(current_state) = current_vars.get(symbol) {
                        return current_state.sub(target_state).norm();
                    } else {
                        // 新定义在纯评估模式下总是合法的 (Energy = 0)
                        // 因为我们还没 Commit
                        return 0.0;
                    }
                } else {
                    return 100.0; // Path not found
                }
            },

            ProofAction::Apply { theorem_id, inputs, output_symbol } => {
                if inputs.is_empty() { return 10.0; } 
                
                // 1. 获取输入 u
                let mut u = match current_vars.get(&inputs[0]) {
                    Some(v) => v.clone(),
                    None => return 5.0, // 输入未定义
                };
                
                for i in 1..inputs.len() {
                    let next_var = match current_vars.get(&inputs[i]) {
                        Some(v) => v,
                        None => return 5.0,
                    };
                    u = u.semi_tensor_product(next_var);
                }
                
                // 2. 结构矩阵 L
                let l_matrix = match self.theorems.get(theorem_id) {
                    Some(m) => m,
                    None => return 20.0, 
                };
                
                // 3. 预期 x
                let x_expected = l_matrix.matmul(&u);
                
                // 4. 检查冲突
                if let Some(existing_state) = current_vars.get(output_symbol) {
                    return existing_state.sub(&x_expected).norm();
                } else {
                    return 0.0; // 新推导合法
                }
            },
            
            ProofAction::Assert { subject, relation, object } => {
                let val_subject = match current_vars.get(subject) { Some(v) => v, None => return 5.0 };
                
                let val_object = match current_vars.get(object) { 
                    Some(v) => v, 
                    None => match self.path_to_state.get(object) { 
                        Some(v) => v,
                        None => return 5.0 
                    }
                };

                let m_rel = match self.theorems.get(relation) { Some(m) => m, None => return 20.0 };
                let u = val_subject.semi_tensor_product(val_object);
                let truth = m_rel.matmul(&u);
                
                return truth.sub(&self.delta_true()).norm();
            },

            ProofAction::Branch { case_id: _, sub_proof } => {
                // 对于分支，我们需要模拟状态的演变
                // 因为 sub_proof 中的步骤是有序依赖的
                let mut temp_vars = current_vars.clone();
                let mut total_energy = 0.0;

                for step in sub_proof {
                    // 递归计算每一步的能量
                    let step_energy = self.evaluate_internal(step, &temp_vars);
                    total_energy += step_energy;

                    // 在临时环境中应用该步骤，以便下一步能看到变量的变化
                    // 注意：我们这里复用了 update_vars_internal 逻辑的简化版
                    self.simulate_update(&mut temp_vars, step);
                }
                total_energy
            },

            ProofAction::QED => 0.0,
        }
    }

    /// 辅助：在模拟环境中更新变量 (仅用于 Branch 内部模拟)
    fn simulate_update(&self, vars: &mut HashMap<String, Tensor>, action: &ProofAction) {
        match action {
            ProofAction::Define { symbol, hierarchy_path } => {
                let path_key = hierarchy_path.join("/");
                if let Some(target_state) = self.path_to_state.get(&path_key) {
                    vars.insert(symbol.clone(), target_state.clone());
                }
            },
            ProofAction::Apply { theorem_id, inputs, output_symbol } => {
                // 简化逻辑：假设 theorem_id 存在且输入有效 (因为 evaluate 已经算过 energy 了)
                // 这里只做尽可能的状态推进
                if let Some(l_matrix) = self.theorems.get(theorem_id) {
                    // 构建 u (需要输入存在)
                    let mut valid = true;
                    // ... (省略繁琐的构建 u 代码，实际应复用逻辑) ...
                    // 为简化，若输入存在则更新 output
                    // 在完整实现中应提取公共的 compute_output 方法
                }
            },
             _ => {}
        }
    }

    // =====================================================================
    // State Mutation (应用变更)
    // =====================================================================

    /// 提交动作：真正修改 Context 状态
    pub fn commit_action(&mut self, action: &ProofAction) {
        match action {
            ProofAction::Define { symbol, hierarchy_path } => {
                let path_key = hierarchy_path.join("/");
                if let Some(target_state) = self.path_to_state.get(&path_key) {
                    self.variables.insert(symbol.clone(), target_state.clone());
                }
            },
            ProofAction::Apply { theorem_id, inputs, output_symbol } => {
                // 重新计算并存储结果
                // 注意：这里假设动作已经是低能量的 (Valid)，所以直接计算并覆盖/插入
                if inputs.is_empty() { return; }
                let mut u = match self.variables.get(&inputs[0]) {
                    Some(v) => v.clone(),
                    None => return,
                };
                for i in 1..inputs.len() {
                    if let Some(next_var) = self.variables.get(&inputs[i]) {
                        u = u.semi_tensor_product(next_var);
                    }
                }
                if let Some(l_matrix) = self.theorems.get(theorem_id) {
                    let x_new = l_matrix.matmul(&u);
                    self.variables.insert(output_symbol.clone(), x_new);
                }
            },
            _ => {} // Assert, QED, Branch (Branch 应该递归 commit)
        }
    }

    /// 便捷方法：计算并应用 (Legacy Support)
    /// 返回能量。如果能量为 0，则状态会被更新。
    pub fn apply_check(&mut self, action: &ProofAction) -> f64 {
        let e = self.calculate_energy(action);
        if e < 1e-6 {
            self.commit_action(action);
        }
        e
    }
}
