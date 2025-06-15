use pyo3::prelude::*;
use serde::{Serialize, Deserialize};

/// 绩效统计指标结构体
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

/// 默认值实现
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