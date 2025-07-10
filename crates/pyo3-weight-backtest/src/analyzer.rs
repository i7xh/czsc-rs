use std::collections::HashMap;
use chrono::NaiveDate;
use polars::prelude::*;
use polars_ops::pivot::pivot;
use crate::config::BacktestConfig;
use crate::errors::CzscResult;
use crate::types::{DailyMetric, SymbolResult};

#[derive(Debug, Clone)]
pub struct PortfolioAnalyzer {
    config: BacktestConfig,
}

impl PortfolioAnalyzer {
    pub fn new(config: BacktestConfig) -> Self {
        PortfolioAnalyzer { config }
    }

    pub fn get_config(&self) -> &BacktestConfig {
        &self.config
    }

    pub fn analyze_portfolio_metrics(
        &self, 
        symbol_results: &HashMap<String, SymbolResult>
    ) -> CzscResult<()> {
        let all_daily_metrics = symbol_results
            .values()
            .flat_map(|r| &r.daily_metrics)
            .cloned()
            .collect::<Vec<DailyMetric>>();

        let daily_metric_df: DataFrame = Self::to_dateframe(&all_daily_metrics)?;
        let out = pivot(
            &daily_metric_df,
            ["date"],
            Some(["symbol"]),
            Some(["return",]),
            false,
            None,
            None,)?;
        println!("Pivoted DataFrame:\n{}", out);

        unimplemented!()
    }

    fn to_dateframe(metrics: &[DailyMetric]) -> CzscResult<DataFrame> {
        Ok(df![
            "date" => metrics.iter().map(|m| m.date).collect::<Vec<NaiveDate>>(),
            "symbol" => metrics.iter().map(|m| m.symbol.clone()).collect::<Vec<String>>(),
            "edge" => metrics.iter().map(|m| m.edge).collect::<Vec<f64>>(),
            "return" => metrics.iter().map(|m| m.return_val).collect::<Vec<f64>>(),
            "cost" => metrics.iter().map(|m| m.cost).collect::<Vec<f64>>(),
            "n1b" => metrics.iter().map(|m| m.n1b).collect::<Vec<f64>>(),
            "turnover" => metrics.iter().map(|m| m.turnover).collect::<Vec<f64>>(),
            "long_edge" => metrics.iter().map(|m| m.long_edge).collect::<Vec<f64>>(),
            "long_cost" => metrics.iter().map(|m| m.long_cost).collect::<Vec<f64>>(),
            "long_return" => metrics.iter().map(|m| m.long_return).collect::<Vec<f64>>(),
            "long_turnover" => metrics.iter().map(|m| m.long_turnover).collect::<Vec<f64>>(),
            "short_edge" => metrics.iter().map(|m| m.short_edge).collect::<Vec<f64>>(),
            "short_cost" => metrics.iter().map(|m| m.short_cost).collect::<Vec<f64>>(),
            "short_return" => metrics.iter().map(|m| m.short_return).collect::<Vec<f64>>(),
            "short_turnover" => metrics.iter().map(|m| m.short_turnover).collect::<Vec<f64>>(),
        ]?)
    }

}