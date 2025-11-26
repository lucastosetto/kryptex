//! Mathematical utilities for indicator calculations.

/// Calculate Simple Moving Average (SMA)
pub fn sma(values: &[f64], period: usize) -> Option<f64> {
    if values.len() < period {
        return None;
    }
    let sum: f64 = values.iter().rev().take(period).sum();
    Some(sum / period as f64)
}

/// Calculate Exponential Moving Average (EMA)
pub fn ema(values: &[f64], period: usize) -> Option<f64> {
    if values.is_empty() || period == 0 {
        return None;
    }
    
    if values.len() < period {
        return None;
    }
    
    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema_value = sma(&values[..period], period)?;
    
    for &value in values.iter().skip(period) {
        ema_value = (value - ema_value) * multiplier + ema_value;
    }
    
    Some(ema_value)
}

/// Calculate EMA from a starting EMA value
pub fn ema_from_previous(current_value: f64, previous_ema: f64, period: usize) -> f64 {
    let multiplier = 2.0 / (period as f64 + 1.0);
    (current_value - previous_ema) * multiplier + previous_ema
}

/// Calculate standard deviation
pub fn standard_deviation(values: &[f64], period: usize) -> Option<f64> {
    if values.len() < period {
        return None;
    }
    
    let mean = sma(values, period)?;
    let variance: f64 = values
        .iter()
        .rev()
        .take(period)
        .map(|&x| {
            let diff = x - mean;
            diff * diff
        })
        .sum::<f64>()
        / period as f64;
    
    Some(variance.sqrt())
}

/// Calculate True Range (TR) for a single candle
pub fn true_range(current_high: f64, current_low: f64, previous_close: f64) -> f64 {
    let hl = current_high - current_low;
    let hc = (current_high - previous_close).abs();
    let lc = (current_low - previous_close).abs();
    hl.max(hc).max(lc)
}


