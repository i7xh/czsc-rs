use chrono::NaiveDateTime;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass]
pub struct DailyResult {
    #[pyo3(get)]
    pub date: String,

    #[pyo3(get)]
    pub symbol: String,

    #[pyo3(get)]
    pub edge: f32,

    #[pyo3(get, name = "return")]
    pub return_val: f32,

    #[pyo3(get)]
    pub cost: f32,

    #[pyo3(get)]
    pub n1b: f32,

    #[pyo3(get)]
    pub turnover: f32,

    #[pyo3(get)]
    pub long_edge: f32,

    #[pyo3(get)]
    pub long_cost: f32,

    #[pyo3(get)]
    pub long_return: f32,

    #[pyo3(get)]
    pub long_turnover: f32,

    #[pyo3(get)]
    pub short_edge: f32,

    #[pyo3(get)]
    pub short_cost: f32,

    #[pyo3(get)]
    pub short_return: f32,

    #[pyo3(get)]
    pub short_turnover: f32,
}

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

#[derive(Debug, Serialize)]
pub struct DailyMetrics {
    // FIXME: 根据实际需要添加字段
    pub date: String,
    pub symbol: String,
    pub edge: f64,
    pub return_val: f64,
    pub cost: f64,
    pub n1b: f64,
    pub turnover: f64,
    pub long_edge: f64,
    pub short_edge: f64,
    // 其他字段...
}

#[derive(Serialize, Debug)]
pub struct SymbolResult {
    pub daily_metrics: Vec<DailyMetrics>,
    pub trade_pairs: Vec<TradePair>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct PerformanceStats {
    /// 绝对收益（累计收益率）
    #[pyo3(get, set)]
    #[serde(rename = "绝对收益")]
    pub absolute_return: f32,

    /// 年化收益率（百分比）
    #[pyo3(get, set)]
    #[serde(rename = "年化")]
    pub annualized_return: f32,

    /// 夏普比率
    #[pyo3(get, set)]
    #[serde(rename = "夏普")]
    pub sharpe_ratio: f32,

    /// 最大回撤（负值，表示损失）
    #[pyo3(get, set)]
    #[serde(rename = "最大回撤")]
    pub max_drawdown: f32,

    /// 卡玛比率（年化收益/最大回撤）
    #[pyo3(get, set)]
    #[serde(rename = "卡玛")]
    pub calmar_ratio: f32,

    /// 日胜率（0-1之间的小数）
    #[pyo3(get, set)]
    #[serde(rename = "日胜率")]
    pub daily_win_rate: f32,

    /// 日盈亏比（平均盈利/平均亏损）
    #[pyo3(get, set)]
    #[serde(rename = "日盈亏比")]
    pub daily_profit_loss_ratio: f32,

    /// 日赢面（0-1之间的小数）
    #[pyo3(get, set)]
    #[serde(rename = "日赢面")]
    pub daily_winning_edge: f32,

    /// 年化波动率
    #[pyo3(get, set)]
    #[serde(rename = "年化波动率")]
    pub annualized_volatility: f32,

    /// 下行波动率
    #[pyo3(get, set)]
    #[serde(rename = "下行波动率")]
    pub downside_volatility: f32,

    /// 非零覆盖（0-1之间的小数）
    #[pyo3(get, set)]
    #[serde(rename = "非零覆盖")]
    pub non_zero_coverage: f32,

    /// 盈亏平衡点（达到盈亏平衡所需的最小收益率）
    #[pyo3(get, set)]
    #[serde(rename = "盈亏平衡点")]
    pub break_even_point: f32,

    /// 新高间隔（天数）
    #[pyo3(get, set)]
    #[serde(rename = "新高间隔")]
    pub new_high_interval: f32,

    /// 新高占比（0-1之间的小数）
    #[pyo3(get, set)]
    #[serde(rename = "新高占比")]
    pub new_high_ratio: f32,

    /// 回撤风险（0-1之间的小数）
    #[pyo3(get, set)]
    #[serde(rename = "回撤风险")]
    pub drawdown_risk: f32,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            absolute_return: 0.0,
            annualized_return: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            calmar_ratio: 0.0,
            daily_win_rate: 0.0,
            daily_profit_loss_ratio: 0.0,
            daily_winning_edge: 0.0,
            annualized_volatility: 0.0,
            downside_volatility: 0.0,
            non_zero_coverage: 0.0,
            break_even_point: 0.0,
            new_high_interval: 0.0,
            new_high_ratio: 0.0,
            drawdown_risk: 0.0,
        }
    }
}
