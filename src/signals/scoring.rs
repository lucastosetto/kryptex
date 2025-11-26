//! Score normalization and confidence calculation

/// Normalize a value to -1 to +1 range
/// 
/// For indicators that output values in different ranges, this converts them
/// to a standardized -1 (bearish) to +1 (bullish) scale
pub fn normalize_score(value: f64, min: f64, max: f64) -> f64 {
    if max == min {
        return 0.0;
    }
    let normalized = 2.0 * ((value - min) / (max - min)) - 1.0;
    normalized.max(-1.0).min(1.0)
}

/// Normalize RSI (0-100) to -1 to +1
pub fn normalize_rsi(rsi: f64) -> f64 {
    normalize_score(rsi, 0.0, 100.0)
}

/// Normalize MACD histogram to -1 to +1
/// Uses a scaling factor based on typical MACD values
pub fn normalize_macd_histogram(histogram: f64, scale: f64) -> f64 {
    let normalized = histogram / scale;
    normalized.max(-1.0).min(1.0)
}

/// Normalize ADX (0-100) to -1 to +1
/// Higher ADX = stronger trend (positive score)
pub fn normalize_adx(adx: f64) -> f64 {
    normalize_score(adx, 0.0, 100.0)
}

/// Normalize EMA cross signal
/// Returns -1 for bearish cross, +1 for bullish cross, 0 for no cross
pub fn normalize_ema_cross(cross_signal: i32) -> f64 {
    cross_signal as f64
}

/// Normalize Bollinger Bands position
/// Returns -1 if price is at lower band, +1 if at upper band
pub fn normalize_bollinger_position(price: f64, lower: f64, upper: f64) -> f64 {
    if upper == lower {
        return 0.0;
    }
    normalize_score(price, lower, upper)
}

/// Normalize ATR (always positive, higher = more volatility)
/// Converts to a score where higher volatility = more uncertainty (closer to 0)
pub fn normalize_atr(atr: f64, price: f64) -> f64 {
    if price == 0.0 {
        return 0.0;
    }
    let atr_pct = (atr / price) * 100.0;
    // Higher volatility reduces confidence (moves toward 0)
    // This is a simplified approach - in practice, volatility interpretation depends on context
    -normalize_score(atr_pct, 0.0, 10.0).abs()
}

/// Normalize SuperTrend
/// Returns -1 for downtrend, +1 for uptrend
pub fn normalize_supertrend(trend: i32) -> f64 {
    trend as f64
}

/// Normalize support/resistance proximity
/// Returns positive if closer to support (potential bounce up), negative if closer to resistance
pub fn normalize_support_resistance(
    price: f64,
    support: Option<f64>,
    resistance: Option<f64>,
) -> f64 {
    match (support, resistance) {
        (Some(sup), Some(res)) => {
            let range = res - sup;
            if range == 0.0 {
                return 0.0;
            }
            let position = (price - sup) / range;
            // Closer to support = positive (bullish), closer to resistance = negative (bearish)
            normalize_score(position, 0.0, 1.0)
        }
        (Some(sup), None) => {
            let distance = (price - sup) / price;
            distance.max(-1.0).min(1.0)
        }
        (None, Some(res)) => {
            let distance = (res - price) / price;
            -distance.max(-1.0).min(1.0)
        }
        (None, None) => 0.0,
    }
}

/// Calculate confidence from normalized scores
/// Confidence is the absolute value of the weighted average score
pub fn calculate_confidence(global_score: f64) -> f64 {
    global_score.abs()
}


