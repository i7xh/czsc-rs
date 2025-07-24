use crate::config::BacktestConfig;
use crate::errors::CzscResult;
use crate::stats::{daily_performance, evaluate_pairs};
use crate::types::{Direction, SymbolResult, TradePair};
use crate::utils::RoundTo;
use chrono::{Days, NaiveDate};
use polars::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CorrelationStats {
    pub volatility_ratio       : f64,
    pub volatility_correlation : f64,
    pub return_correlation     : f64,
    pub downside_correlation   : f64,
}

// 构建器结构
pub struct PortfolioMetricsBuilder<'a> {
    config            : &'a BacktestConfig,
    symbol_results    : &'a HashMap<String, SymbolResult>,
    df                : &'a DataFrame,
    daily_df          : &'a DataFrame,
    daily_ew_return_df: &'a DataFrame,
    stats             : HashMap<String, f64>,
}

impl<'a> PortfolioMetricsBuilder<'a> {
    /// 创建新的构建器实例
    pub fn new(
        config: &'a BacktestConfig,
        df: &'a DataFrame,
        daily_df: &'a DataFrame,
        daily_ew_return_df: &'a DataFrame,
        symbol_results: &'a HashMap<String, SymbolResult>,
    ) -> Self {
        PortfolioMetricsBuilder {
            config,
            symbol_results,
            df,
            daily_df,
            daily_ew_return_df,
            stats: HashMap::new(),
        }
    }

    /// 添加基本指标
    pub fn add_basic_metrics(mut self) -> CzscResult<Self> {
        // 品种数量
        self.stats.insert("品种数量".to_string(), self.symbol_results.len() as f64);

        // 日期范围
        let base_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        if let Some(min_days)
            = self.daily_df.column("date")?.str()?.cast(&DataType::Date)?.date()?.min() {
            if let Some(min_date) = base_date
                .checked_add_days(Days::new(min_days as u64)) {
                self.stats.insert(
                    "最小日期".to_string(),
                    min_date.format("%Y%m%d").to_string().parse::<f64>()?
                );
            }
        }

        if let Some(max_days)
            = self.daily_df.column("date")?.str()?.cast(&DataType::Date)?.date()?.max() {
            if let Some(max_date) = base_date
                .checked_add_days(Days::new(max_days as u64)) {
                self.stats.insert(
                    "最大日期".to_string(),
                    max_date.format("%Y%m%d").to_string().parse::<f64>()?,
                );
            }
        }

        Ok(self)
    }

    /// 添加交易对统计指标
    pub fn add_trade_pair_metrics(mut self) -> CzscResult<Self> {
        println!("process add_trade_pair_metrics");
        let trade_pairs: Vec<TradePair> = self
            .symbol_results
            .values()
            .flat_map(|sr| sr.trade_pairs.iter().cloned())
            .collect();

        let stats = evaluate_pairs(&trade_pairs, Direction::LongShort)?;

        self.stats.insert("单笔收益".to_string(), stats.avg_profit_per_trade);
        self.stats.insert("持仓K线数".to_string(), stats.avg_bars_held);
        self.stats.insert("交易胜率".to_string(), stats.win_rate);
        self.stats.insert("持仓天数".to_string(), stats.avg_days_held);

        Ok(self)
    }

    /// 添加多空占比指标
    pub fn add_long_short_metrics(mut self) -> CzscResult<Self> {
        println!("process add_long_short_metrics");
        let (long_rate, short_rate) = self.calculate_longshort_rates()?;

        self.stats.insert("多头占比".to_string(), long_rate.round_to(4));
        self.stats.insert("空头占比".to_string(), short_rate.round_to(4));

        Ok(self)
    }

    /// 添加基准相关性指标
    pub fn add_benchmark_correlations(mut self) -> CzscResult<Self> {
        println!("process add_benchmark_correlations");
        let alpha_df = self.get_alpha_df()?;
        let corr_stats = self.calculate_correlations(&alpha_df)?;

        self.stats.insert(
            "波动比".to_string(),
            corr_stats.volatility_ratio.round_to(4),
        );
        self.stats.insert(
            "与基准波动相关性".to_string(),
            corr_stats.volatility_correlation,
        );
        self.stats.insert(
            "与基准收益相关性".to_string(),
            corr_stats.return_correlation,
        );
        self.stats.insert(
            "与基准空头相关性".to_string(),
            corr_stats.downside_correlation,
        );

        Ok(self)
    }

    /// 添加组合收益指标
    pub fn add_portfolio_return_metrics(mut self) -> CzscResult<Self> {
        println!("process add_portfolio_return_metrics");
        let returns: Vec<f64> =
            self.daily_ew_return_df.column("total")?.f64()?.into_iter().flatten().collect();

        let perf_stats = daily_performance(&returns, Some(self.config.yearly_days as f64));

        for (key, value) in perf_stats.iter() {
            self.stats.insert(key.clone(), *value);
        }
        Ok(self)
    }

    /// 完成构建并返回指标集合
    pub fn build(self) -> HashMap<String, f64> {
        self.stats
    }

    /// 计算多空占比
    fn calculate_longshort_rates(&self) -> CzscResult<(f64, f64)> {
        if self.df.is_empty() {
            return Ok((0.0, 0.0));
        }

        // 使用表达式计算
        let df = self
            .df
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
        let total = self.df.height() as f64;
        let long_count = df.column("long_count")?.get(0)?.try_extract::<f64>()?;
        let short_count = df.column("short_count")?.get(0)?.try_extract::<f64>()?;

        // 计算比例
        let long_rate = long_count / total;
        let short_rate = short_count / total;

        Ok((long_rate, short_rate))
    }

    /// 计算相关性指标
    fn calculate_correlations(&self, alpha_df: &DataFrame) -> CzscResult<CorrelationStats> {
        let strategy_returns = alpha_df.column("策略")?.f64()?;
        let benchmark_returns = alpha_df.column("基准")?.f64()?;

        let strategy_std = strategy_returns.std(1).unwrap_or(f64::NAN);
        let benchmark_std = benchmark_returns.std(1).unwrap_or(f64::NAN);

        let volatility_ratio = if benchmark_std > 1e-6 && benchmark_std.is_finite() {
            strategy_std / benchmark_std
        } else {
            f64::NAN
        };

        let corr_df = alpha_df
            .clone()
            .lazy()
            .select(&[
                pearson_corr(col("策略"), col("基准").abs()).alias("vol_corr"),
                pearson_corr(col("策略"), col("基准")).alias("return_corr"),
            ])
            .collect()?;

        let volatility_correlation =
            corr_df.column("vol_corr")?.f64()?.get(0).unwrap_or(f64::NAN).round_to(4);

        let return_correlation = corr_df
            .column("return_corr")?
            .f64()?
            .get(0)
            .unwrap_or(f64::NAN)
            .abs()
            .round_to(4);

        let downside_correlation = alpha_df
            .clone()
            .lazy()
            .filter(col("基准").lt(0.0))
            .select([pearson_corr(col("策略"), col("基准")).alias("down_corr")])
            .collect()?
            .column("down_corr")?
            .f64()?
            .get(0)
            .unwrap_or(f64::NAN)
            .round_to(4);

        Ok(CorrelationStats {
            volatility_ratio,
            volatility_correlation,
            return_correlation,
            downside_correlation,
        })
    }

    fn get_alpha_df(&self) -> CzscResult<DataFrame> {
        // 分组聚合计算平均值
        let grouped = self
            .daily_df
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
            .select(&[col("date"), col("策略"), col("基准"), col("超额")])
            .collect()?;

        Ok(result)
    }
}
