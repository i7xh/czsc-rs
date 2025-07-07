use polars::io::ipc::IpcReader;
use polars::prelude::*;
use pyo3_polars::PyDataFrame;
use weight_backtest_pyo3::WeightBacktest;

// 生成基本测试数据
fn create_test_df() -> DataFrame {
    let dt = &[
        "2023-01-01 09:00:00",
        "2023-01-01 09:01:00",
        "2023-01-01 09:02:00",
        "2023-01-01 09:03:00",
    ];
    let symbol = &["AAPL", "AAPL", "AAPL", "AAPL"];
    let weight = &[0.5, 0.5, 0.0, 0.0];
    let price = &[100.0, 101.0, 102.0, 103.0];

    DataFrame::new(vec![
        Column::from(Series::new(PlSmallStr::from("dt"), dt)),
        Column::from(Series::new(PlSmallStr::from("symbol"), symbol)),
        Column::from(Series::new(PlSmallStr::from("weight"), weight)),
        Column::from(Series::new(PlSmallStr::from("price"), price)),
    ])
    .unwrap()
}
fn read_feather_sync(path: &str) -> DataFrame {
    // 打开文件
    IpcReader::new(std::fs::File::open(path).expect("文件不存在"))
        .finish()
        .expect("Feather 格式错误")
}

#[test]
fn test_backtest_engine_creation() {
    // let df = create_test_df();
    let df = read_feather_sync("/Users/i7xh/Downloads/weight_example.feather");

    let col_names = df.get_column_names();
    println!("方法1 - 所有列名: {:?}", col_names);

    let py_df = PyDataFrame(df);
    let engine = WeightBacktest::new(py_df, 2, &*"ts".to_string(), 0.0002, 252, 1);

    assert!(engine.is_ok());
    // let engine = engine.unwrap();
    // assert_eq!(engine.symbols, vec!["AAPL"]);
}
