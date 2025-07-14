use std::collections::{HashMap, VecDeque};
use anyhow::anyhow;
use chrono::{NaiveDate, NaiveDateTime};
use super::types::*;
use polars::prelude::*;
use crate::config::{BacktestConfig, WeightType};
use crate::errors::CzscResult;
use crate::stats::{daily_performance};
use crate::trade_position::TradePositionState;
use crate::types::TradeAction::{CloseLong, CloseShort, OpenLong, OpenShort};

#[derive(Debug, Clone)]
pub struct MetricProcessor {
    config: BacktestConfig,
}

impl MetricProcessor {
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    pub fn process_daily_metrics(
        &self,
        symbol: &str,
        symbol_df: &DataFrame,
    ) -> CzscResult<Vec<DailyMetric>> {
        let fee_rate = self.config.fee_rate;
        // 实现核心指标计算逻辑
        // 使用 Polars 高效计算
        let df = symbol_df.clone()
            .lazy()
            // 计算基准收益率：n1b = (下一期价格 / 当前价格) - 1
            .with_column(
                (col("price").shift(Expr::from(-1)) / col("price") - lit(1.0))
                    .alias("n1b")
            )
            // 计算策略理论收益：edge = weight * n1b
            .with_column(
                (col("weight") * col("n1b"))
                    .alias("edge")
            )
            // 计算换手率：|当期权重 - 上期权重|
            .with_column(
                (col("weight").shift(Expr::from(1)) - col("weight"))
                    .abs()
                    .fill_null(lit(0.0))
                    .alias("turnover"),
            )
            // 计算交易成本：cost = turnover * fee_rate
            .with_column(
                (col("turnover") * lit(fee_rate))
                    .alias("cost")
            )
            // 计算净收益：return = edge - cost
            .with_column(
                (col("edge") - col("cost"))
                    .alias("return")
            )
            // 分离多空头寸：weight > 0
            .with_column(
                when(col("weight").gt(0.0))
                    .then(col("weight"))
                    .otherwise(lit(0.0))
                    .alias("long_weight")
            )
            // 分离空头寸：weight < 0
            .with_column(
                when(col("weight").lt(0.0))
                    .then(col("weight"))
                    .otherwise(lit(0.0))
                    .alias("short_weight"),
            )
            // 计算多头理论收益
            .with_column(
                (col("long_weight") * col("n1b")).alias("long_edge")
            )
            // 计算空头理论收益
            .with_column(
                (col("short_weight") * col("n1b")).alias("short_edge"),
            )
            // 计算多头换手率
            .with_column(
                (col("long_weight").shift(Expr::from(1)) - col("long_weight"))
                    .abs()
                    .fill_null(lit(0.0))
                    .alias("long_turnover"),
            )
            // 计算空头换手率
            .with_column(
                (col("short_weight").shift(Expr::from(1)) - col("short_weight"))
                    .abs()
                    .fill_null(lit(0.0))
                    .alias("short_turnover"),
            )
            // 计算多头交易成本
            .with_column(
                (col("long_turnover") * lit(fee_rate)).alias("long_cost")
            )
            // 计算空头交易成本
            .with_column(
                (col("short_turnover") * lit(fee_rate)).alias("short_cost"),
            )
            // 计算多头净收益
            .with_column(
                (col("long_edge") - col("long_cost")).alias("long_return")
            )
            // 计算空头净收益
            .with_column(
                (col("short_edge") - col("short_cost")).alias("short_return"),
            )
            // 提取日期部分（不含时间）
            .with_column(
                col("dt").dt().strftime("%Y-%m-%d").alias("date")
            );

        let aggregated_df = df.lazy()
            .group_by([col("date")])
            .agg([
                col("edge").sum().alias("edge"),
                col("return").sum().alias("return"),
                col("cost").sum().alias("cost"),
                col("n1b").sum().alias("n1b"),
                col("turnover").sum().alias("turnover"),
                col("long_edge").sum().alias("long_edge"),
                col("long_cost").sum().alias("long_cost"),
                col("long_return").sum().alias("long_return"),
                col("long_turnover").sum().alias("long_turnover"),
                col("short_edge").sum().alias("short_edge"),
                col("short_cost").sum().alias("short_cost"),
                col("short_return").sum().alias("short_return"),
                col("short_turnover").sum().alias("short_turnover"),
            ])
            .sort(["date"], SortMultipleOptions::default())
            .collect()?;

        let mut daily_metrics = Vec::with_capacity(aggregated_df.height());

        for idx in 0..aggregated_df.height() {
            let date = aggregated_df.column("date")?.str()?.get(idx).unwrap_or_default();
            let edge = aggregated_df.column("edge")?.f64()?.get(idx).unwrap_or(0.0);
            let return_val = aggregated_df.column("return")?.f64()?.get(idx).unwrap_or(0.0);
            let cost = aggregated_df.column("cost")?.f64()?.get(idx).unwrap_or(0.0);
            let n1b = aggregated_df.column("n1b")?.f64()?.get(idx).unwrap_or(0.0);
            let turnover = aggregated_df.column("turnover")?.f64()?.get(idx).unwrap_or(0.0);
            let long_edge = aggregated_df.column("long_edge")?.f64()?.get(idx).unwrap_or(0.0);
            let long_cost = aggregated_df.column("long_cost")?.f64()?.get(idx).unwrap_or(0.0);
            let long_return = aggregated_df.column("long_return")?.f64()?.get(idx).unwrap_or(0.0);
            let long_turnover = aggregated_df.column("long_turnover")?.f64()?.get(idx).unwrap_or(0.0);
            let short_edge = aggregated_df.column("short_edge")?.f64()?.get(idx).unwrap_or(0.0);
            let short_cost = aggregated_df.column("short_cost")?.f64()?.get(idx).unwrap_or(0.0);
            let short_return = aggregated_df.column("short_return")?.f64()?.get(idx).unwrap_or(0.0);
            let short_turnover = aggregated_df.column("short_turnover")?.f64()?.get(idx).unwrap_or(0.0);

            daily_metrics.push(DailyMetric {
                date: date.parse().unwrap(),
                symbol: symbol.to_string(),
                edge,
                return_val,
                cost,
                n1b,
                turnover,
                long_edge,
                long_cost,
                long_return,
                long_turnover,
                short_edge,
                short_cost,
                short_return,
                short_turnover,
            });
        }

        Ok(daily_metrics)
    }

    pub fn generate_trade_pairs(
        &self,
        symbol: &str,
        symbol_df: &DataFrame,
    ) -> CzscResult<Vec<TradePair>> {
        let mut state = TradePositionState::Flat;
        let mut all_actions = Vec::new();

        let dt_series = symbol_df.column("dt")?.datetime()?;
        let volume_series = symbol_df.column("volume")?.i32()?;
        let price_series = symbol_df.column("price")?.f64()?;
        let bar_id_series = symbol_df.column("bar_id")?.u32()?;
        let weight_series = symbol_df.column("weight")?.f64()?;

        for i in 0..symbol_df.height() {
            let (dt, volume, price, bar_id, weight) = match (
                dt_series
                    .get(i)
                    .and_then(|ts| NaiveDateTime::from_timestamp_nanos(ts)),
                volume_series.get(i),
                price_series.get(i),
                bar_id_series.get(i),
                weight_series.get(i),
            ) {
                (Some(dt), Some(volume), Some(price), Some(bar_id), Some(weight)) => {
                    (dt, volume, price, bar_id, weight)
                }
                _ =>
                    return Err(anyhow!("DataFrame contains null values in required columns").into())
            };
            let actions = state.handle_transition(volume, dt, price as f32, bar_id as usize);
            all_actions.extend(actions);
        }
        let trade_pairs = self.actions_to_trade_pairs(symbol, all_actions)?;

        Ok(trade_pairs)
    }

    fn actions_to_trade_pairs(&self, symbol: &str, actions: Vec<TradeAction>) -> CzscResult<Vec<TradePair>> {
        let mut trade_pairs: Vec<TradePair> = vec![];
        let mut open_long_queue: VecDeque<TradeAction> = VecDeque::new();
        let mut open_short_queue: VecDeque<TradeAction> = VecDeque::new();

        for action in actions {
            match action {
                OpenLong {
                    dt,
                    price,
                    bar_id,
                } => {
                    open_long_queue.push_back(OpenLong {
                        dt,
                        price,
                        bar_id,
                    });
                }
                OpenShort {
                    dt,
                    price,
                    bar_id,
                } => {
                    open_short_queue.push_back(OpenShort {
                        dt,
                        price,
                        bar_id,
                    });
                }
                CloseLong {
                    dt,
                    price,
                    bar_id,
                } => {
                    if let Some(open_action) = open_long_queue.pop_front() {
                        if let OpenLong {
                            dt: open_dt,
                            price: open_price,
                            bar_id: open_bar_id,
                        } = open_action
                        {
                            let bar_count = bar_id - open_bar_id + 1;
                            let holding_days = (dt - open_dt).num_days() as usize + 1;
                            let profit_ratio = (price - open_price) / open_price * 10000.0;
                            trade_pairs.push(TradePair {
                                symbol: symbol.to_string(), // Symbol should be set appropriately
                                direction: Direction::Long,
                                open_dt,
                                close_dt: dt,
                                open_price: open_price as f64,
                                close_price: price as f64,
                                bar_count,
                                event_sequence: "开多 -> 平多".to_string(), // Event sequence should be set appropriately
                                holding_days: holding_days as i64,
                                profit_ratio: profit_ratio as f64,
                            });
                        }
                    } else {
                        // Handle case where there is no open long position
                        eprintln!("Warning: Attempted to close a long position without an open position.");
                    }
                }
                CloseShort {
                    dt,
                    price,
                    bar_id,
                } => {
                    if let Some(open_action) = open_short_queue.pop_front() {
                        if let OpenShort {
                            dt: open_dt,
                            price: open_price,
                            bar_id: open_bar_id,
                        } = open_action
                        {
                            let bar_count = bar_id - open_bar_id + 1;
                            let holding_days = (dt - open_dt).num_days() as usize + 1;
                            let profit_ratio = (open_price - price) / open_price * 10000.0;
                            trade_pairs.push(TradePair {
                                symbol: symbol.to_string(),
                                direction: Direction::Short,
                                open_dt,
                                close_dt: dt,
                                open_price: open_price as f64,
                                close_price: price as f64,
                                bar_count,
                                event_sequence: "开空 -> 平空".to_string(),
                                holding_days: holding_days as i64,
                                profit_ratio: profit_ratio as f64,
                            });
                        }
                    } else {
                        // Handle case where there is no open short position
                        eprintln!("Warning: Attempted to close a short position without an open position.");
                    }
                }
            }
        }
        Ok(trade_pairs)
    }

}