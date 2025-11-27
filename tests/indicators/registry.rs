//! Unit tests for indicator registry

use perptrix::indicators::registry::{IndicatorCategory, IndicatorRegistry};

#[test]
fn test_category_weights_sum_to_one() {
    let total: f64 = IndicatorRegistry::all_categories()
        .iter()
        .map(|&cat| IndicatorRegistry::category_weight(cat))
        .sum();
    assert!((total - 1.0).abs() < 0.001);
}

#[test]
fn test_category_weights() {
    assert_eq!(IndicatorRegistry::category_weight(IndicatorCategory::Momentum), 0.25);
    assert_eq!(IndicatorRegistry::category_weight(IndicatorCategory::Trend), 0.35);
    assert_eq!(IndicatorRegistry::category_weight(IndicatorCategory::Volatility), 0.20);
    assert_eq!(IndicatorRegistry::category_weight(IndicatorCategory::MarketStructure), 0.20);
}

