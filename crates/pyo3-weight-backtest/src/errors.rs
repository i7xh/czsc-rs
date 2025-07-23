use crate::config::WeightType;
use anyhow::anyhow;
use polars::error::PolarsError;
use pyo3::PyErr;
use rayon::ThreadPoolBuildError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CzscError {
    /// 数据验证错误
    #[error("数据验证失败: {0}")]
    Validation(String),

    /// 数据处理错误
    #[error("数据处理错误: {0}")]
    DataProcessing(String),

    /// 回测逻辑错误
    #[error("回测逻辑错误: {0}")]
    BacktestLogic(String),

    /// 性能计算错误
    #[error("绩效计算错误: {0}")]
    Performance(String),

    /// Python交互错误
    #[error("Python交互错误: {0}")]
    PythonInteraction(String),

    /// 输入输出错误
    #[error("IO错误: {0}")]
    Io(#[from] io::Error),

    /// 序列化/反序列化错误
    #[error("序列化错误: {0}")]
    Serialization(String),

    /// Polars数据处理错误
    #[error("Polars错误: {0}")]
    Polars(#[from] PolarsError),

    // Rayon 并行处理错误
    #[error("Rayon 错误: {0}")]
    Rayon(#[from] ThreadPoolBuildError),

    #[error("通用错误: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Invalid weight type: {0:?}")]
    InvalidWeightType(WeightType),

    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("Parse Float Error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("未知错误: {0}")]
    Unknown(String),
}

impl From<serde_json::Error> for CzscError {
    fn from(err: serde_json::Error) -> Self {
        CzscError::Serialization(err.to_string())
    }
}

impl From<String> for CzscError {
    fn from(err: String) -> Self {
        CzscError::Unknown(err)
    }
}

// 实现 PyO3 错误转换
impl From<CzscError> for PyErr {
    fn from(err: CzscError) -> PyErr {
        match err {
            CzscError::Validation(msg) => pyo3::exceptions::PyValueError::new_err(msg),
            _ => pyo3::exceptions::PyRuntimeError::new_err(err.to_string()),
        }
    }
}

// 扩展 Result 类型
pub type CzscResult<T> = Result<T, CzscError>;

/// 为错误添加上下文
pub trait ErrorContext<T, E> {
    fn context(self, context: &str) -> CzscResult<T>;
}

impl<T, E> ErrorContext<T, E> for Result<T, E>
where
    E: Into<CzscError>,
{
    fn context(self, context: &str) -> CzscResult<T> {
        self.map_err(|e| {
            let base_err: CzscError = e.into();
            match base_err {
                CzscError::Validation(msg) => {
                    CzscError::Validation(format!("{}: {}", context, msg))
                }
                CzscError::DataProcessing(msg) => {
                    CzscError::DataProcessing(format!("{}: {}", context, msg))
                }
                CzscError::BacktestLogic(msg) => {
                    CzscError::BacktestLogic(format!("{}: {}", context, msg))
                }
                CzscError::Performance(msg) => {
                    CzscError::Performance(format!("{}: {}", context, msg))
                }
                CzscError::PythonInteraction(msg) => {
                    CzscError::PythonInteraction(format!("{}: {}", context, msg))
                }
                CzscError::Io(e) => {
                    CzscError::Io(io::Error::new(e.kind(), format!("{}: {}", context, e)))
                }
                CzscError::Serialization(msg) => {
                    CzscError::Serialization(format!("{}: {}", context, msg))
                }
                CzscError::Polars(e) => CzscError::Polars(e),
                CzscError::Rayon(e) => CzscError::Rayon(e),
                CzscError::Anyhow(e) => anyhow!("{}: {}", context, e).into(),
                CzscError::Unknown(msg) => CzscError::Unknown(format!("{}: {}", context, msg)),
                CzscError::InvalidWeightType(weight_type) => {
                    CzscError::InvalidWeightType(weight_type)
                }
                CzscError::ColumnNotFound(col) => {
                    CzscError::ColumnNotFound(format!("{}: {}", context, col))
                }
                _ => {
                    CzscError::Unknown(format!("{}: {}", context, base_err.to_string()))
                }
            }
        })
    }
}
