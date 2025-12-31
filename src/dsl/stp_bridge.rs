use crate::dsl::schema::{ProofAction, LogicType, LogicValue};
use std::collections::HashMap;

/// STP (Semi-Tensor Product) 逻辑验证引擎上下文
///
/// 职责：模拟逻辑物理场。计算逻辑动作序列的“能量”。
///
/// 核心假设：
/// 1. 逻辑一致性 = 零能量 (E = 0)
/// 2. 逻辑矛盾 = 高能量 (E > 0)
/// 3. 语法/引用错误 = 惩罚能量
pub struct STPContext {
    // 符号表：存储变量名及其当前的逻辑值（以向量形式存储，模拟 STP 向量空间）
    symbol_table: HashMap<String, LogicValue>,
    // 累积的逻辑违规能量
    energy: f64,
}

impl STPContext {
    /// 创建一个新的 STP 上下文
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            energy: 0.0,
        }
    }

    /// 执行一系列动作并计算总能量
    ///
    /// 这是 VAPO 优化器的目标函数。
    pub fn calculate_energy(&mut self, actions: &[ProofAction]) -> f64 {
        self.energy = 0.0; // 重置能量
        self.symbol_table.clear(); // 清空状态

        // 惩罚空路径：没有逻辑也是一种错误
        if actions.is_empty() {
            return 1000.0; 
        }

        for action in actions {
            match action {
                ProofAction::Define { symbol, initial_type } => {
                    // 将类型映射为 STP 向量
                    // 2维逻辑空间 (Parity Logic):
                    // Even = [1, 0]^T
                    // Odd  = [0, 1]^T
                    let val = match initial_type {
                        LogicType::Even => LogicValue::Vector(vec![1.0, 0.0]),
                        LogicType::Odd => LogicValue::Vector(vec![0.0, 1.0]),
                        // 暂不支持其他类型，视为标量 0
                        _ => LogicValue::Scalar(0.0),
                    };
                    self.symbol_table.insert(symbol.clone(), val);
                }
                
                ProofAction::Apply { .. } => {
                    // 占位符：用于显式的函数调用步骤。
                    // 在当前的 Adapter 实现中，函数调用隐含在 Assert 中。
                    // 未来可以在这里实现具体的矩阵乘法 M \ltimes V。
                }

                ProofAction::Assert { condition } => {
                    self.evaluate_assertion(condition);
                }
            }
        }

        self.energy
    }

    /// 评估断言的能量
    ///
    /// 当前简化实现：解析字符串 "(n + m) is Type"，并在内部计算真值。
    fn evaluate_assertion(&mut self, condition: &str) {
        // 1. 简单的字符串解析 (Parser 逻辑硬编码以提高性能)
        // 期望格式: "(n + m) is Even" 或 "(n + m) is Odd"
        
        // 尝试获取 n 和 m 的向量值
        let n_vec = self.get_vector("n");
        let m_vec = self.get_vector("m");

        if let (Some(n), Some(m)) = (n_vec, m_vec) {
            // 2. 模拟 STP 矩阵运算: Modulo 2 Addition (XOR)
            // 物理真值计算：
            // n=[1,0](Even), m=[1,0](Even) -> Sum=[1,0](Even)
            // n=[1,0](Even), m=[0,1](Odd)  -> Sum=[0,1](Odd)
            // n=[0,1](Odd),  m=[0,1](Odd)  -> Sum=[1,0](Even)
            
            let n_is_odd = n[1] > 0.5;
            let m_is_odd = m[1] > 0.5;
            
            // 逻辑加法 (XOR)
            let sum_is_odd = n_is_odd ^ m_is_odd;

            // 3. 解析期望值 (Claimed Truth)
            let expected_odd = if condition.contains("Odd") {
                true
            } else if condition.contains("Even") {
                false
            } else {
                // 无法识别的类型断言
                self.energy += 50.0;
                return;
            };

            // 4. 计算能量 (Violation)
            // 如果物理计算结果与声明结果不符，能量激增
            if sum_is_odd != expected_odd {
                // 逻辑矛盾！
                self.energy += 100.0;
            } else {
                // 逻辑自洽，能量不变 (+0.0)
            }
        } else {
            // 引用错误：变量未定义
            self.energy += 50.0;
        }
    }

    fn get_vector(&self, symbol: &str) -> Option<&Vec<f64>> {
        match self.symbol_table.get(symbol) {
            Some(LogicValue::Vector(v)) => Some(v),
            _ => None,
        }
    }
}
