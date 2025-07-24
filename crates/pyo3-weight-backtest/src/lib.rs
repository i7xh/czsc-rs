mod analyzer;
pub mod config;
pub mod engine;
mod errors;
mod portfolio_builder;
mod processor;
mod stats;
mod trade_position;
mod types;
pub mod utils;

use crate::config::BacktestConfig;
use crate::engine::{BacktestEngine, BacktestResult};
use crate::types::{DailyMetric, Direction, SymbolResult, TradePair};
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;

#[pyclass]
pub struct WeightBacktest {
    engine: BacktestEngine,
}

#[pymethods]
impl WeightBacktest {
    #[new]
    pub fn new(
        py_df: PyDataFrame,
        digits: usize,
        weight_type: &str,
        fee_rate: f32,
        yearly_days: usize,
        n_jobs: usize,
    ) -> PyResult<Self> {
        let config = BacktestConfig::new(
            digits,
            fee_rate,
            weight_type.to_string(),
            yearly_days,
            n_jobs,
        )?;

        Ok(WeightBacktest {
            engine: BacktestEngine::new(py_df.into(), config)?
        })
    }

    pub fn run_backtest(&self) -> PyResult<BacktestResult> {
        let result = self.engine.run_backtest()?;
        Ok(result)
    }
}

#[pymodule]
fn weight_backtest_pyo3(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<WeightBacktest>()?;
    m.add_class::<Direction>()?;
    m.add_class::<DailyMetric>()?;
    m.add_class::<TradePair>()?;
    m.add_class::<SymbolResult>()?;
    m.add_class::<BacktestResult>()?;
    Ok(())
}
