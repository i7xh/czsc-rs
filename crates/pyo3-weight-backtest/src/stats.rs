use polars::prelude::*;
use std::collections::HashMap;
use pyo3::impl_::wrap::SomeWrap;
use crate::errors::CzscResult;
use crate::types::{Direction, TradeEvaluation, TradePair};
use crate::utils::RoundTo;

// 计算盈亏平衡点的辅助函数
fn cal_break_even_point(seq: &[f64]) -> f64 {
    // 处理空序列或总收益为负的情况
    if seq.is_empty() || seq.iter().sum::<f64>() < 0.0 {
        return 1.0;
    }

    // 创建可修改的副本并排序（升序）
    let mut sorted_seq = seq.to_vec();
    sorted_seq.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = sorted_seq.len() as f64;
    let mut cumulative_sum = 0.0;
    let mut break_even_index = sorted_seq.len(); // 默认值（全部交易后平衡）

    // 遍历排序后的序列，找到第一个累计和 >= 0 的位置
    for (i, &value) in sorted_seq.iter().enumerate() {
        cumulative_sum += value;
        if cumulative_sum >= 0.0 {
            break_even_index = i + 1; // i 是从 0 开始，需要 +1
            break;
        }
    }

    // 返回盈亏平衡点比例
    break_even_index as f64 / n
}

// 评估交易记录
pub fn evaluate_pairs(pairs: &[TradePair], direction: Direction) -> CzscResult<TradeEvaluation> {

    // 筛选指定方向的交易
    let filtered_pairs: Vec<&TradePair> = if direction == Direction::LongShort {
        pairs.iter().collect()
    } else {
        pairs.iter()
            .filter(|p| p.direction == direction)
            .collect()
    };

    let mut result = TradeEvaluation {
        trade_direction: Direction::LongShort.to_string(),
        ..Default::default()
    };

    // 如果筛选后为空，返回默认结果
    if filtered_pairs.is_empty() {
        return Ok(result);
    }

    // 设置交易方向
    result.trade_direction = direction.to_string();
    // 计算基本统计
    result.trade_count = filtered_pairs.len();

    // 盈亏平衡点
    let profit_ratios: Vec<f64> = filtered_pairs.iter()
        .map(|p| p.profit_ratio)
        .collect();
    result.break_even_point = cal_break_even_point(&profit_ratios).round_to(4);

    // 累计收益和平均收益
    result.total_profit = filtered_pairs.iter()
        .map(|p| p.profit_ratio)
        .sum::<f64>()
        .round_to(2);
    result.avg_profit_per_trade = (result.total_profit / result.trade_count as f64).round_to(2);

    // 平均持仓天数和K线数
    result.avg_days_held = filtered_pairs.iter()
        .map(|p| p.holding_days)
        .sum::<i64>() as f64 / result.trade_count as f64;
    result.avg_bars_held = filtered_pairs.iter()
        .map(|p| p.holding_days as f64)
        .sum::<f64>() / result.trade_count as f64;

    // 分离盈利和亏损交易
    let (win_trades, loss_trades): (Vec<&TradePair>, Vec<&TradePair>) = filtered_pairs.iter()
        .partition(|p| p.profit_ratio >= 0.0);

    // 计算盈利相关指标
    if !win_trades.is_empty() {
        result.win_count = win_trades.len();
        result.total_win_profit = win_trades.iter()
            .map(|p| p.profit_ratio)
            .sum::<f64>();
        result.avg_win_profit = (result.total_win_profit / result.win_count as f64).round_to(4);
        result.win_rate = (result.win_count as f64 / result.trade_count as f64).round_to(4);
    }

    // 计算亏损相关指标
    if !loss_trades.is_empty() {
        result.loss_count = loss_trades.len();
        result.total_loss = loss_trades.iter()
            .map(|p| p.profit_ratio)
            .sum::<f64>();
        result.avg_loss = (result.total_loss / result.loss_count as f64).round_to(4);

        // 计算盈亏比
        if result.total_loss != 0.0 {
            result.total_profit_loss_ratio = (result.total_win_profit / result.total_loss.abs()).round_to(4);
        }
        if result.avg_loss != 0.0 {
            result.avg_profit_loss_ratio = (result.avg_win_profit / result.avg_loss.abs()).round_to(4);
        }
    }

    Ok(result)
}

// 采用单利计算日收益数据的各项指标
pub fn daily_performance(daily_returns: &[f64], yearly_days: Option<f64>) -> HashMap<String, f64> {
    let yearly_days = yearly_days.unwrap_or(252.0);

    // 初始化结果
    let mut metrics: HashMap<String, f64> = HashMap::new();
    metrics.insert("绝对收益".to_string(), 0.0);
    metrics.insert("年化".to_string(), 0.0);
    metrics.insert("夏普".to_string(), 0.0);
    metrics.insert("最大回撤".to_string(), 0.0);
    metrics.insert("卡玛".to_string(), 0.0);
    metrics.insert("日胜率".to_string(), 0.0);
    metrics.insert("日盈亏比".to_string(), 0.0);
    metrics.insert("日赢面".to_string(), 0.0);
    metrics.insert("年化波动率".to_string(), 0.0);
    metrics.insert("下行波动率".to_string(), 0.0);
    metrics.insert("非零覆盖".to_string(), 0.0);
    metrics.insert("盈亏平衡点".to_string(), 0.0);
    metrics.insert("新高间隔".to_string(), 0.0);
    metrics.insert("新高占比".to_string(), 0.0);
    metrics.insert("回撤风险".to_string(), 0.0);

    if daily_returns.is_empty() {
        return metrics;
    }

    // 计算基本统计量
    let n = daily_returns.len() as f64;
    let sum_return: f64 = daily_returns.iter().sum();
    let mean_return = sum_return / n;
    let variance: f64 = daily_returns.iter()
        .map(|x| (x - mean_return).powi(2))
        .sum::<f64>() / n;
    let std_dev = variance.sqrt();

    // 检查无效数据
    if std_dev == 0.0 || daily_returns.iter().all(|&x| x == 0.0) {
        return metrics;
    }

    // 计算累计收益和回撤
    let mut cum_returns = Vec::with_capacity(daily_returns.len());
    let mut current_cum = 0.0;
    for &ret in daily_returns {
        current_cum += ret;
        cum_returns.push(current_cum);
    }

    let mut max_drawdown = 0.0;
    let mut peak = f64::MIN;
    let mut drawdowns = vec![0.0; daily_returns.len()];
    let mut new_high_count = 0;

    for (i, &cum) in cum_returns.iter().enumerate() {
        if cum > peak {
            peak = cum;
            new_high_count += 1;
        }
        let dd = peak - cum;
        if dd > max_drawdown {
            max_drawdown = dd;
        }
        drawdowns[i] = dd;
    }

    // 计算新高间隔
    let mut new_high_durations = Vec::new();
    let mut current_duration = 0;

    for &dd in &drawdowns {
        if dd == 0.0 {
            if current_duration > 0 {
                new_high_durations.push(current_duration);
            }
            current_duration = 1;
        } else {
            current_duration += 1;
        }
    }
    if current_duration > 0 {
        new_high_durations.push(current_duration);
    }

    let max_interval = new_high_durations.iter().max().copied().unwrap_or(0) as f64;
    let high_pct = new_high_count as f64 / n;

    // 计算盈利/亏损交易
    let win_returns: Vec<f64> = daily_returns.iter()
        .filter(|&&x| x >= 0.0)
        .copied()
        .collect();

    let loss_returns: Vec<f64> = daily_returns.iter()
        .filter(|&&x| x < 0.0)
        .copied()
        .collect();

    let win_count = win_returns.len() as f64;
    let loss_count = loss_returns.len() as f64;

    let win_pct = win_count / n;
    let win_mean = if win_count > 0.0 {
        win_returns.iter().sum::<f64>() / win_count
    } else {
        0.0
    };

    let loss_mean = if loss_count > 0.0 {
        loss_returns.iter().sum::<f64>() / loss_count
    } else {
        0.0
    };

    let daily_ykb = if loss_mean != 0.0 {
        win_mean / loss_mean.abs()
    } else if win_mean > 0.0 {
        5.0 // 默认值
    } else {
        0.0
    };

    let win_expectation = win_pct * daily_ykb - (1.0 - win_pct);

    // 计算波动率
    let annual_volatility = std_dev * yearly_days.sqrt();

    let downside_returns: Vec<f64> = loss_returns.iter()
        .map(|x| x.abs())
        .collect();

    let downside_std = if downside_returns.is_empty() {
        0.0
    } else {
        let downside_mean = downside_returns.iter().sum::<f64>() / downside_returns.len() as f64;
        let downside_var: f64 = downside_returns.iter()
            .map(|x| (x - downside_mean).powi(2))
            .sum::<f64>() / downside_returns.len() as f64;
        downside_var.sqrt()
    };

    let downside_volatility = downside_std * yearly_days.sqrt();

    // 计算非零覆盖
    let non_zero_count = daily_returns.iter().filter(|&&x| x != 0.0).count() as f64;
    let non_zero_cover = non_zero_count / n;

    // 计算卡玛比率
    let kama = if max_drawdown != 0.0 {
        let annual_return = mean_return * yearly_days;
        annual_return / max_drawdown
    } else {
        10.0 // 默认值
    };

    // 辅助函数：限制指标范围
    fn min_max(x: f64, min_val: f64, max_val: f64, digits: i32) -> f64 {
        let multiplier = 10_f64.powi(digits);
        let x_clamped = x.clamp(min_val, max_val);
        (x_clamped * multiplier).round() / multiplier
    }

    // 填充结果
    metrics.insert("绝对收益".to_string(), (sum_return * 10000.0).round() / 10000.0);
    metrics.insert("年化".to_string(), (mean_return * yearly_days * 10000.0).round() / 10000.0);
    metrics.insert("夏普".to_string(), min_max(
        (mean_return / std_dev) * yearly_days.sqrt(),
        -5.0, 10.0, 2
    ));
    metrics.insert("最大回撤".to_string(), (max_drawdown * 10000.0).round() / 10000.0);
    metrics.insert("卡玛".to_string(), min_max(kama, -10.0, 20.0, 2));
    metrics.insert("日胜率".to_string(), (win_pct * 10000.0).round() / 10000.0);
    metrics.insert("日盈亏比".to_string(), (daily_ykb * 10000.0).round() / 10000.0);
    metrics.insert("日赢面".to_string(), (win_expectation * 10000.0).round() / 10000.0);
    metrics.insert("年化波动率".to_string(), (annual_volatility * 10000.0).round() / 10000.0);
    metrics.insert("下行波动率".to_string(), (downside_volatility * 10000.0).round() / 10000.0);
    metrics.insert("非零覆盖".to_string(), (non_zero_cover * 10000.0).round() / 10000.0);
    metrics.insert("盈亏平衡点".to_string(), (cal_break_even_point(daily_returns) * 10000.0).round() / 10000.0);
    metrics.insert("新高间隔".to_string(), max_interval);
    metrics.insert("新高占比".to_string(), (high_pct * 10000.0).round() / 10000.0);
    metrics.insert("回撤风险".to_string(), (max_drawdown / annual_volatility * 10000.0).round() / 10000.0);

    metrics
}