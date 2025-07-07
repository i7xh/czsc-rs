pub mod engine;
mod errors;
mod types;
pub mod utils;
mod trade_position;
mod calculator;
pub mod config;

use crate::types::{DailyResult, PerformanceStats, TradePair};
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;
use crate::config::BacktestConfig;
use crate::engine::BacktestEngine;

#[pyclass]
pub struct WeightBacktest {
    daily_results: Vec<DailyResult>,
    trade_pairs: Vec<TradePair>,
    performance_stats: PerformanceStats,
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
        n_job: usize,
    ) -> PyResult<Self> {
        let config = BacktestConfig {
            digits,
            fee_rate,
            weight_type: weight_type.to_string(),
            yearly_days,
            n_jobs: n_job, // Default to single-threaded
        };

        println!("config: {:?}", config);
        let polar_df: DataFrame = py_df.into();

        let engine = BacktestEngine::new(polar_df, config)?;
        let _ = engine.run()?;
        unimplemented!()
    }

}

#[pymodule]
fn weight_backtest_pyo3(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<WeightBacktest>()?;
    m.add_class::<DailyResult>()?;
    m.add_class::<PerformanceStats>()?;
    Ok(())
}
