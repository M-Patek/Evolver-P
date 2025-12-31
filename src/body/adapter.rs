/// 证明动作 (Proof Action)
/// 定义了 STP 引擎可以执行的逻辑操作。
/// 
/// 注意：在完整系统中，这个结构体可能应该定义在 `crate::dsl::schema` 中。
/// 这里为了演示适配器逻辑，我们在本地定义它。
#[derive(Debug, Clone, PartialEq)]
pub enum ProofAction {
    /// 定义语义: 将某个符号绑定到特定类型 (例如 "n" is "Odd")
    Define { symbol: String, value_type: String },
    /// 变换语义: 对目标应用规则 (例如 "double" "n")
    Transform { target: String, rule: String },
    /// 断言语义: 检查某个条件 (例如 "n > 0")
    Assert { condition: String },
    /// 空操作: 当路径熵不足或无法解析时
    NoOp,
}

/// STP 语义适配器 (Semantic Adapter)
/// 
/// 将 v-PuNN 的抽象数字路径映射为具体的 STP 证明动作。
/// 这是连接“潜意识”(代数路径) 和 “显意识”(逻辑符号) 的桥梁。
///
/// 映射逻辑是确定性的，由数字路径的特定位决定动作的类型和参数。
///
/// # 参数
/// * `digits` - 由 v-PuNN 生成的数字路径序列 (Vec<u64>)
///
/// # 返回
/// * `ProofAction` - 可被 STP 引擎验证的具体业务动作
pub fn path_to_proof_action(digits: &[u64]) -> ProofAction {
    // 安全检查：如果路径为空，无法产生动作
    if digits.is_empty() {
        return ProofAction::NoOp;
    }

    // 1. 动作路由 (Action Routing)
    // 使用路径的第一个数字 (Seed Digit) 来决定动作的“意图类别”。
    // 这里使用模运算将巨大的状态空间坍缩为有限的动作集。
    match digits[0] % 3 {
        0 => decode_define(&digits[1..]),
        1 => decode_transform(&digits[1..]),
        2 => decode_assert(&digits[1..]),
        _ => ProofAction::NoOp, // 理论上不可达
    }
}

/// 解码 "Define" 动作
/// 需要路径提供至少两个额外的熵源：一个用于符号，一个用于类型。
fn decode_define(context: &[u64]) -> ProofAction {
    // 熵检查
    if context.len() < 2 {
        // 如果剩下的路径太短，回退到默认的安全定义
        return ProofAction::Define { 
            symbol: "unknown".to_string(), 
            value_type: "Entity".to_string() 
        };
    }

    // 静态符号表 (实际系统中可能从知识库加载)
    let symbols = ["n", "m", "k", "x", "y", "sum", "prod"];
    let types = ["Odd", "Even", "Prime", "Integer", "Zero", "Positive"];

    // 确定性映射
    let sym_idx = (context[0] as usize) % symbols.len();
    let type_idx = (context[1] as usize) % types.len();

    ProofAction::Define {
        symbol: symbols[sym_idx].to_string(),
        value_type: types[type_idx].to_string(),
    }
}

/// 解码 "Transform" 动作
fn decode_transform(context: &[u64]) -> ProofAction {
    if context.is_empty() {
        return ProofAction::NoOp;
    }
    
    let targets = ["n", "m", "sum", "result"];
    let rules = ["increment", "square", "double", "halve", "negate"];
    
    let t_idx = (context[0] as usize) % targets.len();
    // 如果有足够的熵，使用下一个数字；否则重用第一个数字（这会降低随机性，但在边缘情况下是安全的）
    let r_idx = if context.len() > 1 {
        (context[1] as usize) % rules.len()
    } else {
        (context[0] as usize) % rules.len()
    };

    ProofAction::Transform {
        target: targets[t_idx].to_string(),
        rule: rules[r_idx].to_string(),
    }
}

/// 解码 "Assert" 动作
fn decode_assert(context: &[u64]) -> ProofAction {
    if context.is_empty() {
         return ProofAction::Assert { condition: "true".to_string() };
    }
    
    let conditions = ["n > 0", "sum == 0", "n != m", "isPrime(n)", "E == 0"];
    let idx = (context[0] as usize) % conditions.len();
    
    ProofAction::Assert {
        condition: conditions[idx].to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_action_mapping() {
        // 构造一个会导致 "Define" 的路径
        // digits[0] % 3 == 0 (e.g., 3) -> Define
        // digits[1] % 7 == 0 (e.g., 7) -> "n"
        // digits[2] % 6 == 0 (e.g., 6) -> "Odd"
        let path = vec![3, 7, 6, 100, 200];
        
        let action = path_to_proof_action(&path);
        
        if let ProofAction::Define { symbol, value_type } = action {
            assert_eq!(symbol, "n");
            assert_eq!(value_type, "Odd");
        } else {
            panic!("Expected Define action");
        }
    }
}
