
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub digits: usize,
    pub fee_rate: f32,
    pub weight_type: String,
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
    ) -> Self {
        BacktestConfig {
            digits,
            fee_rate,
            weight_type,
            yearly_days,
            n_jobs,
        }
    }
}
