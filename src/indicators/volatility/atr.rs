//! ATR (Average True Range) indicator

use crate::common::math;
use crate::models::indicators::{AtrIndicator, Candle};

/// Calculate ATR (Average True Range)
/// 
/// ATR measures market volatility by averaging true range over a period
pub fn calculate_atr(candles: &[Candle], period: u32) -> Option<AtrIndicator> {
    if candles.len() < period as usize + 1 {
        return None;
    }

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

    // ATR is typically calculated using smoothed moving average (Wilder's smoothing)
    // For simplicity, we'll use SMA here
    let atr_value = math::sma(&tr_values, period as usize)?;

    Some(AtrIndicator {
        value: atr_value,
        period,
    })
}

/// Calculate ATR with default period (14)
pub fn calculate_atr_default(candles: &[Candle]) -> Option<AtrIndicator> {
    calculate_atr(candles, 14)
}


