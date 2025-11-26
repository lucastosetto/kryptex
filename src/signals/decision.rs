//! Signal direction decision logic

use crate::models::signal::SignalDirection;

/// Direction thresholds as defined in the RFC
pub struct DirectionThresholds;

impl DirectionThresholds {
    pub const LONG_THRESHOLD: f64 = 0.60;
    pub const SHORT_THRESHOLD: f64 = 0.40;
    
    /// Determine signal direction from global score (0-1 range)
    pub fn determine_direction(global_score: f64) -> SignalDirection {
        if global_score > Self::LONG_THRESHOLD {
            SignalDirection::Long
        } else if global_score < Self::SHORT_THRESHOLD {
            SignalDirection::Short
        } else {
            SignalDirection::Neutral
        }
    }
    
    /// Convert normalized score (-1 to +1) to percentage (0 to 1)
    pub fn to_percentage(normalized_score: f64) -> f64 {
        (normalized_score + 1.0) / 2.0
    }
}

/// SL/TP calculation logic
pub struct StopLossTakeProfit;

impl StopLossTakeProfit {
    /// Calculate SL and TP from ATR
    /// SL = ATR * 1.2
    /// TP = ATR * 2.0
    pub fn calculate_from_atr(atr: f64, price: f64) -> (f64, f64) {
        let sl_pct = (atr * 1.2 / price) * 100.0;
        let tp_pct = (atr * 2.0 / price) * 100.0;
        (sl_pct, tp_pct)
    }
    
    /// Calculate SL and TP for Long position
    pub fn calculate_long(atr: f64, price: f64) -> (f64, f64) {
        Self::calculate_from_atr(atr, price)
    }
    
    /// Calculate SL and TP for Short position
    pub fn calculate_short(atr: f64, price: f64) -> (f64, f64) {
        Self::calculate_from_atr(atr, price)
    }
}


