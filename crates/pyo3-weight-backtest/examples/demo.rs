use polars::io::ipc::IpcReader;
use polars::prelude::*;
use weight_backtest_pyo3::config::BacktestConfig;
use weight_backtest_pyo3::engine::BacktestEngine;

fn read_feather_sync(path: &str) -> DataFrame {
    // 打开文件
    IpcReader::new(std::fs::File::open(path).expect("文件不存在"))
        .finish()
        .expect("Feather 格式错误")
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let df = read_feather_sync("/Users/i7xh/Downloads/weight_example.feather");
    let df = read_feather_sync("/Users/i7xh/works/a_share_daily_20170101_20250429.feather");

    let config = BacktestConfig::new(
        1,                // digits
        0.0002,           // fee_rate
        "ts".to_string(), // weight_type
        252,              // yearly_days
        1,               // n_jobs
    )?;

    let engine = BacktestEngine::new(df, config.clone())?;
    println!("engine: {:?}", engine);
    let r = engine.run_backtest();
    // let daily_result = weight_backtest_pyo3::data_processing::calc_daily_results(df.clone(), "ZZUR9001", &config)?.collect()?;    // 应用函数
    // let shifted = engine::data_processing::gen_trade_pairs(df, "ZZUR9001", &config)?;

    unsafe {
        std::env::set_var("POLARS_FMT_MAX_COLS", "100");
    }
    // println!("Shifted DataFrame:\n{:?}", daily_result);

    Ok(())
}
