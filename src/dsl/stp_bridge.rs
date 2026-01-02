// src/dsl/stp_bridge.rs
// STP 桥接器: 消费强类型的 ProofAction 并计算系统能量
// 包含完整的逻辑矩阵库 (The Logic Matrix Library) 和 严格的递归下降解析器 (Strict Parser) 喵！

use crate::dsl::schema::{ProofAction, LogicType, LogicValue};
use crate::dsl::math_kernel::Matrix;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

/// 逻辑矩阵库：提供所有标准逻辑算子的结构矩阵 M
/// 基于 Z_2 域 (奇偶/真假逻辑):
/// 状态 0 (False/Even) -> [1, 0]^T
/// 状态 1 (True/Odd)  -> [0, 1]^T
struct LogicMatrix;

impl LogicMatrix {
    fn identity() -> Matrix { Matrix::identity(2) }
    fn not() -> Matrix { Matrix::new(2, 2, vec![0.0, 1.0, 1.0, 0.0]) }
    fn always_false() -> Matrix { Matrix::new(2, 2, vec![1.0, 1.0, 0.0, 0.0]) }
    fn always_true() -> Matrix { Matrix::new(2, 2, vec![0.0, 0.0, 1.0, 1.0]) }
    fn is_prime() -> Matrix { Self::identity() } // Odd->True, Even->False

    // 二元算子
    fn equal() -> Matrix { // XNOR
        Matrix::new(2, 4, vec![
            0.0, 1.0, 1.0, 0.0, // F
            1.0, 0.0, 0.0, 1.0, // T
        ])
    }

    fn not_equal() -> Matrix { // XOR
        Matrix::new(2, 4, vec![
            1.0, 0.0, 0.0, 1.0, // F
            0.0, 1.0, 1.0, 0.0, // T
        ])
    }

    fn and() -> Matrix {
        Matrix::new(2, 4, vec![
            1.0, 1.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }
    
    fn greater_than() -> Matrix {
        Matrix::new(2, 4, vec![
            1.0, 1.0, 0.0, 1.0,
            0.0, 0.0, 1.0, 0.0,
        ])
    }

    // 算术别名
    fn add() -> Matrix { Self::not_equal() } // Z2 Addition is XOR
    fn mul() -> Matrix { Self::and() }       // Z2 Multiplication is AND
}

pub struct STPContext {
    symbol_table: HashMap<String, LogicValue>,
    energy: f64,
}

impl STPContext {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            energy: 0.0,
        }
    }

    pub fn calculate_energy(&mut self, actions: &[ProofAction]) -> f64 {
        self.energy = 0.0; 
        self.symbol_table.clear(); 

        if actions.is_empty() { return 1000.0; }

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
        }

        self.energy
    }

    fn apply_transform(&mut self, target: &str, rule: &str) {
        let current_vec_data = if let Some(LogicValue::Vector(v)) = self.symbol_table.get(target) {
            v.clone()
        } else {
            self.energy += 20.0; 
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
            self.energy += 5.0; 
        }
    }

    fn evaluate_assertion(&mut self, condition: &str) {
        let mut parser = AssertionParser::new(condition, &self.symbol_table);
        match parser.parse_and_evaluate() {
            Ok(result_vec) => {
                // 断言期望结果为真 [0, 1]^T
                let target = Matrix::new(2, 1, vec![0.0, 1.0]);
                let dist = self.matrix_dist_sq(&result_vec, &target);
                
                // 允许微小的浮点误差，但任何显著偏差都视为逻辑错误
                if dist > 0.01 {
                    self.energy += 100.0 + dist * 10.0; 
                }
            },
            Err(_) => {
                // 解析错误 (语法错误或未知变量)
                self.energy += 10.0;
            }
        }
    }

    fn matrix_dist_sq(&self, a: &Matrix, b: &Matrix) -> f64 {
        a.data.iter().zip(b.data.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum()
    }
}

// --- 严格解析器实现 ---

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Identifier(String),
    Number(String),
    Eq, Neq, Gt, Plus, Star, LParen, RParen, IsPrime,
    Eof
}

struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self { chars: input.chars().peekable() }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        match self.chars.next() {
            Some(c) => match c {
                '(' => Token::LParen,
                ')' => Token::RParen,
                '+' => Token::Plus,
                '*' => Token::Star,
                '=' => {
                    if let Some('=') = self.chars.peek() {
                        self.chars.next(); Token::Eq
                    } else { Token::Eof } // 不支持赋值
                },
                '!' => {
                    if let Some('=') = self.chars.peek() {
                        self.chars.next(); Token::Neq
                    } else { Token::Eof }
                },
                '>' => Token::Gt,
                _ if c.is_alphabetic() => {
                    let mut ident = String::new();
                    ident.push(c);
                    while let Some(&nc) = self.chars.peek() {
                        if nc.is_alphanumeric() || nc == '_' {
                            ident.push(self.chars.next().unwrap());
                        } else { break; }
                    }
                    if ident == "isPrime" { Token::IsPrime } else { Token::Identifier(ident) }
                },
                _ if c.is_numeric() => {
                    let mut num = String::new();
                    num.push(c);
                    Token::Number(num)
                },
                _ => Token::Eof, // Unknown char
            },
            None => Token::Eof,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() { self.chars.next(); } else { break; }
        }
    }
}

struct AssertionParser<'a> {
    tokenizer: Tokenizer<'a>,
    current_token: Token,
    symbols: &'a HashMap<String, LogicValue>,
}

impl<'a> AssertionParser<'a> {
    fn new(input: &'a str, symbols: &'a HashMap<String, LogicValue>) -> Self {
        let mut tokenizer = Tokenizer::new(input);
        let current_token = tokenizer.next_token();
        Self { tokenizer, current_token, symbols }
    }

    fn advance(&mut self) {
        self.current_token = self.tokenizer.next_token();
    }

    fn parse_and_evaluate(&mut self) -> Result<Matrix, String> {
        self.parse_expression()
    }

    // Expression -> Term { (== | != | >) Term }
    fn parse_expression(&mut self) -> Result<Matrix, String> {
        let mut left = self.parse_arithmetic()?;

        loop {
            let op_matrix = match self.current_token {
                Token::Eq => LogicMatrix::equal(),
                Token::Neq => LogicMatrix::not_equal(),
                Token::Gt => LogicMatrix::greater_than(),
                _ => break,
            };
            self.advance();
            let right = self.parse_arithmetic()?;
            
            // STP: M * (L (x) R)
            let input = left.stp(&right);
            left = op_matrix.stp(&input);
        }
        Ok(left)
    }

    // Arithmetic -> Factor { (+ | *) Factor }
    // 简化：目前 + 和 * 优先级相同，左结合
    fn parse_arithmetic(&mut self) -> Result<Matrix, String> {
        let mut left = self.parse_factor()?;

        loop {
            let op_matrix = match self.current_token {
                Token::Plus => LogicMatrix::add(), // XOR
                Token::Star => LogicMatrix::mul(), // AND
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            
            let input = left.stp(&right);
            left = op_matrix.stp(&input);
        }
        Ok(left)
    }

    // Factor -> Identifier | Number | isPrime(Expr) | (Expr)
    fn parse_factor(&mut self) -> Result<Matrix, String> {
        match self.current_token.clone() {
            Token::Identifier(name) => {
                self.advance();
                if let Some(LogicValue::Vector(v)) = self.symbols.get(&name) {
                    Ok(Matrix::new(2, 1, v.clone()))
                } else {
                    Err(format!("Undefined symbol: {}", name))
                }
            },
            Token::Number(val) => {
                self.advance();
                // 简单的数字处理：0为偶，非0为奇 (或在符号表中查找)
                if val == "0" {
                    Ok(Matrix::new(2, 1, vec![1.0, 0.0]))
                } else {
                    Ok(Matrix::new(2, 1, vec![0.0, 1.0]))
                }
            },
            Token::IsPrime => {
                self.advance();
                if self.current_token != Token::LParen { return Err("Expected ( after isPrime".into()); }
                self.advance();
                let expr = self.parse_expression()?; // 递归解析内部表达式
                if self.current_token != Token::RParen { return Err("Expected )".into()); }
                self.advance();
                
                // M_isPrime * V
                Ok(LogicMatrix::is_prime().stp(&expr))
            },
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                if self.current_token != Token::RParen { return Err("Expected )".into()); }
                self.advance();
                Ok(expr)
            },
            _ => Err("Unexpected token".into()),
        }
    }
}
