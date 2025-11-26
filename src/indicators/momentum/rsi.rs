//! RSI (Relative Strength Index) indicator

use crate::models::indicators::{Candle, RsiIndicator};

/// Calculate RSI indicator
/// 
/// RSI = 100 - (100 / (1 + RS))
/// RS = Average Gain / Average Loss
pub fn calculate_rsi(candles: &[Candle], period: u32) -> Option<RsiIndicator> {
    if candles.len() < period as usize + 1 {
        return None;
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..candles.len() {
        let change = candles[i].close - candles[i - 1].close;
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(change.abs());
        }
    }

    if gains.len() < period as usize {
        return None;
    }

    let avg_gain: f64 = gains.iter().rev().take(period as usize).sum::<f64>() / period as f64;
    let avg_loss: f64 = losses.iter().rev().take(period as usize).sum::<f64>() / period as f64;

    if avg_loss == 0.0 {
        return Some(RsiIndicator {
            value: 100.0,
            period: Some(period),
        });
    }

    let rs = avg_gain / avg_loss;
    let rsi = 100.0 - (100.0 / (1.0 + rs));

    Some(RsiIndicator {
        value: rsi,
        period: Some(period),
    })
}

/// Calculate RSI with default period (14)
pub fn calculate_rsi_default(candles: &[Candle]) -> Option<RsiIndicator> {
    calculate_rsi(candles, 14)
}


