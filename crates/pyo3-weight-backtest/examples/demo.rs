use polars::prelude::*;
use weight_backtest_pyo3::{engine, utils};
use polars::prelude::*;
use polars::io::ipc::IpcReader;

fn read_feather_sync(path: &str) -> DataFrame {
    // 打开文件
    IpcReader::new(std::fs::File::open(path).expect("文件不存在"))
        .finish()
        .expect("Feather 格式错误")
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let df = read_feather_sync("/Users/i7xh/Downloads/weight_example.feather");

    let config = utils::BacktestConfig::new(
        2,                // digits
        0.0002,           // fee_rate
        "ts".to_string(), // weight_type
        252,              // yearly_days
        1,                // n_jobs
    );

    let daily_result = engine::data_processing::calculate_daily_results(df.clone(), "ZZUR9001", &config)?;    // 应用函数
    // let shifted = engine::data_processing::create_shifted_dataframes(&df)?;

    // 收集并打印结果
    println!("{:?}", daily_result.collect());

    Ok(())
}