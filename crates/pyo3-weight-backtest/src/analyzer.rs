use std::collections::HashMap;
use anyhow::anyhow;
use polars::prelude::*;
use polars_ops::pivot::pivot;
use pyo3::impl_::wrap::SomeWrap;
use crate::config::{BacktestConfig, WeightType};
use crate::errors::CzscResult;
use crate::stats::daily_performance;
use crate::types::{DailyMetric, SymbolResult, TradePair};

enum AggType { Mean, Sum }

#[derive(Debug, Clone)]
pub struct PortfolioAnalyzer {
    config: BacktestConfig,
}

impl PortfolioAnalyzer {
    pub fn new(config: BacktestConfig) -> Self {
        PortfolioAnalyzer { config }
    }

    fn to_trade_pair_dateframe(pairs: &[TradePair]) -> CzscResult<DataFrame> {
        let mut symbols = Vec::with_capacity(pairs.len());
        let mut directions = Vec::with_capacity(pairs.len());
        let mut open_dts = Vec::with_capacity(pairs.len());
        let mut close_dts = Vec::with_capacity(pairs.len());
        let mut open_prices = Vec::with_capacity(pairs.len());
        let mut close_prices = Vec::with_capacity(pairs.len());
        let mut bar_counts = Vec::with_capacity(pairs.len());
        let mut holding_days = Vec::with_capacity(pairs.len());
        let mut profit_ratios = Vec::with_capacity(pairs.len());


        for pair in pairs {
            symbols.push(pair.symbol.clone());
            directions.push(pair.direction);
            open_dts.push(pair.open_dt);
            close_dts.push(pair.close_dt);
            open_prices.push(pair.open_price);
            close_prices.push(pair.close_price);
            bar_counts.push(pair.bar_count as i64);
            holding_days.push(pair.holding_days);
            profit_ratios.push(pair.profit_ratio);
        }

        Ok(
            df![
                "symbol" => symbols,
                // "direction" => directions,
                "open_dt" => open_dts,
                "close_dt" => close_dts,
                "open_price" => open_prices,
                "close_price" => close_prices,
                "bar_count" => bar_counts,
                "holding_days" => holding_days,
                "profit_ratio" => profit_ratios,
            ]?)
    }

    fn to_daily_dateframe(metrics: &[DailyMetric]) -> CzscResult<DataFrame> {
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

    fn add_agg_column(
        lf: LazyFrame,
        symbols: &[&str],
        agg_type: AggType,
    ) -> CzscResult<LazyFrame> {
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

    pub fn analyze_portfolio_metrics(
        &self,
        symbol_results: &HashMap<String, SymbolResult>
    ) -> CzscResult<DataFrame> {
        let all_daily_metrics = symbol_results
            .values()
            .flat_map(|r| &r.daily_metrics)
            .cloned()
            .collect::<Vec<DailyMetric>>();

        let daily_metric_df: DataFrame = Self::to_daily_dateframe(&all_daily_metrics)?;

        let dret_df = pivot(
            &daily_metric_df,
            ["symbol"],
            Some(["date"]),
            Some(["return",]),
            false,
            None,
            None,)?
            .fill_null(FillNullStrategy::Zero)?;

        println!("Pivoted DataFrame:\n{}", dret_df);
        let symbols = symbol_results.keys().map(|s| s.as_str()).collect::<Vec<&str>>();
        let dret_lf = match self.config.weight_type {
            WeightType::TimeSeries => {
                Self::add_agg_column(dret_df.lazy(), &*symbols, AggType::Mean)?
            },
            WeightType::CrossSection => {
                Self::add_agg_column(dret_df.lazy(), &*symbols, AggType::Sum)?
            },
            _ => return Err(anyhow!("Unsupported weight type {:?}", self.config.weight_type).into()),
        };

        // 创建表达式列表
        let mut exprs = vec![col("date")];  // 保留 date 列

        // 添加所有其他列的四舍五入表达式
        let rounded_exprs = all()
            .exclude(["date"])
            .round(4, RoundMode::HalfAwayFromZero);
        exprs.push(rounded_exprs);

        // 应用选择并添加行索引
        let dret_lf = dret_lf
            .select(exprs)
            .with_row_index("idx", Some(0));

        let dret_df = dret_lf.collect()?;

        let mut res = HashMap::new();
        res.insert("品种等权日收益".to_string(), &dret_df);

        //TODO: 进一步分析和处理数据
        let ca = dret_df.column("date")?.date()?;
        if let Some(min) = ca.min() {
            println!("数据集最小日期: {}", min);
        }
        if  let Some(max) = ca.max() {
            println!("数据集最大日期: {}", max);
        }

        let returns: Vec<f64> = dret_df.column("total")?.f64()?
            .into_iter()
            .flatten()
            .collect();
        daily_performance(&returns, Some(self.config.yearly_days as f64));

        let trade_pairs: Vec<TradePair> = symbol_results
            .values()
            .flat_map(|sr| sr.trade_pairs.iter().cloned())
            .collect();

        let trade_pair_df = Self::to_trade_pair_dateframe(&trade_pairs);

        unimplemented!()
    }
}