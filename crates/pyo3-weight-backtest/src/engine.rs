use crate::analyzer::PortfolioAnalyzer;
use crate::config::BacktestConfig;
use crate::errors::CzscResult;
use crate::processor::MetricProcessor;
use crate::types::SymbolResult;
use crate::utils::validate_dataframe;
use anyhow::Context;
// 引入 Context trait
use indicatif::{ProgressBar, ProgressStyle};
use polars::prelude::RoundMode::HalfAwayFromZero;
use polars::prelude::*;
use pyo3::pyclass;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct BacktestEngine {
    config   : BacktestConfig,
    df       : DataFrame,
    symbols  : Vec<String>,
    processor: MetricProcessor,
}

#[pyclass]
pub struct BacktestResult {
    pub symbol_results    : HashMap<String, SymbolResult>,
    pub portfolio_metrics : HashMap<String, f64>,
    pub daily_ew_return_df: DataFrame,
}

impl BacktestEngine {
    pub fn new(df: DataFrame, config: BacktestConfig) -> CzscResult<Self> {
        // 数据检验
        validate_dataframe(&df).context("DataFrame validation")?;

        // 获取 symbols
        let symbols = df
            .column("symbol")?
            .str()?
            .unique()?
            .into_iter()
            .flatten()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        println!("Symbols found: {:?}", symbols);

        // 预处理 dataframe
        let prepared_df = df
            .lazy()
            .with_columns([col("weight")
                .cast(DataType::Float64)
                .round(config.digits as u32, HalfAwayFromZero)
                .alias("weight")])
            .with_column(
                (col("weight") * lit(10f64.powi(config.digits as i32)))
                    .cast(DataType::Int32)
                    .alias("volume"),
            )
            .sort(["dt"], SortMultipleOptions::default())
            .with_row_index("bar_id", Some(0))
            .collect()?;

        let processor = MetricProcessor::new(config.clone());

        Ok(Self {
            config,
            df: prepared_df,
            symbols,
            processor,
        })
    }

    pub fn run_backtest(&self) -> CzscResult<(BacktestResult)> {
        let symbol_results = if self.config.n_jobs > 1 {
            // 多线程处理
            self.run_parallel()?
        } else {
            // 单线程处理
            self.run_sequential()?
        };

        let daily_df = PortfolioAnalyzer::gen_daily_metric_df(&symbol_results);
        let daily_ew_return_df = PortfolioAnalyzer::gen_daily_ew_return_df(
            self.config.weight_type,
            &symbol_results,
            &daily_df,
        )?;

        // 计算组合指标
        let analyzer = PortfolioAnalyzer::new(
            self.config.clone(),
            &symbol_results,
            &self.df,
            &daily_df,
            &daily_ew_return_df,
        );

        let metrics = analyzer.analyze_portfolio_metrics()?;

        Ok(BacktestResult {
            symbol_results,
            portfolio_metrics: metrics,
            daily_ew_return_df: daily_ew_return_df,
        })
    }

    fn run_sequential(&self) -> CzscResult<HashMap<String, SymbolResult>> {
        let pb = ProgressBar::new(self.symbols.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("##-"),
        );

        let mut results = HashMap::new();

        for symbol in &self.symbols {
            let sr = self.process_symbol(symbol)?;
            results.insert(symbol.to_string(), sr);
            pb.inc(1); // 更新进度条
            pb.set_message(symbol.to_string());
        }

        pb.finish_with_message("完成");
        Ok(results)
    }

    fn run_parallel(&self) -> CzscResult<HashMap<String, SymbolResult>> {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build()?;

        // 创建进度条
        let pb = ProgressBar::new(self.symbols.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("##-"),
        );

        // 使用 Mutex 包装进度条以便在并行环境中安全更新
        let pb_mutex = Mutex::new(pb);

        pool.install(|| {
            let results: Vec<CzscResult<(String, SymbolResult)>> = self
                .symbols
                .iter()
                .map(|symbol| {
                    // 处理当前 symbol
                    let result = self.process_symbol(symbol).map(|sr| (symbol.clone(), sr));

                    // 更新进度条（互斥访问）
                    if let Ok(ref mut pb) = pb_mutex.lock() {
                        pb.inc(1);
                    }

                    result
                })
                .collect();

            // 完成进度条
            if let Ok(pb) = pb_mutex.lock() {
                pb.finish();
            }

            // 处理结果
            let pairs: CzscResult<Vec<(String, SymbolResult)>> = results.into_iter().collect();
            pairs.map(|pairs| pairs.into_iter().collect())
        })
    }

    fn process_symbol(&self, symbol: &String) -> CzscResult<SymbolResult> {
        // 过滤出当前 symbol 的数据
        let symbol_df =
            self.df.clone().lazy().filter(col("symbol").eq(lit(symbol.clone()))).collect()?;

        let column_names = symbol_df.get_column_names();
        println!("Processing symbol: {}, columns: {:?}", symbol, column_names);

        // 生成每日结果
        let daily_metrics = self.processor.process_daily_metrics(symbol, &symbol_df)?;
        // 生成交易对
        let trade_pairs = self.processor.generate_trade_pairs(symbol, &symbol_df)?;

        Ok(SymbolResult {
            daily_metrics,
            trade_pairs,
        })
    }
}
