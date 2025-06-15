mod models;

use pyo3::prelude::*;
use polars::prelude::*;
use pyo3_polars::PyDataFrame;
use crate::models::{
    daily_result::DailyResult,
    trade_pair::TradePair,
    performance_stats::PerformanceStats,
};

#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub digits: usize,
    pub fee_rate: f32,
    pub weight_type: String,
    pub yearly_days: usize,
    pub n_jobs: usize,
}

#[pyclass]
pub struct WeightBacktest {
    daily_results: Vec<DailyResult>,
    trade_pairs: Vec<TradePair>,
    performance_stats: PerformanceStats,
}

#[pymethods]
impl WeightBacktest {
    #[new]
    pub fn new(py_df: PyDataFrame, digits: usize, weight_type: &str, fee_rate: f32, yearly_days: usize, n_job: usize) -> PyResult<Self> {
       let config = BacktestConfig {
            digits,
            fee_rate,
            weight_type: weight_type.to_string(),
            yearly_days,
            n_jobs: n_job, // Default to single-threaded
        };
        println!("config: {:?}", config);
        let polar_df: DataFrame = py_df.into();
        let daily_results: Vec<DailyResult> = vec![]; // Initialize with the results from the DataFrame
        let trade_pairs: Vec<TradePair> = vec![]; // Initialize with an empty vector
        let performance_stats = PerformanceStats::default(); // Initialize with default values

        Ok(WeightBacktest {
            daily_results,
            trade_pairs,
            performance_stats,
        })
    }

    fn add_daily_result(&mut self, result: DailyResult) {
        self.daily_results.push(result);
    }

    fn add_trade_pair(&mut self, trade_pair: TradePair) {
        self.trade_pairs.push(trade_pair);
    }

    fn get_daily_results(&self) -> Vec<DailyResult> {
        self.daily_results.clone()
    }

    fn get_trade_pairs(&self) -> Vec<TradePair> {
        self.trade_pairs.clone()
    }}

#[pymodule]
fn weight_backtest_pyo3(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<WeightBacktest>()?;
    m.add_class::<DailyResult>()?;
    m.add_class::<TradePair>()?;
    m.add_class::<PerformanceStats>()?;
    Ok(())
}