use crate::indicators::macd::evaluation::{CrossoverType, MacdEvaluation};
use crate::indicators::macd::weights::MacdWeights;
use crate::models::indicators::MacdIndicator;

const CROSSOVER_THRESHOLD: f64 = 0.0001;
const DISTANCE_SCALE: f64 = 50.0;
const HISTOGRAM_SCALE: f64 = 25.0;

pub fn detect_crossover(macd: &MacdIndicator) -> (CrossoverType, f64) {
    let diff = macd.macd - macd.signal;
    
    if diff.abs() < CROSSOVER_THRESHOLD {
        (CrossoverType::None, 0.0)
    } else if diff > 0.0 {
        let strength = (diff.abs() / DISTANCE_SCALE).min(1.0);
        (CrossoverType::Bullish, strength)
    } else {
        let strength = (diff.abs() / DISTANCE_SCALE).min(1.0);
        (CrossoverType::Bearish, strength)
    }
}

pub fn calculate_distance_score(macd: &MacdIndicator) -> f64 {
    let distance = (macd.macd - macd.signal).abs();
    (distance / DISTANCE_SCALE).min(1.0)
}

pub fn calculate_histogram_momentum_score(macd: &MacdIndicator) -> f64 {
    let histogram_abs = macd.histogram.abs();
    (histogram_abs / HISTOGRAM_SCALE).min(1.0)
}

pub fn evaluate_macd(macd: &MacdIndicator, weights: &MacdWeights) -> MacdEvaluation {
    let (crossover_type, crossover_strength) = detect_crossover(macd);
    let distance = (macd.macd - macd.signal).abs();
    let distance_score = calculate_distance_score(macd);
    let histogram_momentum_score = calculate_histogram_momentum_score(macd);
    
    let crossover_score = if crossover_type != CrossoverType::None {
        crossover_strength
    } else {
        0.0
    };
    
    let overall_score = (crossover_score * weights.crossover_weight)
        + (distance_score * weights.distance_weight)
        + (histogram_momentum_score * weights.histogram_momentum_weight);
    
    MacdEvaluation::new(
        crossover_type,
        crossover_score,
        distance_score,
        histogram_momentum_score,
        overall_score,
        macd.macd,
        macd.signal,
        macd.histogram,
        distance,
    )
}

