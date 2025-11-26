//! Support and Resistance levels detection

use crate::models::indicators::{Candle, SupportResistanceIndicator};

/// Calculate support and resistance levels
/// 
/// Finds local minima (support) and maxima (resistance) within a lookback window
pub fn calculate_support_resistance(
    candles: &[Candle],
    lookback: usize,
    current_price: f64,
) -> Option<SupportResistanceIndicator> {
    if candles.len() < lookback * 2 {
        return None;
    }

    let recent_candles = &candles[candles.len() - lookback..];
    
    // Find local minima (support) and maxima (resistance)
    let mut lows: Vec<f64> = recent_candles.iter().map(|c| c.low).collect();
    let mut highs: Vec<f64> = recent_candles.iter().map(|c| c.high).collect();
    
    lows.sort_by(|a, b| a.partial_cmp(b).unwrap());
    highs.sort_by(|a, b| b.partial_cmp(a).unwrap());
    
    // Use median of lowest/highest values as support/resistance
    let support_level = if lows.len() >= 3 {
        Some(lows[lows.len() / 3])
    } else {
        lows.first().copied()
    };
    
    let resistance_level = if highs.len() >= 3 {
        Some(highs[highs.len() / 3])
    } else {
        highs.first().copied()
    };
    
    let support_distance_pct = support_level.map(|support| {
        ((current_price - support) / current_price) * 100.0
    });
    
    let resistance_distance_pct = resistance_level.map(|resistance| {
        ((resistance - current_price) / current_price) * 100.0
    });
    
    Some(SupportResistanceIndicator {
        support_level,
        resistance_level,
        support_distance_pct,
        resistance_distance_pct,
    })
}

/// Calculate support/resistance with default lookback (20)
pub fn calculate_support_resistance_default(
    candles: &[Candle],
    current_price: f64,
) -> Option<SupportResistanceIndicator> {
    calculate_support_resistance(candles, 20, current_price)
}


