// src/body/adapter.rs
// 适配器: 负责将原始的数字信号 (Body) 解码为强类型的逻辑动作 (Logic)

use crate::dsl::schema::{ProofAction, LogicType};

/// STP 语义适配器
/// 将 v-PuNN 的抽象数字路径映射为 Schema 中定义的 ProofAction
pub fn path_to_proof_action(digits: &[u64]) -> ProofAction {
    // 安全检查
    if digits.is_empty() {
        return ProofAction::NoOp;
    }

    // 动作路由: 使用第一个数字决定动作类别
    match digits[0] % 3 {
        0 => decode_define(&digits[1..]),
        1 => decode_transform(&digits[1..]),
        2 => decode_assert(&digits[1..]),
        _ => ProofAction::NoOp,
    }
}

/// 解码 "Define" 动作
fn decode_define(context: &[u64]) -> ProofAction {
    // 熵检查
    if context.len() < 2 {
        return ProofAction::Define { 
            symbol: "unknown".to_string(), 
            value_type: LogicType::Unknown 
        };
    }

    let symbols = ["n", "m", "k", "x", "y", "sum", "prod"];
    
    // [Fix] 类型映射逻辑: 数字 -> 强类型 LogicType
    let type_idx = (context[1] as usize) % 5;
    let logic_type = match type_idx {
        0 => LogicType::Even,
        1 => LogicType::Odd,
        2 => LogicType::Prime,
        3 => LogicType::Integer,
        4 => LogicType::Integer, // 冗余映射以增加概率
        _ => LogicType::Unknown,
    };

    let sym_idx = (context[0] as usize) % symbols.len();

    ProofAction::Define {
        symbol: symbols[sym_idx].to_string(),
        value_type: logic_type,
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
