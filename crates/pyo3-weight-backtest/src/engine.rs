pub mod data_processing;

use polars::prelude::*;
use crate::errors::BacktestError;
use crate::utils::{validate_dataframe, BacktestConfig};


#[derive(Debug, Clone)]
struct BTEngine {
    config: BacktestConfig,
}

pub fn run_backtest(
    df: &mut DataFrame,
    backtest_config: &BacktestConfig,
) -> Result<(), BacktestError> {

    // Validate the DataFrame structure
    validate_dataframe(df)?;

    // process the 'weight' column with rounding
    data_processing::process_weight(df.clone(), backtest_config.digits)?;

    // 获取所有品种
    let symbols: Vec<String> = df.clone()
        .lazy()
        .select([col("symbol")])
        .unique(None, UniqueKeepStrategy::First)  // 获取唯一值
        .collect()?  // 执行计算
        .column("symbol")?
        .str()?
        .into_iter()
        .flatten()
        .map(|s| s.to_string())
        .collect();
    println!("Symbols: {:?}", symbols);

    Ok(())
}

fn process_symbol(symbol: &String, df: DataFrame, config: &BacktestConfig) -> Result<(), BacktestError> {
    let symbol_df = df
        .lazy()
        .filter(col("symbol").eq(lit(symbol.as_str())))
        .sort(["dt"], SortMultipleOptions::default().with_maintain_order(true))
        .collect()?;

    let daily_result = data_processing::calculate_daily_results(symbol_df.clone(), symbol, config)?;
    let trade_pairs = data_processing::get_symbol_pairs(symbol_df.clone(), symbol, config.clone())?;
    Ok(())
}
