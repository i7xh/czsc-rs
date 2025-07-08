use chrono::{NaiveDate, NaiveDateTime};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum TradeAction {
    OpenLong { dt: NaiveDateTime, price: f32, bar_id: usize },
    OpenShort { dt: NaiveDateTime, price: f32, bar_id: usize },
    CloseLong { dt: NaiveDateTime, price: f32, bar_id: usize },
    CloseShort { dt: NaiveDateTime, price: f32, bar_id: usize },
}


// 交易方向枚举
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Direction {
    Long,
    Short,
}

// 交易对结构体
#[derive(Debug, Clone, Serialize)]
pub struct TradePair {
    pub symbol:         String,
    pub direction:      Direction,
    pub open_dt:        NaiveDateTime,
    pub close_dt:       NaiveDateTime,
    pub open_price:     f64,
    pub close_price:    f64,
    pub bar_count:      usize,
    pub event_sequence: String,
    pub holding_days:   i64,
    pub profit_ratio:   f64,
}

// DailyMetrics 结构体定义
#[derive(Debug, Clone, Serialize)]
pub struct DailyMetric {
    pub date: NaiveDate,
    pub symbol: String,
    pub edge: f64,
    pub return_val: f64,
    pub cost: f64,
    pub n1b: f64,
    pub turnover: f64,
    pub long_edge: f64,
    pub long_cost: f64,
    pub long_return: f64,
    pub long_turnover: f64,
    pub short_edge: f64,
    pub short_cost: f64,
    pub short_return: f64,
    pub short_turnover: f64,
}

#[derive(Serialize, Debug)]
pub struct SymbolResult {
    pub daily_metrics: Vec<DailyMetric>,
    pub trade_pairs: Vec<TradePair>,
}

/// 组合级绩效指标
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    // 基本信息
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub num_symbols: usize,
    pub total_trades: usize,

    // 收益指标
    pub total_return: f64,
    pub annualized_return: f64,
    pub cagr: f64,  // 复合年增长率

    // 风险指标
    pub volatility: f64,  // 年化波动率
    pub max_drawdown: f64,
    pub max_drawdown_duration: usize,  // 最大回撤持续时间(天)

    // 风险调整收益
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,

    // 胜率相关
    pub win_rate: f64,  // 胜率
    pub profit_factor: f64,  // 盈利因子 (总盈利/总亏损)
    pub avg_win_trade: f64,  // 平均盈利交易收益
    pub avg_loss_trade: f64,  // 平均亏损交易损失

    // 持仓时间
    pub avg_holding_days: f64,
    pub median_holding_days: f64,

    // 多空指标
    pub long_ratio: f64,  // 多头交易占比
    pub short_ratio: f64,  // 空头交易占比
    pub long_win_rate: f64,
    pub short_win_rate: f64,

    // 换手率
    pub avg_daily_turnover: f64,
    pub annual_turnover: f64,

    // 基准对比
    pub alpha: f64,  // 超额收益
    pub beta: f64,   // 系统性风险
    pub tracking_error: f64,  // 跟踪误差
    pub information_ratio: f64,

    // 其他
    pub skewness: f64,  // 收益偏度
    pub kurtosis: f64,  // 收益峰度
}
