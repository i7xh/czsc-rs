use std::fmt::format;
use polars::datatypes::DataType;
use polars::error::PolarsResult;
use polars::frame::DataFrame;
use polars::prelude::*;
use polars::prelude::RoundMode::HalfAwayFromZero;
use crate::utils::BacktestConfig;


pub fn process_weight(df: DataFrame, digits: usize) -> PolarsResult<DataFrame> {
    Ok(df.lazy()
        .with_columns([
            col("weight")
                .cast(DataType::Float32)
                .round(digits as u32, HalfAwayFromZero)
                .alias("weight"),
        ]).collect()?)
}

// 计算每日结果
pub fn calculate_daily_results(
    df: DataFrame,
    symbol: &str,
    config: &BacktestConfig,
) -> PolarsResult<LazyFrame> {

    let lazy_df = df.lazy()
        .filter(col("symbol").eq(lit(symbol)))
        .with_column(
            (col("weight") * lit(10f64.powi(config.digits as i32))).cast(DataType::Int32).alias("volume"))
        .with_column(
            (lit(1)..lit(i32::MAX)).end.alias("bar_id")
        )
        .with_column(
            (col("price").shift(Expr::from(-1)) / col("price") - lit(1.0)).alias("n1b"),
        )
        .with_column(
            (col("weight") * col("n1b")).alias("edge"),
        )
        .with_column(
            (col("weight").shift(Expr::from(1)) - col("weight")).abs().alias("turnover"),
        )
        .with_column(
            (col("turnover") * lit(config.fee_rate)).alias("cost"),
        )
        .with_columns(
            [
                (col("edge") - col("cost")).alias("return"),
                when(col("weight").gt(0.0)).then(col("weight")).otherwise(lit(0.0)).alias("long_weight"),
                when(col("weight").lt(0.0)).then(col("weight")).otherwise(lit(0.0)).alias("short_weight"),
            ]
        )
        .with_columns(
            [
                (col("long_weight") * col("n1b")).alias("long_edge"),
                (col("short_weight") * col("n1b")).alias("short_edge"),
            ]
        )
        .with_columns(
            [
                (col("long_weight").shift(Expr::from(1)) - col("long_weight")).abs().alias("long_turnover"),
                (col("short_weight").shift(Expr::from(1)) - col("short_weight")).abs().alias("short_turnover"),
            ]
        )
        .with_columns(
            [
                (col("long_turnover") * lit(config.fee_rate)).alias("long_cost"),
                (col("short_turnover") * lit(config.fee_rate)).alias("short_cost"),
            ]
        )
        .with_columns(
            [
                (col("long_edge") - col("long_cost")).alias("long_return"),
                (col("short_edge") - col("short_cost")).alias("short_return"),
            ]
        );
    Ok(lazy_df)

}

pub fn get_symbol_pairs(df: DataFrame, symbol: &str, config: BacktestConfig) -> Result<(), PolarsError> {
    let new_df = df.clone().lazy()
        .filter(col("symbol").eq(lit(symbol)))
        .with_column(
            (col("weight") * lit(10f64.powi(config.digits as i32))).cast(DataType::Int32).alias("volume"))
        .with_column(
            (lit(1)..lit(i32::MAX)).end.alias("bar_id")
        )
        .collect()?;
    Ok(())
}
pub fn create_shifted_dataframes(df: &DataFrame) -> PolarsResult<LazyFrame> {
    let column_names: Vec<_> = df.get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // 创建前一行的DataFrame (rows[0..n-1])
    let prev_df = df
        .slice(0, df.height().saturating_sub(1))  // 安全处理空DataFrame情况
        .lazy()
        .select(
            column_names
                .iter()
                .map(|col_name| col(col_name).alias(&format!("{}_prev", col_name)))
                .collect::<Vec<_>>()
        );

    // 创建当前行的DataFrame (rows[1..n])
    let curr_df = df
        .slice(1, df.height().saturating_sub(1))  // 安全处理空DataFrame情况
        .lazy()
        .select(
            column_names
                .iter()
                .map(|col_name| col(col_name).alias(&format!("{}_curr", col_name)))
                .collect::<Vec<_>>()
        );

    // 水平拼接两个DataFrame
    Ok(concat_lf_horizontal(
        [prev_df, curr_df],
        UnionArgs {
            rechunk: false,  // 避免不必要的内存重分配
            parallel: true,  // 启用并行处理
            to_supertypes: false,  // 不自动转换类型
            ..Default::default()
        }
    )?)
}
