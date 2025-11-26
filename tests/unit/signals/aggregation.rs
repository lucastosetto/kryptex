//! Unit tests for signal aggregation

use kryptex::indicators::registry::IndicatorCategory;
use kryptex::signals::aggregation::{Aggregator, IndicatorScore};

#[test]
fn test_aggregate_by_category() {
    let scores = vec![
        IndicatorScore {
            name: "RSI".to_string(),
            score: 0.5,
            category: IndicatorCategory::Momentum,
            weight: 1.0,
        },
        IndicatorScore {
            name: "MACD".to_string(),
            score: 0.3,
            category: IndicatorCategory::Momentum,
            weight: 1.0,
        },
    ];
    let category_scores = Aggregator::aggregate_by_category(&scores);
    assert_eq!(category_scores.len(), 1);
    assert_eq!(category_scores[0].0, IndicatorCategory::Momentum);
}

#[test]
fn test_calculate_global_score() {
    let category_scores = vec![
        (IndicatorCategory::Momentum, 0.5),
        (IndicatorCategory::Trend, 0.3),
    ];
    let global = Aggregator::calculate_global_score(&category_scores);
    assert!(global > 0.0);
}

