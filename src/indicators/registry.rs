//! Indicator registry and trait system

/// Indicator category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndicatorCategory {
    Momentum,
    Trend,
    Volatility,
    MarketStructure,
}

/// Trait for all indicators
pub trait Indicator {
    /// Get the category this indicator belongs to
    fn category(&self) -> IndicatorCategory;
    
    /// Get the name of the indicator
    fn name(&self) -> &'static str;
}

/// Indicator registry for organizing indicators by category
pub struct IndicatorRegistry;

impl IndicatorRegistry {
    /// Get category weight (as percentage)
    pub fn category_weight(category: IndicatorCategory) -> f64 {
        match category {
            IndicatorCategory::Momentum => 0.25,
            IndicatorCategory::Trend => 0.35,
            IndicatorCategory::Volatility => 0.20,
            IndicatorCategory::MarketStructure => 0.20,
        }
    }
    
    /// Get all categories
    pub fn all_categories() -> Vec<IndicatorCategory> {
        vec![
            IndicatorCategory::Momentum,
            IndicatorCategory::Trend,
            IndicatorCategory::Volatility,
            IndicatorCategory::MarketStructure,
        ]
    }
}


