use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossoverType {
    Bullish,
    Bearish,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacdEvaluation {
    pub crossover_type: CrossoverType,
    pub crossover_score: f64,
    pub distance_score: f64,
    pub histogram_momentum_score: f64,
    pub overall_score: f64,
    pub macd_value: f64,
    pub signal_value: f64,
    pub histogram_value: f64,
    pub distance: f64,
}

impl MacdEvaluation {
    pub fn new(
        crossover_type: CrossoverType,
        crossover_score: f64,
        distance_score: f64,
        histogram_momentum_score: f64,
        overall_score: f64,
        macd_value: f64,
        signal_value: f64,
        histogram_value: f64,
        distance: f64,
    ) -> Self {
        Self {
            crossover_type,
            crossover_score,
            distance_score,
            histogram_momentum_score,
            overall_score,
            macd_value,
            signal_value,
            histogram_value,
            distance,
        }
    }
}

