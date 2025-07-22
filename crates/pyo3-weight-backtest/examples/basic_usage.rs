use chrono::NaiveDate;
use polars::prelude::*;
use std::fmt::Debug;

fn sort_dataframe(df: &DataFrame) -> PolarsResult<DataFrame> {
    Ok(df
        .clone()
        .lazy()
        .sort(
            ["date"],
            SortMultipleOptions {
                descending: vec![false],
                nulls_last: vec![true],
                ..Default::default()
            },
        )
        .collect()?)
}

fn main() {
    let df = df![
        "date" => &["2023-01-15", "2023-01-10", "2017-01-20", "2023-01-05"],
        "value" => &[10.5, 20.3, 15.2, 5.7]
    ]
    .unwrap();

    println!("原始 DataFrame:\n{}", df);

    let options = StrptimeOptions {
        format: Some("%Y-%m-%d".into()), // 日期格式
        strict: true,                    // 严格解析
        exact: true,                     // 要求精确匹配
        ..Default::default()             // 其他参数使用默认值
    };

    let df = df
        .lazy()
        .with_columns([col("date")
            .str()
            .to_date(options) // 将字符串转换为日期
            .alias("date")])
        .collect()
        .unwrap();

    // 从 DataFrame 获取 DateChunked
    let date_ca = df.column("date").unwrap().date().unwrap();

    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    // 获取并展示日期范围
    if let Some(min) = date_ca.min() {
        let min_naive = epoch.checked_add_days(chrono::Days::new(min as u64)).unwrap();
        println!("\n数据集最小日期: {}", min_naive.format("%Y-%m-%d"));
    }

    if let Some(max) = date_ca.max() {
        let max_naive = epoch.checked_add_days(chrono::Days::new(max as u64)).unwrap();
        println!("数据集最大日期: {}", max_naive.format("%Y-%m-%d"));
    }

    println!("转换后的 DataFrame:\n{}", df);

    let xx = df
        .lazy()
        .select(&[
            col("date").max().alias("max_date"),
            col("date").min().alias("min_date"),
        ])
        .collect()
        .unwrap();

    println!("最大日期和最小日期:\n{}", xx);

}
