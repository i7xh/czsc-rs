use crate::errors::{CzscError, CzscResult};
use crate::czsc_err;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeightType {
    TimeSeries,
    CrossSection,
}

#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub digits: usize,
    pub fee_rate: f32,
    pub weight_type: WeightType,
    pub yearly_days: usize,
    pub n_jobs: usize,
}

impl BacktestConfig {
    pub fn new(
        digits: usize,
        fee_rate: f32,
        weight_type: String,
        yearly_days: usize,
        n_jobs: usize,
    ) -> CzscResult<Self> {

        let weight_type_enum = match weight_type.to_lowercase().as_str() {
            "ts" => WeightType::TimeSeries,
            "cs" => WeightType::CrossSection,
            _ => return Err(czsc_err!(Unknown, "Invalid weight_type {:?}, must be 'ts' or 'cs'", weight_type)),
        };
        
        Ok(BacktestConfig {
            digits,
            fee_rate,
            weight_type: weight_type_enum,
            yearly_days,
            n_jobs,
        })
    }
}
