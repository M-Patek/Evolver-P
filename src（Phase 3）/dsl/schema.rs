/// Evolver Proof DSL - 原子动作定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofAction {
    /// 1. 实体定义 (v-PuNNs 约束)
    /// 用于引入新的数学对象，并声明其在 p-进树上的层级位置。
    /// 例如: Define("x", ["Number", "Integer", "Prime"])
    Define {
        symbol: String,
        hierarchy_path: Vec<String>, // 对应 v-PuNNs 的 Valuation 路径
    },

    /// 2. 逻辑断言 (STP 约束)
    /// 声明两个对象之间的逻辑关系。对应 STP 的状态向量积。
    /// 例如: Assert("x", "in", "Set_P")
    Assert {
        subject: String,
        relation: String, // 对应 STP 结构矩阵 L_relation
        object: String,
    },

    /// 3. 定理应用 (状态转移)
    /// 应用预定义的定理规则进行推导。这是 STP 动力学方程的核心 x(t+1) = L * u * x(t)
    /// 例如: Apply("Theorem_5.7", inputs=["x", "bias"])
    Apply {
        theorem_id: String,
        inputs: Vec<String>,
        output_symbol: String, // 推导出的新符号
    },

    /// 4. 分支探索 (拓扑结构)
    /// 对应证明树的分叉。
    Branch {
        case_id: String,
        sub_proof: Vec<ProofAction>, // 递归结构
    },

    /// 5. 目标达成
    QED,
}
