use polars::prelude::*;
use crate::errors::BacktestError;
use crate::errors::BacktestError::Validation;

#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub digits: usize,
    pub fee_rate: f32,
    pub weight_type: String,
    pub yearly_days: usize,
    pub n_jobs: usize,
}

impl BacktestConfig {
    pub fn new(digits: usize, fee_rate: f32, weight_type: String, yearly_days: usize, n_jobs: usize) -> Self {
        BacktestConfig {
            digits,
            fee_rate,
            weight_type,
            yearly_days,
            n_jobs,
        }
    }
}

pub fn validate_dataframe(df: &DataFrame) -> Result<(), BacktestError> {
    let required_columns = ["dt", "symbol", "weight", "price"];
    for &col in &required_columns {
        if !df.column(col).is_ok() {
            return Err(Validation(
                format!("DataFrame is missing required column: {}", col)
            ));
        }
    }
    if df.height() == 0 {
        return Err(Validation(
            "DataFrame is empty".to_string(),
        ));
    }

    let total_nulls = df.get_columns()
        .iter()
        .map(|s| s.null_count())
        .sum::<usize>();
    if total_nulls > 0 {
        return Err(Validation("DataFrame contains null values".to_string()));
    }
    
    Ok(())
}