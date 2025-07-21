use polars::prelude::*;
use std::collections::HashMap;
use polars_ops::pivot::pivot;
use crate::config::{BacktestConfig, WeightType};
use crate::errors::{CzscError, CzscResult};
use crate::portfolio_builder::PortfolioMetricsBuilder;
use crate::types::{DailyMetric, SymbolResult};

enum AggType { Mean, Sum }

#[derive(Debug, Clone)]
pub struct PortfolioAnalyzer<'a> {
    config: BacktestConfig,
    symbol_results: &'a HashMap<String, SymbolResult>,
    df: &'a DataFrame,
    daily_df: &'a DataFrame,
    daily_ew_return_df: &'a DataFrame,
}

impl<'a> PortfolioAnalyzer<'a> {
    pub fn new(
        config: BacktestConfig,
        symbol_results: &'a HashMap<String, SymbolResult>,
        df: &'a DataFrame,
        daily_df: &'a DataFrame,
        daily_ew_return_df: &'a DataFrame,
    ) -> Self {
        PortfolioAnalyzer { config , symbol_results, df, daily_df: &daily_df, daily_ew_return_df: &daily_ew_return_df }
    }

    fn to_daily_dateframe(metrics: &[&DailyMetric]) -> CzscResult<DataFrame> {
        let mut dates = Vec::with_capacity(metrics.len());
        let mut symbols = Vec::with_capacity(metrics.len());
        let mut edges = Vec::with_capacity(metrics.len());
        let mut returns = Vec::with_capacity(metrics.len());
        let mut costs = Vec::with_capacity(metrics.len());
        let mut n1bs = Vec::with_capacity(metrics.len());
        let mut turnovers = Vec::with_capacity(metrics.len());
        let mut long_edges = Vec::with_capacity(metrics.len());
        let mut long_costs = Vec::with_capacity(metrics.len());
        let mut long_returns = Vec::with_capacity(metrics.len());
        let mut long_turnovers = Vec::with_capacity(metrics.len());
        let mut short_edges = Vec::with_capacity(metrics.len());
        let mut short_costs = Vec::with_capacity(metrics.len());
        let mut short_returns = Vec::with_capacity(metrics.len());
        let mut short_turnovers = Vec::with_capacity(metrics.len());

        for metric in metrics {
            dates.push(metric.date);
            symbols.push(metric.symbol.clone());
            edges.push(metric.edge);
            returns.push(metric.return_val);
            costs.push(metric.cost);
            n1bs.push(metric.n1b);
            turnovers.push(metric.turnover);
            long_edges.push(metric.long_edge);
            long_costs.push(metric.long_cost);
            long_returns.push(metric.long_return);
            long_turnovers.push(metric.long_turnover);
            short_edges.push(metric.short_edge);
            short_costs.push(metric.short_cost);
            short_returns.push(metric.short_return);
            short_turnovers.push(metric.short_turnover);
        }

        Ok(df![
            "date" => dates,
            "symbol" => symbols,
            "edge" => edges,
            "return" => returns,
            "cost" => costs,
            "n1b" => n1bs,
            "turnover" => turnovers,
            "long_edge" => long_edges,
            "long_cost" => long_costs,
            "long_return" => long_returns,
            "long_turnover" => long_turnovers,
            "short_edge" => short_edges,
            "short_cost" => short_costs,
            "short_return" => short_returns,
            "short_turnover" => short_turnovers,
        ]?)
    }
    pub fn gen_daily_metric_df(symbol_results: &HashMap<String, SymbolResult>) -> DataFrame {
        let all_daily_metrics: Vec<&DailyMetric> = symbol_results
            .values()
            .flat_map(|r| &r.daily_metrics)
            .collect();
        Self::to_daily_dateframe(&all_daily_metrics).unwrap()
    }

    fn add_agg_column(lf: LazyFrame, symbols: &[&str], agg_type: AggType, ) -> CzscResult<LazyFrame> {
        let agg_expr = symbols
            .iter()
            .map(|&s| col(s))
            .reduce(|acc, col| acc + col)
            .unwrap_or(lit(0.0));

        let expr = match agg_type {
            AggType::Mean => (agg_expr / lit(symbols.len() as f64)).alias("total"),
            AggType::Sum => agg_expr.alias("total"),
        };
        Ok(lf.with_columns([expr]))
    }

    pub fn gen_daily_ew_return_df(weight_type: WeightType, symbol_results: &HashMap<String, SymbolResult>, daily_df: &DataFrame) -> CzscResult<DataFrame> {
        let dret_df = pivot(
            daily_df,
            ["symbol"],
            Some(["date"]),
            Some(["return"]),
            false,
            None,
            None,
        )?
            .fill_null(FillNullStrategy::Zero)?;

        let symbols = symbol_results.keys().map(|s| s.as_str()).collect::<Vec<&str>>();

        let dret_lf = match weight_type {
            WeightType::TimeSeries => Self::add_agg_column(dret_df.lazy(), &symbols, AggType::Mean)?,
            WeightType::CrossSection => Self::add_agg_column(dret_df.lazy(), &symbols, AggType::Sum)?,
            _ => return Err(CzscError::InvalidWeightType(weight_type).into()),
        };

        Ok(dret_lf.select(&[
                col("date"),
                all().exclude(["date"]).round(4, RoundMode::HalfAwayFromZero)
            ])
            .with_row_index("idx", Some(0))
            .collect()?)
    }
    pub fn analyze_portfolio_metrics(&self) -> CzscResult<HashMap<String, f64>> {

        let metrics = PortfolioMetricsBuilder::new(
            &self.config, &self.df, &self.daily_df, &self.daily_ew_return_df, self.symbol_results)
            .add_basic_metrics()?
            .add_trade_pair_metrics()?
            .add_long_short_metrics()?
            .add_benchmark_correlations()?
            .add_portfolio_return_metrics()?
            .build();
        Ok(metrics)
    }

}