use pyo3::pyclass;
use serde::{Deserialize, Serialize};

/// 日度回测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass]
pub struct DailyResult {
    #[pyo3(get)]
    pub date: String,

    #[pyo3(get)]
    pub symbol: String,

    #[pyo3(get)]
    pub edge: f32,

    #[pyo3(get, name = "return")]
    pub return_val: f32,

    #[pyo3(get)]
    pub cost: f32,

    #[pyo3(get)]
    pub n1b: f32,

    #[pyo3(get)]
    pub turnover: f32,

    #[pyo3(get)]
    pub long_edge: f32,

    #[pyo3(get)]
    pub long_cost: f32,

    #[pyo3(get)]
    pub long_return: f32,

    #[pyo3(get)]
    pub long_turnover: f32,

    #[pyo3(get)]
    pub short_edge: f32,

    #[pyo3(get)]
    pub short_cost: f32,

    #[pyo3(get)]
    pub short_return: f32,

    #[pyo3(get)]
    pub short_turnover: f32,
}
