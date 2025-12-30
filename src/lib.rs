// src/lib.rs
// Rust 和 Python 的“外交公署” v0.2
// 适配 BiasChannel v0.2 API

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use rug::Integer; 

use crate::control::bias_channel::{VapoConfig};
use crate::interface::{EvolverEngine, ActionDecoder, CorrectionRequest};
use crate::dsl::schema::ProofAction;
use crate::crypto::algebra::ClassGroupElement;
use crate::crypto::primes::hash_to_prime;

mod dsl;
mod control;
mod interface;
pub mod crypto; 

// =========================================================================
// Python 适配器：模拟解码器
// =========================================================================

struct SimpleRustDecoder {
    vocab_size: usize,
}

impl ActionDecoder for SimpleRustDecoder {
    fn action_space_size(&self) -> usize {
        self.vocab_size
    }

    fn decode(&self, logits: &[f64]) -> ProofAction {
        // 简单的 Argmax 实现
        let max_idx = logits.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        // [Mock] 简化的词表映射
        if max_idx == 0 {
             ProofAction::Define { 
                 symbol: "entity".to_string(), 
                 hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
             }
        } else {
             ProofAction::QED
        }
    }
}

// =========================================================================
// Crypto 适配器：HTP 协议原语
// =========================================================================

#[pyfunction]
fn py_hash_to_prime(user_id: String, bit_size: u32) -> PyResult<String> {
    hash_to_prime(&user_id, bit_size)
        .map(|i| i.to_string())
        .map_err(|e| PyValueError::new_err(format!("Prime Gen Error: {}", e)))
}

#[pyclass]
struct PyClassGroup {
    inner: ClassGroupElement,
    d: Integer, 
}

#[pymethods]
impl PyClassGroup {
    #[new]
    fn new(discriminant_str: String) -> PyResult<Self> {
        let d = Integer::from_str_radix(&discriminant_str, 10)
            .map_err(|e| PyValueError::new_err(format!("Invalid Integer: {}", e)))?;
        
        let elem = ClassGroupElement::generator(&d);
        Ok(PyClassGroup { inner: elem, d })
    }

    fn compose(&self, other: &PyClassGroup) -> PyResult<PyClassGroup> {
        if self.d != other.d {
            return Err(PyValueError::new_err("Discriminant mismatch! Cannot compose elements from different groups."));
        }
        let res = self.inner.compose(&other.inner, &self.d)
            .map_err(|e| PyValueError::new_err(e))?;
        
        Ok(PyClassGroup { inner: res, d: self.d.clone() })
    }

    fn pow(&self, exp_str: String) -> PyResult<PyClassGroup> {
        let exp = Integer::from_str_radix(&exp_str, 10)
            .map_err(|e| PyValueError::new_err(format!("Invalid Exponent: {}", e)))?;
        
        let res = self.inner.pow(&exp, &self.d)
            .map_err(|e| PyValueError::new_err(e))?;
            
        Ok(PyClassGroup { inner: res, d: self.d.clone() })
    }

    fn __repr__(&self) -> String {
        format!("ClassGroup(a={}, b={}, c={})", self.inner.a, self.inner.b, self.inner.c)
    }
}

// =========================================================================
// Python 类导出：PyEvolver
// =========================================================================

#[pyclass]
struct PyEvolver {
    inner: EvolverEngine,
    action_size: usize,
}

#[pymethods]
impl PyEvolver {
    #[new]
    fn new(action_size: usize) -> Self {
        let config = VapoConfig {
            max_iterations: 50,
            initial_temperature: 1.5,
            valuation_decay: 0.9,
        };
        PyEvolver {
            inner: EvolverEngine::new(Some(config)),
            action_size,
        }
    }

    /// [Fix] 更新签名，接收 context 参数 (字符串)
    /// Python 调用示例: evolver.align(logits_list, "Prove X is Y")
    fn align(&mut self, logits: Vec<f64>, context: String) -> PyResult<String> {
        let decoder = SimpleRustDecoder { vocab_size: self.action_size };
        
        // [New] 生成一个随机种子或基于 context hash 的种子
        // 在生产环境中，这个种子可能由 Python 端传入或者根据请求 ID 生成
        // 这里简单起见使用固定种子，模拟可重现性
        let seed = 0xCAFEBABE; 

        let request = CorrectionRequest {
            base_logits: logits,
            request_id: "py_req".to_string(),
            context: context, // [New] 传入上下文
            seed: seed,       // [New] 传入种子
        };

        match self.inner.align_generation(request, &decoder) {
            Ok(response) => {
                serde_json::to_string(&response)
                    .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
            },
            Err(e) => Err(PyValueError::new_err(e)),
        }
    }
    
    fn inject_context(&mut self, action_json: String) -> PyResult<()> {
        let action: ProofAction = serde_json::from_str(&action_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;
        
        self.inner.inject_context(&action);
        Ok(())
    }
}

// =========================================================================
// 模块定义
// =========================================================================
#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    m.add_class::<PyClassGroup>()?;
    m.add_function(wrap_pyfunction!(py_hash_to_prime, m)?)?;
    Ok(())
}
