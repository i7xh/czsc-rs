use std::error::Error;
use std::fmt;
use polars::error::PolarsError;
use rayon::ThreadPoolBuildError;

#[derive(Debug)]
pub enum BacktestError {
    Polars(PolarsError),
    Validation(String),
    Processing(String),
    Rayon(ThreadPoolBuildError),
}

impl fmt::Display for BacktestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BacktestError::Polars(e) => write!(f, "Polars error: {}", e),
            BacktestError::Validation(e) => write!(f, "Validation error: {}", e),
            BacktestError::Processing(e) => write!(f, "Processing error: {}", e),
            BacktestError::Rayon(e) => write!(f, "Rayon error: {}", e),
        }
    }
}

impl From<PolarsError> for BacktestError {
    fn from(err: PolarsError) -> Self {
        BacktestError::Polars(err)
    }
}

impl From<ThreadPoolBuildError> for BacktestError {
    fn from(err: ThreadPoolBuildError) -> Self {
        BacktestError::Rayon(err)
    }
    
}