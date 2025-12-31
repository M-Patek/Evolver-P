use pyo3::prelude::*;
use pyo3::types::PyDict;
use num_bigint::BigInt;
use serde_json;

use crate::soul::algebra::ClassGroupElement;

/// 暴露给 Python 的 Evolver 核心接口
#[pyclass(name = "AlgebraicSoul")]
pub struct PyAlgebraicSoul {
    /// 当前的代数状态 (The Current Intuition State)
    state: ClassGroupElement,
    
    /// 系统的基础判别式 Δ (The Fundamental Discriminant)
    /// 所有的演化都必须保持在这个判别式定义的类群中。
    discriminant: BigInt,
}

#[pymethods]
impl PyAlgebraicSoul {
    /// 构造一个新的代数灵魂
    ///
    /// # 参数
    /// * `discriminant_str`: 判别式 Δ 的字符串表示 (e.g. "-23")
    ///   注意：必须是负数，且 ≡ 0, 1 (mod 4)。
    #[new]
    fn new(discriminant_str: String) -> PyResult<Self> {
        let delta: BigInt = discriminant_str.parse()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid BigInt: {}", e)))?;

        // 初始状态为单位元 (Identity / Principal Class)
        // 代表“心如止水”的初始直觉
        let identity = ClassGroupElement::identity(&delta);

        Ok(PyAlgebraicSoul {
            state: identity,
            discriminant: delta,
        })
    }

    /// 获取当前状态的 JSON 表示
    /// 返回: {"a": "...", "b": "...", "c": "..."}
    fn get_state_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.state)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Serialization error: {}", e)))
    }

    /// 演化 (Evolve)
    ///
    /// 注入外部熵 (Seed)，推动灵魂在类群轨道上跳跃一步。
    /// 这是产生“确定性混沌”的关键动作。
    ///
    /// # 参数
    /// * `seed`: 外部输入的随机种子 (u64)
    ///
    /// # 返回
    /// 演化后的新状态 JSON
    fn evolve(&mut self, seed: u64) -> PyResult<String> {
        // 调用核心代数引擎进行演化
        // state <- state * g_in(seed)
        self.state = self.state.evolve(seed);

        self.get_state_json()
    }

    /// 重置 (Reset)
    ///
    /// 将灵魂重置回单位元状态。
    fn reset(&mut self) -> PyResult<String> {
        self.state = ClassGroupElement::identity(&self.discriminant);
        self.get_state_json()
    }

    /// 逆向回溯 (Inverse)
    ///
    /// 计算当前状态的逆元。
    /// 在逻辑验证失败需要“撤回”直觉时非常有用。
    fn inverse(&self) -> PyResult<String> {
        let inv_state = self.state.inverse();
        serde_json::to_string(&inv_state)
             .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Serialization error: {}", e)))
    }
}
