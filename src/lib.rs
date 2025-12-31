use pyo3::prelude::*;

// 模块声明
pub mod control;
pub mod dsl;
pub mod interface;
pub mod soul; // [New] 注入灵魂模块！

/// Python 模块入口
#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    // 注册 Python 类和函数
    // m.add_class::<interface::PyEvolver>()?;
    Ok(())
}
