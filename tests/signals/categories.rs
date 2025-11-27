//! Unit tests for signal categories

use perptrix::indicators::registry::IndicatorCategory;
use perptrix::signals::categories::CategoryWeights;

#[test]
fn test_weights_sum_to_one() {
    assert!(CategoryWeights::verify());
}

#[test]
fn test_category_weights() {
    assert_eq!(CategoryWeights::get(IndicatorCategory::Momentum), 0.25);
    assert_eq!(CategoryWeights::get(IndicatorCategory::Trend), 0.35);
    assert_eq!(CategoryWeights::get(IndicatorCategory::Volatility), 0.20);
    assert_eq!(CategoryWeights::get(IndicatorCategory::MarketStructure), 0.20);
}

