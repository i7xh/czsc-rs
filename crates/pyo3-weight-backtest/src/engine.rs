use std::collections::HashMap;
use anyhow::Context; // 引入 Context trait
use polars::prelude::*;
use polars::prelude::RoundMode::HalfAwayFromZero;
use crate::calculator::MetricCalculator;
use crate::errors::CzscResult;
use crate::utils::validate_dataframe;
use crate::config::BacktestConfig;
use crate::types::SymbolResult;

#[derive(Debug, Clone)]
pub struct BacktestEngine {
    config: BacktestConfig,
    df: DataFrame,
    symbols: Vec<String>,
    calculator: MetricCalculator,
}

impl BacktestEngine {
    pub fn new(df: DataFrame, config: BacktestConfig) -> CzscResult<Self> {

        // 数据检验
        validate_dataframe(&df).context("DataFrame validation")?;

        // 获取 symbols
        let symbols = df
            .column("symbol")?
            .str()?
            .into_iter()
            .flatten()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // 预处理 dataframe
        let prepared_df= df.lazy()
            .with_columns([col("weight")
                .cast(DataType::Float32)
                .round(config.digits as u32, HalfAwayFromZero)
                .alias("weight")])
            .sort(["dt"], SortMultipleOptions::default())
            .collect()?;
        
        let calculator = MetricCalculator::new(config.clone());

        Ok(Self { config, df: prepared_df, symbols, calculator})
    }

    pub fn run(&self) -> CzscResult<()> {
        let n_jobs = self.config.n_jobs;

        if n_jobs > 1 {
            // 多线程处理
            self.run_parallel();
        } else {
            // 单线程处理
            self.run_sequential();
        }
        unimplemented!()
    }

    fn run_sequential(&self) -> CzscResult<HashMap<String, SymbolResult>> {
        
        let mut results: HashMap<String, SymbolResult> = HashMap::new();
        
        for symbol in self.symbols.iter() {
            let sr = self.process_symbol(symbol)?;
            results.insert(symbol.clone(), sr);
        }
        Ok(results)
    }

    fn run_parallel(&self) -> CzscResult<()> {
        unimplemented!()
    }
    
    fn process_symbol(
        &self,
        symbol: &String,
    ) -> CzscResult<SymbolResult> {
        // 过滤出当前 symbol 的数据
        let symbol_df = self.df.clone()
            .lazy()
            .filter(col("symbol").eq(lit(symbol.clone())))
            .collect()?;
        
        // 生成每日结果
        let daily_df = self.calculator.calculate_daily_metrics(symbol, &symbol_df)?;
        // 生成交易对
        let trade_pairs = self.calculator.generate_trade_pairs(symbol, &symbol_df)?;
        
        unimplemented!();
    }
}

