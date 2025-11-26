//! ADX (Average Directional Index) indicator

use crate::common::math;
use crate::models::indicators::{AdxIndicator, Candle};

/// Calculate ADX indicator
/// 
/// ADX measures trend strength regardless of direction
/// Requires calculation of +DI and -DI first
pub fn calculate_adx(candles: &[Candle], period: u32) -> Option<AdxIndicator> {
    if candles.len() < period as usize + 1 {
        return None;
    }

    let mut tr_values = Vec::new();
    let mut plus_dm_values = Vec::new();
    let mut minus_dm_values = Vec::new();

    for i in 1..candles.len() {
        let tr = math::true_range(
            candles[i].high,
            candles[i].low,
            candles[i - 1].close,
        );
        tr_values.push(tr);

        let plus_dm = if candles[i].high > candles[i - 1].high {
            candles[i].high - candles[i - 1].high
        } else {
            0.0
        };
        plus_dm_values.push(plus_dm);

        let minus_dm = if candles[i].low < candles[i - 1].low {
            candles[i - 1].low - candles[i].low
        } else {
            0.0
        };
        minus_dm_values.push(minus_dm);
    }

    if tr_values.len() < period as usize {
        return None;
    }

    // Calculate smoothed TR, +DM, -DM
    let atr = math::sma(&tr_values, period as usize)?;
    let plus_dm_avg = math::sma(&plus_dm_values, period as usize)?;
    let minus_dm_avg = math::sma(&minus_dm_values, period as usize)?;

    // Calculate +DI and -DI
    let plus_di = if atr > 0.0 {
        100.0 * (plus_dm_avg / atr)
    } else {
        0.0
    };

    let minus_di = if atr > 0.0 {
        100.0 * (minus_dm_avg / atr)
    } else {
        0.0
    };

    // Calculate DX
    let di_sum = plus_di + minus_di;
    let dx = if di_sum > 0.0 {
        100.0 * ((plus_di - minus_di).abs() / di_sum)
    } else {
        0.0
    };

    // ADX is smoothed DX (using EMA)
    let dx_values = vec![dx];
    let adx_value = math::ema(&dx_values, period as usize).unwrap_or(dx);

    Some(AdxIndicator {
        value: adx_value,
        plus_di,
        minus_di,
        period,
    })
}

/// Calculate ADX with default period (14)
pub fn calculate_adx_default(candles: &[Candle]) -> Option<AdxIndicator> {
    calculate_adx(candles, 14)
}


