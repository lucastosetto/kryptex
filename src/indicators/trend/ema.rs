//! EMA (Exponential Moving Average) indicator

use crate::common::math;
use crate::models::indicators::{Candle, EmaIndicator};

/// Calculate EMA for a specific period
pub fn calculate_ema(candles: &[Candle], period: u32) -> Option<EmaIndicator> {
    if candles.len() < period as usize {
        return None;
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let ema_value = math::ema(&closes, period as usize)?;

    Some(EmaIndicator {
        value: ema_value,
        period,
    })
}

/// Calculate multiple EMAs at once
pub fn calculate_emas(candles: &[Candle], periods: &[u32]) -> Vec<EmaIndicator> {
    periods
        .iter()
        .filter_map(|&period| calculate_ema(candles, period))
        .collect()
}

/// Check for EMA cross (e.g., EMA 12 crossing above/below EMA 26)
pub fn check_ema_cross(candles: &[Candle], fast_period: u32, slow_period: u32) -> Option<i32> {
    let fast_ema = calculate_ema(candles, fast_period)?;
    let slow_ema = calculate_ema(candles, slow_period)?;

    if fast_ema.value > slow_ema.value {
        Some(1) // Bullish cross
    } else if fast_ema.value < slow_ema.value {
        Some(-1) // Bearish cross
    } else {
        Some(0) // No cross
    }
}


