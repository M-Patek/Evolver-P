// Proof DSL Parser
// 用于将生成器 (LLM) 输出的 JSON 字符串解析为严格类型的 ProofAction 序列。

use crate::dsl::schema::{ProofAction, ProofSequence};
use serde_json::Error;

pub struct ProofParser;

impl ProofParser {
    /// 从 JSON 字符串解析完整的证明序列
    pub fn parse(json_input: &str) -> Result<ProofSequence, Error> {
        serde_json::from_str(json_input)
    }

    /// 从 JSON 字符串解析单个动作 (用于流式处理)
    pub fn parse_action(json_input: &str) -> Result<ProofAction, Error> {
        serde_json::from_str(json_input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::schema::ProofAction;

    #[test]
    fn test_parse_simple_proof() {
        // 模拟生成器输出的 JSON 数据
        // 场景：证明两个奇数之和是偶数
        let json_input = r#"
        {
            "goal": "Prove sum of two odd integers is even",
            "steps": [
                {
                    "action": "Define",
                    "params": {
                        "symbol": "n",
                        "hierarchy_path": ["Number", "Integer", "Odd"]
                    }
                },
                {
                    "action": "Define",
                    "params": {
                        "symbol": "m",
                        "hierarchy_path": ["Number", "Integer", "Odd"]
                    }
                },
                {
                    "action": "Apply",
                    "params": {
                        "theorem_id": "ModAdd",
                        "inputs": ["n", "m"],
                        "output_symbol": "sum"
                    }
                },
                {
                    "action": "Assert",
                    "params": {
                        "subject": "sum",
                        "relation": "IsEven",
                        "object": "True"
                    }
                },
                {
                    "action": "QED",
                    "params": {}
                }
            ]
        }
        "#;

        let result = ProofParser::parse(json_input);
        
        // 验证解析结果
        assert!(result.is_ok());
        let proof = result.unwrap();
        
        assert_eq!(proof.goal, "Prove sum of two odd integers is even");
        assert_eq!(proof.steps.len(), 5);

        // 检查第一步：Define n
        if let ProofAction::Define { symbol, hierarchy_path } = &proof.steps[0] {
            assert_eq!(symbol, "n");
            assert_eq!(hierarchy_path, &vec!["Number", "Integer", "Odd"]);
        } else {
            panic!("Step 0 should be Define");
        }

        // 检查第三步：Apply ModAdd
        if let ProofAction::Apply { theorem_id, inputs, output_symbol } = &proof.steps[2] {
            assert_eq!(theorem_id, "ModAdd");
            assert_eq!(inputs, &vec!["n", "m"]);
            assert_eq!(output_symbol, "sum");
        } else {
            panic!("Step 2 should be Apply");
        }
        
        // 检查最后一步：QED
        assert_eq!(proof.steps[4], ProofAction::QED);
    }
}
