// src/dsl/stp_bridge.rs
// STP 桥接器: 负责计算“离散势垒”能量
// [Fix] 重构为严格的分级能量模型 (Tiered Energy Model)

use crate::dsl::schema::{ProofAction, LogicType, LogicValue};
use crate::dsl::math_kernel::Matrix;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

/// 能量层级 (Energy Tier)
/// 严格对应 Unified Energy Metric 文档中的定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnergyTier {
    Truth = 0,         // 逻辑完备，无任何错误
    SyntaxError = 10,  // 解析错误，生成的不是合法代码
    LogicError = 100,  // 逻辑矛盾 (例如 1 == 0)
}

impl EnergyTier {
    pub fn base_cost(&self) -> f64 {
        *self as u32 as f64
    }
}

/// 逻辑矩阵库 (The Logic Matrix Library)
struct LogicMatrix;
impl LogicMatrix {
    fn identity() -> Matrix { Matrix::identity(2) }
    fn not() -> Matrix { Matrix::new(2, 2, vec![0.0, 1.0, 1.0, 0.0]) }
    fn always_false() -> Matrix { Matrix::new(2, 2, vec![1.0, 1.0, 0.0, 0.0]) }
    // is_prime 在二值逻辑下简化为恒等 (奇数=True/Prime, 偶数=False)
    // 实际应更复杂，但为 Demo 保持简单
    fn is_prime() -> Matrix { Self::identity() } 
    
    // 比较算子
    fn equal() -> Matrix { Matrix::new(2, 4, vec![0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0]) }
    fn not_equal() -> Matrix { Matrix::new(2, 4, vec![1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0]) }
    fn greater_than() -> Matrix { Matrix::new(2, 4, vec![1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0]) }
    
    // 逻辑算子
    fn and() -> Matrix { Matrix::new(2, 4, vec![1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]) }
    
    // 算术算子 (Z2 域)
    fn add() -> Matrix { Self::not_equal() } // XOR
    fn mul() -> Matrix { Self::and() }       // AND
}

pub struct STPContext {
    symbol_table: HashMap<String, LogicValue>,
    // 不再直接累加 energy float，而是记录最严重的错误层级
    pub max_tier: EnergyTier,
    // 辅助的距离惩罚 (用于同层级内的微调)
    pub dist_penalty: f64,
}

impl STPContext {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            max_tier: EnergyTier::Truth, // 默认为 Truth，发现错误则升级
            dist_penalty: 0.0,
        }
    }

    /// 计算势垒能量
    /// 返回值 = BaseCost(Tier) + clamped(penalty)
    pub fn calculate_barrier(&mut self, actions: &[ProofAction]) -> f64 {
        self.reset();

        if actions.is_empty() {
            // 空动作视为语法错误 (没有生成任何有效逻辑)
            return EnergyTier::SyntaxError.base_cost() + 1.0;
        }

        // 初始化常量
        self.symbol_table.insert("0".to_string(), LogicValue::Vector(vec![1.0, 0.0]));
        self.symbol_table.insert("1".to_string(), LogicValue::Vector(vec![0.0, 1.0]));

        for action in actions {
            match action {
                ProofAction::Define { symbol, value_type } => {
                    let val = match value_type {
                        LogicType::Even => LogicValue::Vector(vec![1.0, 0.0]),
                        LogicType::Odd => LogicValue::Vector(vec![0.0, 1.0]),
                        _ => LogicValue::Vector(vec![0.5, 0.5]),
                    };
                    self.symbol_table.insert(symbol.clone(), val);
                }
                
                ProofAction::Transform { target, rule } => {
                    self.apply_transform(target, rule);
                }

                ProofAction::Assert { condition } => {
                    self.evaluate_assertion(condition);
                }
                
                ProofAction::NoOp => {}
            }
            
            // 如果已经发生了 LogicError，我们可以提前终止或者继续跑以收集更多 penalty
            // 为了 VAPO，继续跑可能更好
        }

        // 归一化 penalty，确保它不会超过层级间距 (10.0)
        // penalty range [0, 9.9]
        let clamped_penalty = (self.dist_penalty * 0.1).min(9.9);
        
        self.max_tier.base_cost() + clamped_penalty
    }

    fn reset(&mut self) {
        self.symbol_table.clear();
        self.max_tier = EnergyTier::Truth;
        self.dist_penalty = 0.0;
    }

    fn escalate_tier(&mut self, tier: EnergyTier) {
        if tier > self.max_tier {
            self.max_tier = tier;
        }
    }

    fn apply_transform(&mut self, target: &str, rule: &str) {
        let current_vec_data = if let Some(LogicValue::Vector(v)) = self.symbol_table.get(target) {
            v.clone()
        } else {
            self.escalate_tier(EnergyTier::SyntaxError); // 引用未定义变量
            self.dist_penalty += 1.0;
            return;
        };

        let input_matrix = Matrix::new(2, 1, current_vec_data);
        
        let rule_matrix = match rule {
            "increment" => Some(LogicMatrix::not()),
            "negate" => Some(LogicMatrix::identity()),
            "double" => Some(LogicMatrix::always_false()),
            "square" => Some(LogicMatrix::identity()),
            "halve" => Some(Matrix::new(2, 2, vec![0.5, 0.5, 0.5, 0.5])),
            _ => None,
        };

        if let Some(m) = rule_matrix {
            let result_matrix = m.stp(&input_matrix);
            self.symbol_table.insert(target.to_string(), LogicValue::Vector(result_matrix.data));
        } else {
            self.escalate_tier(EnergyTier::SyntaxError); // 未知规则
            self.dist_penalty += 0.5;
        }
    }

    fn evaluate_assertion(&mut self, condition: &str) {
        let mut parser = AssertionParser::new(condition, &self.symbol_table);
        match parser.parse_and_evaluate() {
            Ok(result_vec) => {
                // 断言期望结果为真 [0, 1]^T
                let target = Matrix::new(2, 1, vec![0.0, 1.0]);
                let dist = self.matrix_dist_sq(&result_vec, &target);
                
                if dist > 0.01 {
                    // 逻辑计算正确执行了，但是结果与预期不符 -> 逻辑错误
                    self.escalate_tier(EnergyTier::LogicError);
                    self.dist_penalty += dist * 10.0; 
                }
            },
            Err(_) => {
                // 解析失败 -> 语法错误
                self.escalate_tier(EnergyTier::SyntaxError);
                self.dist_penalty += 1.0;
            }
        }
    }

    fn matrix_dist_sq(&self, a: &Matrix, b: &Matrix) -> f64 {
        a.data.iter().zip(b.data.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum()
    }
}

// --- Tokenizer and AssertionParser Implementation ---

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Identifier(String), Number(String),
    Eq, Neq, Gt, Plus, Star, LParen, RParen, IsPrime, Eof
}

struct Tokenizer<'a> { chars: Peekable<Chars<'a>> }
impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self { Self { chars: input.chars().peekable() } }
    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        match self.chars.next() {
            Some(c) => match c {
                '(' => Token::LParen, ')' => Token::RParen,
                '+' => Token::Plus, '*' => Token::Star,
                '=' => if let Some('=') = self.chars.peek() { self.chars.next(); Token::Eq } else { Token::Eof },
                '!' => if let Some('=') = self.chars.peek() { self.chars.next(); Token::Neq } else { Token::Eof },
                '>' => Token::Gt,
                _ if c.is_alphabetic() => {
                    let mut ident = String::new(); ident.push(c);
                    while let Some(&nc) = self.chars.peek() {
                        if nc.is_alphanumeric() || nc == '_' { ident.push(self.chars.next().unwrap()); } else { break; }
                    }
                    if ident == "isPrime" { Token::IsPrime } else { Token::Identifier(ident) }
                },
                _ if c.is_numeric() => {
                    let mut num = String::new(); num.push(c);
                    Token::Number(num)
                },
                _ => Token::Eof,
            },
            None => Token::Eof,
        }
    }
    fn skip_whitespace(&mut self) { while let Some(&c) = self.chars.peek() { if c.is_whitespace() { self.chars.next(); } else { break; } } }
}

struct AssertionParser<'a> { tokenizer: Tokenizer<'a>, current_token: Token, symbols: &'a HashMap<String, LogicValue> }
impl<'a> AssertionParser<'a> {
    fn new(input: &'a str, symbols: &'a HashMap<String, LogicValue>) -> Self {
        let mut tokenizer = Tokenizer::new(input); let current_token = tokenizer.next_token();
        Self { tokenizer, current_token, symbols }
    }
    fn advance(&mut self) { self.current_token = self.tokenizer.next_token(); }
    fn parse_and_evaluate(&mut self) -> Result<Matrix, String> { self.parse_expression() }
    fn parse_expression(&mut self) -> Result<Matrix, String> {
        let mut left = self.parse_arithmetic()?;
        loop {
            let op_matrix = match self.current_token {
                Token::Eq => LogicMatrix::equal(), Token::Neq => LogicMatrix::not_equal(), Token::Gt => LogicMatrix::greater_than(), _ => break,
            };
            self.advance(); let right = self.parse_arithmetic()?;
            let input = left.stp(&right); left = op_matrix.stp(&input);
        }
        Ok(left)
    }
    fn parse_arithmetic(&mut self) -> Result<Matrix, String> {
        let mut left = self.parse_factor()?;
        loop {
            let op_matrix = match self.current_token { Token::Plus => LogicMatrix::add(), Token::Star => LogicMatrix::mul(), _ => break };
            self.advance(); let right = self.parse_factor()?;
            let input = left.stp(&right); left = op_matrix.stp(&input);
        }
        Ok(left)
    }
    fn parse_factor(&mut self) -> Result<Matrix, String> {
        match self.current_token.clone() {
            Token::Identifier(name) => { self.advance(); self.symbols.get(&name).map(|v| if let LogicValue::Vector(vec) = v { Matrix::new(2, 1, vec.clone()) } else { Matrix::new(2,1,vec![0.0,0.0]) }).ok_or("Undefined".to_string()) },
            Token::Number(val) => { self.advance(); Ok(if val == "0" { Matrix::new(2, 1, vec![1.0, 0.0]) } else { Matrix::new(2, 1, vec![0.0, 1.0]) }) },
            Token::IsPrime => { self.advance(); self.advance(); let expr = self.parse_expression()?; self.advance(); Ok(LogicMatrix::is_prime().stp(&expr)) },
            Token::LParen => { self.advance(); let expr = self.parse_expression()?; self.advance(); Ok(expr) },
            _ => Err("Unexpected".into()),
        }
    }
}
