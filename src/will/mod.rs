//! The Will Module (意志模块)
//! 
//! 负责系统的意图导向和优化搜索。
//! 在本体论修正案中，Will 变成了在定四元数算术格（Pizer Graph）上的导航员。
//! 
//! - Perturber: 提供图上的合法移动（Hecke 算子）。
//! - Evaluator: 计算状态的能量（真理度）。
//! - Optimizer: 执行搜索策略（VAPO）。
//! - Tracer: 记录因果路径。

pub mod evaluator;
pub mod optimizer;
pub mod perturber;
pub mod tracer;
