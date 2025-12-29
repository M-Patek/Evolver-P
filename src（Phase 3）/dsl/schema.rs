// Evolver Proof DSL Schema Definition
// 这是一个严格类型的中间表示 (IR)，连接生成器 (Generator) 与约束器 (Constraint Checker)。
// 每一个 Action 都能被无歧义地编译为 STP 矩阵运算或 p-进树跳转。

use serde::{Deserialize, Serialize};

/// 证明动作的根枚举
/// 所有的证明步骤必须是以下原子操作之一
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "params")]
pub enum ProofAction {
    /// 1. 实体定义 (v-PuNNs 约束)
    /// 引入一个新的数学对象，并声明其在概念层级树 (p-adic Tree) 上的位置。
    /// 
    /// # 示例
    /// ```json
    /// { "action": "Define", "params": { "symbol": "n", "hierarchy_path": ["Number", "Integer", "Odd"] } }
    /// ```
    /// # 约束检查
    /// - 计算 symbol 的 p-进嵌入坐标是否位于 path 指定的 p-adic Ball 内。
    Define {
        symbol: String,
        hierarchy_path: Vec<String>, // 对应 v-PuNNs 的 Valuation 路径，例如 ["Entity", "MathObj", "Integer"]
    },

    /// 2. 逻辑断言 (STP 约束)
    /// 声明两个或多个对象之间的逻辑关系。对应 STP 的状态向量积验证。
    /// 
    /// # 示例
    /// ```json
    /// { "action": "Assert", "params": { "subject": "sum", "relation": "Equals", "object": "2k" } }
    /// ```
    /// # 约束检查
    /// - 计算 y_truth = M_relation \ltimes x_subject \ltimes x_object
    /// - 检查 || y_truth - \delta_True || == 0
    Assert {
        subject: String,
        relation: String, // 对应 STP 结构矩阵 L_relation 的 ID，例如 "Equals", "SubsetOf"
        object: String,
    },

    /// 3. 定理应用 (状态转移)
    /// 应用预定义的定理规则进行推导。这是 STP 动力学方程的核心 x(t+1) = L \ltimes u(t)。
    /// 
    /// # 示例
    /// ```json
    /// { "action": "Apply", "params": { "theorem_id": "ModAdd", "inputs": ["n", "m"], "output_symbol": "sum" } }
    /// ```
    /// # 约束检查
    /// - 计算 x_expected = M_theorem \ltimes x_input1 \ltimes x_input2 ...
    /// - 比较 x_expected 与 output_symbol 的实际状态，计算能量 E。
    Apply {
        theorem_id: String, // 定理/规则 ID，例如 "Theorem_5.7", "ModAdd"
        inputs: Vec<String>, // 输入符号列表
        output_symbol: String, // 推导出的新符号名
    },

    /// 4. 分支探索 (拓扑结构)
    /// 对应证明树的分叉，用于分类讨论 (Case Analysis)。
    /// 
    /// # 示例
    /// ```json
    /// { "action": "Branch", "params": { "case_id": "n_is_even", "sub_proof": [ ... ] } }
    /// ```
    Branch {
        case_id: String,
        sub_proof: Vec<ProofAction>, // 递归结构
    },

    /// 5. 目标达成
    /// 标记证明结束。
    QED,
}

/// 完整的证明序列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSequence {
    pub goal: String, // 证明目标描述，例如 "Prove odd + odd = even"
    pub steps: Vec<ProofAction>,
}
