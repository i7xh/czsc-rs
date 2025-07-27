use criterion::{criterion_group, criterion_main, Criterion};
use flame;
use polars::prelude::*;
use std::hint::black_box;
use weight_backtest_pyo3::config::BacktestConfig;
use weight_backtest_pyo3::engine::BacktestEngine;

fn read_feather_sync(path: &str) -> DataFrame {
    // 打开文件
    IpcReader::new(std::fs::File::open(path).expect("文件不存在"))
        .finish()
        .expect("Feather 格式错误")
}
fn backtest() -> Result<(), Box<dyn std::error::Error>> {

    // let _guard = flame::start_guard("backtest");

    let df = read_feather_sync("/Users/i7xh/works/a_share_daily_20170101_20250429.feather");

    let config = BacktestConfig::new(
        1,                // digits
        0.0002,           // fee_rate
        "ts".to_string(), // weight_type
        252,              // yearly_days
        50,               // n_jobs
    )?;

    let engine = BacktestEngine::new(df, config.clone())?;
    let _ = engine.run_backtest();
    // println!("engine: {:?}", engine);
    Ok(())
}

fn bench_backtest(c: &mut Criterion) {
    c.bench_function("engine backetest", |b| {
        b.iter(|| {
            // let _guard = flame::start_guard("iteration");
            black_box(backtest())
        }) // black_box 阻止编译器优化
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(50);     // 样本数量（默认100）
    targets = bench_backtest); // 定义测试组
criterion_main!(benches);             // 生成 main 函数