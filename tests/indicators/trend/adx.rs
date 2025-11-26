//! Unit tests for ADX indicator

use kryptex::indicators::trend::{calculate_adx, calculate_adx_default};
use kryptex::models::indicators::Candle;
use chrono::Utc;

fn create_trending_candles(count: usize, uptrend: bool) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let base = if uptrend {
            100.0 + (i as f64 * 0.5)
        } else {
            100.0 - (i as f64 * 0.5)
        };
        candles.push(Candle::new(
            base,
            base + 0.2,
            base - 0.1,
            base + 0.1,
            1000.0,
            Utc::now(),
        ));
    }
    candles
}

#[test]
fn test_adx_insufficient_data() {
    let candles = create_trending_candles(10, true);
    assert!(calculate_adx(&candles, 14).is_none());
}

#[test]
fn test_adx_sufficient_data() {
    let candles = create_trending_candles(30, true);
    let result = calculate_adx_default(&candles);
    assert!(result.is_some());
    let adx = result.unwrap();
    assert!(adx.value >= 0.0);
    assert!(adx.value <= 100.0);
    assert!(adx.plus_di >= 0.0);
    assert!(adx.minus_di >= 0.0);
}

