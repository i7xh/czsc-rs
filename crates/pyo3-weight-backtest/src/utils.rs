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

    let total_nulls = df
        .get_columns()
        .iter()
        .map(|s| s.null_count())
        .sum::<usize>();
    if total_nulls > 0 {
        return Err(Validation("DataFrame contains null values".to_string()));
    }

    Ok(())
}

pub fn union_lazyframes(
    lazy_dfs: Vec<LazyFrame>,
) -> CzscResult<LazyFrame> {
    if lazy_dfs.is_empty() {
        return Err(Validation("No LazyFrames to union".into()));
    }
    unimplemented!();
}