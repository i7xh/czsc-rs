use polars::prelude::*;
use std::collections::HashMap;
use anyhow::anyhow;
use polars_ops::pivot::pivot;
use crate::config::{BacktestConfig, WeightType};
use crate::errors::{CzscError, CzscResult};
use crate::stats::{daily_performance, evaluate_pairs};
use crate::types::{DailyMetric, Direction, SymbolResult, TradePair, MetricKey};
use crate::utils::RoundTo;

enum AggType { Mean, Sum }

#[derive(Debug, Clone)]
pub struct PortfolioAnalyzer {
    config: BacktestConfig,
}

impl PortfolioAnalyzer {
    pub fn new(config: BacktestConfig) -> Self {
        PortfolioAnalyzer { config }
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

    fn get_alpha_df(&self, df: &DataFrame) -> CzscResult<DataFrame> {

        // 分组聚合计算平均值
        let grouped = df
            .clone()
            .lazy()
            .group_by([col("date")])
            .agg([
                col("return").mean().alias("策略"),
                col("n1b").mean().alias("基准"),
            ])
            .collect()?;

        // 计算超额收益
        let result = grouped
            .lazy()
            .with_columns([(col("策略") - col("基准")).alias("超额")])
            .select(&[
                col("date"),
                col("策略"),
                col("基准"),
                col("超额"),
            ])
            .collect()?;

        Ok(result)
    }

    fn calculate_longshort_rates(&self, dfw: &DataFrame) -> CzscResult<(f64, f64)> {
        // 如果 DataFrame 为空，返回 (0.0, 0.0)
        if dfw.is_empty() {
            return Ok((0.0, 0.0));
        }

        // 使用表达式计算
        let df = dfw
            .clone()
            .lazy()
            .select([
                // 计算总行数
                // 计算多头数量
                col("weight").gt(0.0).sum().alias("long_count"),
                // 计算空头数量
                col("weight").lt(0.0).sum().alias("short_count"),
            ])
            .collect()?;

        // 提取结果
        let total = dfw.height() as f64;
        let long_count = df.column("long_count")?.get(0)?.try_extract::<f64>()?;
        let short_count = df.column("short_count")?.get(0)?.try_extract::<f64>()?;

        // 计算比例
        let long_rate = long_count / total;
        let short_rate = short_count / total;

        Ok((long_rate, short_rate))
    }

    pub fn analyze_portfolio_metrics(
        &self,
        df: &DataFrame,
        symbol_results: &HashMap<String, SymbolResult>
    ) -> CzscResult<HashMap<String, f64>> {
        let all_daily_metrics = symbol_results
            .values()
            .flat_map(|r| &r.daily_metrics)
            .cloned()
            .collect::<Vec<DailyMetric>>();

        let daily_metric_df: DataFrame = Self::to_daily_dateframe(&all_daily_metrics)?;
        let column_names = daily_metric_df.get_column_names();
        println!("Pivoted DataFrame columns: {:?}", column_names);

        let dret_df = pivot(
            &daily_metric_df,
            ["symbol"],
            Some(["date"]),
            Some(["return",]),
            false,
            None,
            None,)?
            .fill_null(FillNullStrategy::Zero)?;

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

        let mut stats = HashMap::new();

        let trade_pairs_stats = evaluate_pairs(&trade_pairs, Direction::LongShort)?;

        // 3. 添加关键指标到最终统计
        stats.insert("单笔收益".to_string(), trade_pairs_stats.avg_profit_per_trade);
        stats.insert("持仓K线数".to_string(), trade_pairs_stats.avg_bars_held);
        stats.insert("交易胜率".to_string(), trade_pairs_stats.win_rate);
        stats.insert("持仓天数".to_string(), trade_pairs_stats.avg_days_held);

        // 4. 计算多头空头占比
        let (long_rate, short_rate) = self.calculate_longshort_rates(&df)?;
        stats.insert("多头占比".to_string(), long_rate.round_to(4));
        stats.insert("空头占比".to_string(), short_rate.round_to(4));

        let alpha_df = self.get_alpha_df(&daily_metric_df)?;let strategy_returns = alpha_df.column("策略")?.f64()?;
        let benchmark_returns = alpha_df.column("基准")?.f64()?;

        let strategy_std = strategy_returns.std(1).unwrap_or(f64::NAN);
        let benchmark_std = benchmark_returns.std(1).unwrap_or(f64::NAN);

        // 1. 波动比计算
        stats.insert(
            "波动比".to_string(),
            if benchmark_std > 1e-6 && benchmark_std.is_finite() {
                (strategy_std / benchmark_std).round_to(4)
            } else {
                f64::NAN
            }
        );


        stats.insert(
            "与基准波动相关性".to_string(),
            alpha_df.clone()
                .lazy()
                .select(&[pearson_corr(col("策略"), col("基准").abs()).alias("相关系数")])
                .collect()?
                .column("相关系数")?
                .f64()?
                .get(0)
                .unwrap()
                .round_to(4)
        );

        stats.insert(
            "与基准收益相关性".to_string(),
            alpha_df.clone()
                .lazy()
                .select(&[pearson_corr(col("策略"), col("基准")).alias("相关系数")])
                .collect()?
                .column("相关系数")?
                .f64()?
                .get(0)
                .unwrap()
                .abs()
                .round_to(4)
        );

        stats.insert(
            "与基准空头相关性".to_string(),
            alpha_df.clone()
                .lazy()
                .filter(col("基准").lt(0.0))
                .select(&[pearson_corr(col("策略"), col("基准")).alias("相关系数")])
                .collect()?
                .column("相关系数")?
                .f64()?
                .get(0)
                .unwrap()
                .round_to(4)
        );

        stats.insert(
            "品种数量".to_string(),
            symbol_results.len() as f64
        );

        Ok(stats)
    }

}