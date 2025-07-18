pub mod engine;
mod errors;
mod types;
pub mod utils;
mod trade_position;
mod processor;
pub mod config;
mod stats;
mod analyzer;
mod constants;

use crate::types::TradePair;
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;
use crate::config::BacktestConfig;
use crate::engine::BacktestEngine;

#[pyclass]
pub struct WeightBacktest {
    // daily_results: Vec<DailyResult>,
    trade_pairs: Vec<TradePair>,
    // performance_stats: PerformanceStats,
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
        let config = BacktestConfig::new(
            digits,
            fee_rate,
            weight_type.to_string(),
            yearly_days,
            n_job,
        )?;

        println!("config: {:?}", config);
        let polar_df: DataFrame = py_df.into();

        let engine = BacktestEngine::new(polar_df, config)?;
        let _ = engine.run_backtest()?;
        unimplemented!()
    }

}

#[pymodule]
fn weight_backtest_pyo3(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<WeightBacktest>()?;
    Ok(())
}
