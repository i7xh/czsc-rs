use polars::prelude::*;
use std::collections::HashMap;
use pyo3::impl_::wrap::SomeWrap;

// 计算盈亏平衡点的辅助函数
fn cal_break_even_point(seq: &[f64]) -> f64 {
    // 如果总收益为负，直接返回 1.0
    if seq.iter().sum::<f64>() < 0.0 {
        return 1.0;
    }

    // 创建可排序的副本
    let mut sorted_seq = seq.to_vec();
    sorted_seq.sort_by(|a, b| a.partial_cmp(b).unwrap()); // 从小到大排序

    // 计算累积和
    let mut cum_sum = 0.0;
    let mut negative_count = 0;

    for &value in &sorted_seq {
        cum_sum += value;
        if cum_sum < 0.0 {
            negative_count += 1;
        }
    }

    // 计算盈亏平衡点
    (negative_count as f64 + 1.0) / seq.len() as f64
}

// 评估开平交易记录的表现
pub fn evaluate_pairs(pairs: &DataFrame) -> HashMap<String, f64> {
    
    let trade_dir = pairs.column("交易方向")
        .unwrap()
        .str()
        .unwrap()
        .get(0)
        .unwrap_or("多空");

    let mut p: HashMap<String, f64> = HashMap::new();
    p.insert("交易方向".to_string(), if trade_dir == "多头" {
        1.0
    } else if trade_dir == "空头" {
        -1.0
    } else {
        0.0
    });
    p.insert("交易次数".to_string(), 0.0);
    p.insert("累计收益".to_string(), 0.0);
    p.insert("单笔收益".to_string(), 0.0);
    p.insert("盈利次数".to_string(), 0.0);
    p.insert("累计盈利".to_string(), 0.0);
    p.insert("单笔盈利".to_string(), 0.0);
    p.insert("亏损次数".to_string(), 0.0);
    p.insert("累计亏损".to_string(), 0.0);
    p.insert("单笔亏损".to_string(), 0.0);
    p.insert("交易胜率".to_string(), 0.0);
    p.insert("累计盈亏比".to_string(), 0.0);
    p.insert("单笔盈亏比".to_string(), 0.0);
    p.insert("盈亏平衡点".to_string(), 1.0);
    p.insert("持仓天数".to_string(), 0.0);
    p.insert("持仓K线数".to_string(), 0.0);

    if pairs.height() == 0 {
        return p;
    }

    // 筛选交易方向
    let filtered_pairs = if trade_dir != "多空" {
        let dir_filter = pairs.column("交易方向").unwrap()
            .str()
            .unwrap()
            .equal(trade_dir);

        match pairs.filter(&dir_filter) {
            Ok(df) => df,
            Err(_) => return p,
        }
    } else {
        pairs.clone()
    };

    if filtered_pairs.height() == 0 {
        return p;
    }

    // 获取盈亏比例列
    let returns = filtered_pairs.column("盈亏比例")
        .unwrap()
        .f64()
        .unwrap();

    // 获取持仓天数和持仓K线数列
    let hold_days = filtered_pairs.column("持仓天数")
        .unwrap()
        .f64()
        .unwrap();
    let hold_bars = filtered_pairs.column("持仓K线数")
        .unwrap()
        .f64()
        .unwrap();

    let n = filtered_pairs.height() as f64;
    p.insert("交易次数".to_string(), n);

    // 计算盈亏平衡点
    let returns_vec: Vec<f64> = returns.into_no_null_iter().collect();
    p.insert("盈亏平衡点".to_string(), cal_break_even_point(&returns_vec).round());

    // 计算累计收益和单笔收益
    let total_return: f64 = returns.sum().unwrap_or(0.0);
    p.insert("累计收益".to_string(), (total_return * 100.0).round() / 100.0); // 保留两位小数
    p.insert("单笔收益".to_string(), (total_return / n * 100.0).round() / 100.0);

    // 计算平均持仓天数和K线数
    let avg_hold_days = hold_days.sum().unwrap_or(0.0) / n;
    let avg_hold_bars = hold_bars.sum().unwrap_or(0.0) / n;
    p.insert("持仓天数".to_string(), (avg_hold_days * 100.0).round() / 100.0);
    p.insert("持仓K线数".to_string(), (avg_hold_bars * 100.0).round() / 100.0);

    // 分离盈利和亏损交易
    let x = returns.into_iter();
    let win_trades: Vec<f64> = returns.into_iter()
        .filter_map(|x| x.and_then(|v| 
            if v >= 0.0 { Some(v) } else { None }))
        .collect();
    let loss_trades: Vec<f64> = returns.into_iter()
        .filter_map(|x| x.and_then(|v| 
            if v < 0.0 { Some(v) } else { None }))
        .collect();

    let win_count = win_trades.len() as f64;
    let loss_count = loss_trades.len() as f64;

    if win_count > 0.0 {
        let win_total: f64 = win_trades.iter().sum();
        let avg_win = win_total / win_count;

        p.insert("盈利次数".to_string(), win_count);
        p.insert("累计盈利".to_string(), win_total);
        p.insert("单笔盈利".to_string(), (avg_win * 10000.0).round() / 10000.0); // 保留4位小数
        p.insert("交易胜率".to_string(), (win_count / n * 10000.0).round() / 10000.0);
    }

    if loss_count > 0.0 {
        let loss_total: f64 = loss_trades.iter().sum();
        let avg_loss = loss_total / loss_count;

        p.insert("亏损次数".to_string(), loss_count);
        p.insert("累计亏损".to_string(), loss_total);
        p.insert("单笔亏损".to_string(), (avg_loss * 10000.0).round() / 10000.0);

        // 计算盈亏比
        if let (Some(&win_total), Some(&loss_total)) = (p.get("累计盈利"), p.get("累计亏损")) {
            let total_ratio = win_total / loss_total.abs();
            let per_trade_ratio = if let (Some(avg_win), Some(avg_loss)) = (p.get("单笔盈利"), p.get("单笔亏损")) {
                avg_win / avg_loss.abs()
            } else {
                0.0
            };

            p.insert("累计盈亏比".to_string(), (total_ratio * 10000.0).round() / 10000.0);
            p.insert("单笔盈亏比".to_string(), (per_trade_ratio * 10000.0).round() / 10000.0);
        }
    }

    p
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