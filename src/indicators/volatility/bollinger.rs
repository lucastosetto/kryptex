//! Bollinger Bands indicator

use crate::common::math;
use crate::models::indicators::{BollingerBandsIndicator, Candle};

/// Calculate Bollinger Bands
/// 
/// Middle Band = SMA(period)
/// Upper Band = Middle + (std_dev * standard deviation)
/// Lower Band = Middle - (std_dev * standard deviation)
pub fn calculate_bollinger_bands(
    candles: &[Candle],
    period: u32,
    std_dev: f64,
) -> Option<BollingerBandsIndicator> {
    if candles.len() < period as usize {
        return None;
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let middle = math::sma(&closes, period as usize)?;
    let std = math::standard_deviation(&closes, period as usize)?;

    let upper = middle + (std_dev * std);
    let lower = middle - (std_dev * std);

    Some(BollingerBandsIndicator {
        upper,
        middle,
        lower,
        period,
        std_dev,
    })
}

/// Calculate Bollinger Bands with default parameters (20 SMA, 2σ)
pub fn calculate_bollinger_bands_default(candles: &[Candle]) -> Option<BollingerBandsIndicator> {
    calculate_bollinger_bands(candles, 20, 2.0)
}


