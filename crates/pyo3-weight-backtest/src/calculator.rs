use std::collections::VecDeque;
use chrono::NaiveDateTime;
use super::types::*;
use polars::prelude::*;
use crate::config::BacktestConfig;
use crate::errors::CzscError;
use crate::czsc_err;
use crate::errors::CzscResult;
use crate::trade_position::TradePositionState;
use crate::types::TradeAction::{CloseLong, CloseShort, OpenLong, OpenShort};

#[derive(Debug, Clone)]
pub struct MetricCalculator {
    config: BacktestConfig,
}

impl MetricCalculator {
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    pub fn calculate_daily_metrics(
        &self,
        symbol: &str,
        symbol_df: &DataFrame,
    ) -> CzscResult<DailyResult> {
        let digit_u32 = self.config.digits as u32;
        let fee_rate = self.config.fee_rate;
        // 实现核心指标计算逻辑
        // 使用 Polars 高效计算
        let lazy_df = symbol_df.clone()
            .lazy()
            .with_column(
                (col("weight") * lit(10f64.powi(digit_u32 as i32)))
                    .cast(DataType::Int32)
                    .alias("volume"),
            )
            .with_column((lit(1)..lit(i32::MAX)).end.alias("bar_id"))
            .with_column((col("price").shift(Expr::from(-1)) / col("price") - lit(1.0)).alias("n1b"))
            .with_column((col("weight") * col("n1b")).alias("edge"))
            .with_column(
                (col("weight").shift(Expr::from(1)) - col("weight"))
                    .abs()
                    .alias("turnover"),
            )
            .with_column((col("turnover") * lit(fee_rate)).alias("cost"))
            .with_columns([
                (col("edge") - col("cost")).alias("return"),
                when(col("weight").gt(0.0))
                    .then(col("weight"))
                    .otherwise(lit(0.0))
                    .alias("long_weight"),
                when(col("weight").lt(0.0))
                    .then(col("weight"))
                    .otherwise(lit(0.0))
                    .alias("short_weight"),
            ])
            .with_columns([
                (col("long_weight") * col("n1b")).alias("long_edge"),
                (col("short_weight") * col("n1b")).alias("short_edge"),
            ])
            .with_columns([
                (col("long_weight").shift(Expr::from(1)) - col("long_weight"))
                    .abs()
                    .alias("long_turnover"),
                (col("short_weight").shift(Expr::from(1)) - col("short_weight"))
                    .abs()
                    .alias("short_turnover"),
            ])
            .with_columns([
                (col("long_turnover") * lit(fee_rate)).alias("long_cost"),
                (col("short_turnover") * lit(fee_rate)).alias("short_cost"),
            ])
            .with_columns([
                (col("long_edge") - col("long_cost")).alias("long_return"),
                (col("short_edge") - col("short_cost")).alias("short_return"),
            ]);
        unimplemented!()
    }

    pub fn generate_trade_pairs(
        &self,
        symbol: &str,
        symbol_df: &DataFrame,
    ) -> CzscResult<Vec<TradePair>> {
        let digits_u32 = self.config.digits as u32;

        let new_df = symbol_df.clone()
            .lazy()
            .with_column(
                (col("weight").round(digits_u32, RoundMode::HalfAwayFromZero)
                    * lit(10i32.pow(digits_u32)))
                    .cast(DataType::Int32)
                    .alias("volume"),
            )
            .with_row_index("bar_id", Some(0))
            .collect()?;

        let mut state = TradePositionState::Flat;
        let mut all_actions = Vec::new();

        let dt_series = new_df.column("dt")?.datetime()?;
        let volume_series = new_df.column("volume")?.i32()?;
        let price_series = new_df.column("price")?.f64()?;
        let bar_id_series = new_df.column("bar_id")?.u32()?;
        let weight_series = new_df.column("weight")?.f64()?;

        for i in 0..new_df.height() {
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
                    return Err(czsc_err!(Unknown, "DataFrame contains null values in required columns"))
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