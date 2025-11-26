//! MACD (Moving Average Convergence Divergence) indicator

use crate::common::math;
use crate::models::indicators::{Candle, MacdIndicator};

/// Calculate MACD indicator
/// 
/// MACD = EMA(12) - EMA(26)
/// Signal = EMA(9) of MACD
/// Histogram = MACD - Signal
pub fn calculate_macd(candles: &[Candle], fast_period: u32, slow_period: u32, signal_period: u32) -> Option<MacdIndicator> {
    if candles.len() < slow_period as usize + signal_period as usize {
        return None;
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    
    let fast_ema = math::ema(&closes, fast_period as usize)?;
    let slow_ema = math::ema(&closes, slow_period as usize)?;
    
    let macd_line = fast_ema - slow_ema;
    
    // Calculate signal line (EMA of MACD)
    // We need to build MACD values first
    let mut macd_values = Vec::new();
    let mut fast_ema_prev = math::sma(&closes[..fast_period as usize], fast_period as usize)?;
    let mut slow_ema_prev = math::sma(&closes[..slow_period as usize], slow_period as usize)?;
    
    for i in fast_period as usize..closes.len() {
        fast_ema_prev = math::ema_from_previous(closes[i], fast_ema_prev, fast_period as usize);
        
        if i >= slow_period as usize {
            slow_ema_prev = math::ema_from_previous(closes[i], slow_ema_prev, slow_period as usize);
            macd_values.push(fast_ema_prev - slow_ema_prev);
        }
    }
    
    if macd_values.len() < signal_period as usize {
        return None;
    }
    
    let signal_line = math::ema(&macd_values, signal_period as usize)?;
    let histogram = macd_line - signal_line;
    
    Some(MacdIndicator {
        macd: macd_line,
        signal: signal_line,
        histogram,
        period: Some((fast_period, slow_period, signal_period)),
    })
}

/// Calculate MACD with default periods (12, 26, 9)
pub fn calculate_macd_default(candles: &[Candle]) -> Option<MacdIndicator> {
    calculate_macd(candles, 12, 26, 9)
}


