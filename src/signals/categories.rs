//! Indicator category definitions and weights

use crate::indicators::registry::IndicatorCategory;

/// Category weights as defined in the RFC
pub struct CategoryWeights;

impl CategoryWeights {
    pub const MOMENTUM: f64 = 0.25;
    pub const TREND: f64 = 0.35;
    pub const VOLATILITY: f64 = 0.20;
    pub const MARKET_STRUCTURE: f64 = 0.20;

    /// Get weight for a category
    pub fn get(category: IndicatorCategory) -> f64 {
        match category {
            IndicatorCategory::Momentum => Self::MOMENTUM,
            IndicatorCategory::Trend => Self::TREND,
            IndicatorCategory::Volatility => Self::VOLATILITY,
            IndicatorCategory::MarketStructure => Self::MARKET_STRUCTURE,
        }
    }

    /// Verify weights sum to 1.0
    pub fn verify() -> bool {
        (Self::MOMENTUM + Self::TREND + Self::VOLATILITY + Self::MARKET_STRUCTURE - 1.0).abs() < 0.001
    }
}


