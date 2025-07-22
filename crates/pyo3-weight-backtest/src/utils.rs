use crate::errors::CzscError::Validation;
use crate::errors::CzscResult;
use polars::prelude::*;

pub fn validate_dataframe(df: &DataFrame) -> CzscResult<()> {
    let required_columns = ["dt", "symbol", "weight", "price"];
    for &col in &required_columns {
        if !df.column(col).is_ok() {
            return Err(Validation(format!(
                "DataFrame is missing required column: {}",
                col
            )));
        }
    }
    if df.height() == 0 {
        return Err(Validation("DataFrame is empty".to_string()));
    }

    let total_nulls = df.get_columns().iter().map(|s| s.null_count()).sum::<usize>();
    if total_nulls > 0 {
        return Err(Validation("DataFrame contains null values".to_string()));
    }

    Ok(())
}

pub trait RoundTo {
    fn round_to(&self, decimals: u32) -> f64;
}

impl RoundTo for f64 {
    fn round_to(&self, decimals: u32) -> f64 {
        let factor = 10_f64.powi(decimals as i32);
        (self * factor).round() / factor
    }
}

// 计算标准差
pub fn standard_deviation(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;

    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;

    variance.sqrt()
}
