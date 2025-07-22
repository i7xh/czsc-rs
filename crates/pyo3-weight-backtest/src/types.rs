use chrono::{NaiveDate, NaiveDateTime};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone)]
pub enum TradeAction {
    OpenLong {
        dt    : NaiveDateTime,
        price : f32,
        bar_id: usize,
    },
    OpenShort {
        dt    : NaiveDateTime,
        price : f32,
        bar_id: usize,
    },
    CloseLong {
        dt    : NaiveDateTime,
        price : f32,
        bar_id: usize,
    },
    CloseShort {
        dt    : NaiveDateTime,
        price : f32,
        bar_id: usize,
    },
}

// 交易方向枚举
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Direction {
    Long,
    Short,
    LongShort, // 代表多空都可以
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Long => write!(f, "Long"),
            Direction::Short => write!(f, "Short"),
            Direction::LongShort => write!(f, "LongShort"),
        }
    }
}

// 交易对结构体
#[derive(Debug, Clone, Serialize)]
pub struct TradePair {
    pub symbol        : String,
    pub direction     : Direction,
    pub open_dt       : NaiveDateTime,
    pub close_dt      : NaiveDateTime,
    pub open_price    : f64,
    pub close_price   : f64,
    pub bar_count     : usize,
    pub event_sequence: String,
    pub holding_days  : i64,
    pub profit_ratio  : f64,
}

#[derive(Debug, Default)]
pub struct TradeEvaluation {
    pub trade_direction        : String, // 交易方向
    pub trade_count            : usize,  // 交易次数
    pub total_profit           : f64,    // 累计收益
    pub avg_profit_per_trade   : f64,    // 单笔收益
    pub win_count              : usize,  // 盈利次数
    pub total_win_profit       : f64,    // 累计盈利
    pub avg_win_profit         : f64,    // 单笔盈利
    pub loss_count             : usize,  // 亏损次数
    pub total_loss             : f64,    // 累计亏损
    pub avg_loss               : f64,    // 单笔亏损
    pub win_rate               : f64,    // 交易胜率
    pub total_profit_loss_ratio: f64,    // 累计盈亏比
    pub avg_profit_loss_ratio  : f64,    // 单笔盈亏比
    pub break_even_point       : f64,    // 盈亏平衡点
    pub avg_days_held          : f64,    // 平均持仓天数
    pub avg_bars_held          : f64,    // 平均持仓K线数
}

// DailyMetrics 结构体定义
#[derive(Debug, Clone, Serialize)]
pub struct DailyMetric {
    pub date          : NaiveDate,
    pub symbol        : String,
    pub edge          : f64,
    pub return_val    : f64,
    pub cost          : f64,
    pub n1b           : f64,
    pub turnover      : f64,
    pub long_edge     : f64,
    pub long_cost     : f64,
    pub long_return   : f64,
    pub long_turnover : f64,
    pub short_edge    : f64,
    pub short_cost    : f64,
    pub short_return  : f64,
    pub short_turnover: f64,
}

#[derive(Serialize, Debug)]
pub struct SymbolResult {
    pub daily_metrics: Vec<DailyMetric>,
    pub trade_pairs  : Vec<TradePair>,
}

/// 组合级绩效指标
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    // 基本信息
    pub start_date           : NaiveDate,
    pub end_date             : NaiveDate,
    pub num_symbols          : usize,
    pub total_trades         : usize,

    // 收益指标
    pub total_return         : f64,
    pub annualized_return    : f64,
    pub cagr                 : f64,       // 复合年增长率

    // 风险指标
    pub volatility           : f64,       // 年化波动率
    pub max_drawdown         : f64,
    pub max_drawdown_duration: usize,     // 最大回撤持续时间(天)

    // 风险调整收益
    pub sharpe_ratio         : f64,
    pub sortino_ratio        : f64,
    pub calmar_ratio         : f64,

    // 胜率相关
    pub win_rate             : f64,       // 胜率
    pub profit_factor        : f64,       // 盈利因子 (总盈利/总亏损)
    pub avg_win_trade        : f64,       // 平均盈利交易收益
    pub avg_loss_trade       : f64,       // 平均亏损交易损失

    // 持仓时间
    pub avg_holding_days     : f64,
    pub median_holding_days  : f64,

    // 多空指标
    pub long_ratio           : f64,       // 多头交易占比
    pub short_ratio          : f64,       // 空头交易占比
    pub long_win_rate        : f64,
    pub short_win_rate       : f64,

    // 换手率
    pub avg_daily_turnover   : f64,
    pub annual_turnover      : f64,

    // 基准对比
    pub alpha                : f64,       // 超额收益
    pub beta                 : f64,       // 系统性风险
    pub tracking_error       : f64,       // 跟踪误差
    pub information_ratio    : f64,

    // 其他
    pub skewness             : f64,       // 收益偏度
    pub kurtosis             : f64,       // 收益峰度
}

// 指标键枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricKey {
    TradeProfit,
    HoldingBars,
    WinRate,
    HoldingDays,
    LongRatio,
    ShortRatio,
    VolatilityRatio,
    VolatilityCorrelation,
    ReturnCorrelation,
    BearCorrelation,
    SymbolCount,
    MaxDrawdown,
    SharpeRatio,
    StartDate,
    EndDate,
}

impl MetricKey {
    fn as_str(&self) -> &'static str {
        match self {
            Self::TradeProfit => "单笔收益",
            Self::HoldingBars => "持仓K线数",
            Self::WinRate => "交易胜率",
            Self::HoldingDays => "持仓天数",
            Self::LongRatio => "多头占比",
            Self::ShortRatio => "空头占比",
            Self::VolatilityRatio => "波动比",
            Self::VolatilityCorrelation => "与基准波动相关性",
            Self::ReturnCorrelation => "与基准收益相关性",
            Self::BearCorrelation => "与基准空头相关性",
            Self::SymbolCount => "品种数量",
            Self::MaxDrawdown => "最大回撤",
            Self::SharpeRatio => "夏普比率",
            Self::StartDate => "起始日期",
            Self::EndDate => "结束日期",
        }
    }
}

impl ToString for MetricKey {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}
