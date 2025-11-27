//! Indicator registry and trait system

use crate::config::CategoryWeights;

/// Indicator category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndicatorCategory {
    Momentum,
    Trend,
    Volatility,
    Volume,
    Perp,
}

/// Trait for all indicators
pub trait Indicator {
    /// Get the category this indicator belongs to
    fn category(&self) -> IndicatorCategory;

    /// Get the name of the indicator
    fn name(&self) -> &'static str;
}

/// Indicator registry for organizing indicators by category
pub struct IndicatorRegistry {
    weights: CategoryWeights,
}

impl IndicatorRegistry {
    /// Create a new registry with default weights
    pub fn new() -> Self {
        Self {
            weights: CategoryWeights::default(),
        }
    }

    /// Create a new registry with custom weights
    pub fn with_weights(weights: CategoryWeights) -> Self {
        Self { weights }
    }

    /// Get category weight (as percentage)
    pub fn category_weight(&self, category: IndicatorCategory) -> f64 {
        match category {
            IndicatorCategory::Momentum => self.weights.momentum,
            IndicatorCategory::Trend => self.weights.trend,
            IndicatorCategory::Volatility => self.weights.volatility,
            IndicatorCategory::Volume => self.weights.volume,
            IndicatorCategory::Perp => self.weights.perp,
        }
    }

    /// Get all categories
    pub fn all_categories() -> Vec<IndicatorCategory> {
        vec![
            IndicatorCategory::Momentum,
            IndicatorCategory::Trend,
            IndicatorCategory::Volatility,
            IndicatorCategory::Volume,
            IndicatorCategory::Perp,
        ]
    }
}

impl Default for IndicatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
