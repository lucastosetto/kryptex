//! Indicator registry and category organization tests

use perptrix::indicators::registry::{IndicatorCategory, IndicatorRegistry};

#[test]
fn test_category_weights_sum_to_one() {
    let total: f64 = IndicatorRegistry::all_categories()
        .iter()
        .map(|&cat| IndicatorRegistry::category_weight(cat))
        .sum();
    assert!((total - 1.0).abs() < 0.001, "Category weights should sum to 1.0, got {}", total);
}

#[test]
fn test_category_weights_match_rfc() {
    assert_eq!(IndicatorRegistry::category_weight(IndicatorCategory::Momentum), 0.25);
    assert_eq!(IndicatorRegistry::category_weight(IndicatorCategory::Trend), 0.35);
    assert_eq!(IndicatorRegistry::category_weight(IndicatorCategory::Volatility), 0.20);
    assert_eq!(IndicatorRegistry::category_weight(IndicatorCategory::MarketStructure), 0.20);
}

#[test]
fn test_all_categories() {
    let categories = IndicatorRegistry::all_categories();
    assert_eq!(categories.len(), 4);
    assert!(categories.contains(&IndicatorCategory::Momentum));
    assert!(categories.contains(&IndicatorCategory::Trend));
    assert!(categories.contains(&IndicatorCategory::Volatility));
    assert!(categories.contains(&IndicatorCategory::MarketStructure));
}



