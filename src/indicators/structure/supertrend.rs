//! SuperTrend indicator

use crate::common::math;
use crate::models::indicators::{Candle, SuperTrendIndicator};

/// Calculate SuperTrend indicator
/// 
/// SuperTrend is a trend-following indicator that uses ATR
/// trend: 1 for uptrend, -1 for downtrend
pub fn calculate_supertrend(
    candles: &[Candle],
    period: u32,
    multiplier: f64,
) -> Option<SuperTrendIndicator> {
    if candles.len() < period as usize + 1 {
        return None;
    }

    // Calculate ATR
    let mut tr_values = Vec::new();
    for i in 1..candles.len() {
        let tr = math::true_range(
            candles[i].high,
            candles[i].low,
            candles[i - 1].close,
        );
        tr_values.push(tr);
    }

    if tr_values.len() < period as usize {
        return None;
    }

    let atr = math::sma(&tr_values, period as usize)?;

    // Calculate basic bands
    let hl2 = (candles.last()?.high + candles.last()?.low) / 2.0;
    let upper_band = hl2 + (multiplier * atr);
    let lower_band = hl2 - (multiplier * atr);

    // Determine trend
    let current_price = candles.last()?.close;
    let trend = if current_price > upper_band {
        1 // Uptrend
    } else if current_price < lower_band {
        -1 // Downtrend
    } else {
        // Use previous trend if price is between bands
        // For simplicity, we'll use price position relative to hl2
        if current_price > hl2 {
            1
        } else {
            -1
        }
    };

    let supertrend_value = if trend == 1 {
        lower_band
    } else {
        upper_band
    };

    Some(SuperTrendIndicator {
        value: supertrend_value,
        trend,
        upper_band,
        lower_band,
        period,
        multiplier,
    })
}

/// Calculate SuperTrend with default parameters (10, 3)
pub fn calculate_supertrend_default(candles: &[Candle]) -> Option<SuperTrendIndicator> {
    calculate_supertrend(candles, 10, 3.0)
}


