//! Unit tests for ATR volatility calculations.

use chrono::Utc;
use perptrix::indicators::volatility::atr::{VolatilityRegime, ATR};
use perptrix::indicators::volatility::calculate_atr_default;
use perptrix::models::indicators::Candle;

fn candle(high: f64, low: f64, close: f64) -> Candle {
    Candle::new(close, high, low, close, 1000.0, Utc::now())
}

#[test]
fn atr_updates_and_returns_regime() {
    let mut atr = ATR::new(5);
    let mut last_value = 0.0;

    for offset in 0..6 {
        last_value = atr.update(100.0 + offset as f64, 99.0 - offset as f64 * 0.1, 99.5);
    }

    assert!(last_value > 0.0);
    let regime = atr.get_volatility_regime(last_value, last_value / 2.0);
    assert_eq!(regime, VolatilityRegime::High);
}

#[test]
fn legacy_wrapper_remains() {
    let candles: Vec<_> = (0..20)
        .map(|i| candle(105.0 + i as f64 * 0.2, 95.0, 100.0))
        .collect();
    let indicator = calculate_atr_default(&candles).expect("ATR result");
    assert!(indicator.value > 0.0);
}
