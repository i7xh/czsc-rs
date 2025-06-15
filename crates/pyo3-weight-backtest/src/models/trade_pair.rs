use serde::{Serialize, Deserialize};
use pyo3::prelude::*;

/// 交易对结构体，表示一次完整的开平仓交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass]
pub struct TradePair {
    /// 标的代码，例如 "DLi9001"
    #[pyo3(get, set)]
    pub symbol: String,

    /// 交易方向，例如 "多头" 或 "空头"
    #[pyo3(get, set)]
    pub direction: String,

    /// 开仓时间，精确到秒
    #[pyo3(get, set)]
    pub open_time: String,

    /// 平仓时间，精确到秒
    #[pyo3(get, set)]
    pub close_time: String,

    /// 开仓价格
    #[pyo3(get, set)]
    pub open_price: f32,

    /// 平仓价格
    #[pyo3(get, set)]
    pub close_price: f32,

    /// 持仓K线数量
    #[pyo3(get, set)]
    pub holding_bars: u32,

    /// 事件序列，例如 "开多 -> 平多"
    #[pyo3(get, set)]
    pub event_sequence: String,

    /// 持仓天数（根据开平仓时间计算）
    #[pyo3(get, set)]
    pub holding_days: i32,

    /// 盈亏比例（单位：基点，1基点=0.01%）
    #[pyo3(get, set)]
    pub pnl_ratio: f32,
}