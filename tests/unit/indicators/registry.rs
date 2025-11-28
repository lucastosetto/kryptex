//! Indicator registry and category organization tests

use perptrix::indicators::registry::{IndicatorCategory, IndicatorRegistry};

#[test]
fn test_category_weights_sum_to_one() {
    let registry = IndicatorRegistry::new();
    let total: f64 = IndicatorRegistry::all_categories()
        .iter()
        .map(|&cat| registry.category_weight(cat))
        .sum();
    assert!(
        (total - 1.0).abs() < 0.001,
        "Category weights should sum to 1.0, got {}",
        total
    );
}

#[test]
fn test_category_weights_match_rfc() {
    let registry = IndicatorRegistry::new();
    assert_eq!(registry.category_weight(IndicatorCategory::Momentum), 0.25);
    assert_eq!(registry.category_weight(IndicatorCategory::Trend), 0.30);
    assert_eq!(
        registry.category_weight(IndicatorCategory::Volatility),
        0.15
    );
    assert_eq!(registry.category_weight(IndicatorCategory::Volume), 0.15);
    assert_eq!(registry.category_weight(IndicatorCategory::Perp), 0.15);
}

#[test]
fn test_all_categories() {
    let categories = IndicatorRegistry::all_categories();
    assert_eq!(categories.len(), 5);
    assert!(categories.contains(&IndicatorCategory::Momentum));
    assert!(categories.contains(&IndicatorCategory::Trend));
    assert!(categories.contains(&IndicatorCategory::Volatility));
    assert!(categories.contains(&IndicatorCategory::Volume));
    assert!(categories.contains(&IndicatorCategory::Perp));
}

