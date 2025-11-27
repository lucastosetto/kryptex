//! Unit tests for ATR indicator

use perptrix::indicators::volatility::{calculate_atr, calculate_atr_default};
use perptrix::models::indicators::Candle;
use chrono::Utc;

fn create_volatile_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let base = 100.0 + (i as f64 * 0.1);
        let volatility = (i as f64 % 5.0) * 0.5;
        candles.push(Candle::new(
            base,
            base + volatility + 0.2,
            base - volatility - 0.2,
            base + volatility * 0.5,
            1000.0,
            Utc::now(),
        ));
    }
    candles
}

#[test]
fn test_atr_insufficient_data() {
    let candles = create_volatile_candles(10);
    assert!(calculate_atr(&candles, 14).is_none());
}

#[test]
fn test_atr_sufficient_data() {
    let candles = create_volatile_candles(30);
    let result = calculate_atr_default(&candles);
    assert!(result.is_some());
    let atr = result.unwrap();
    assert!(atr.value > 0.0);
    assert_eq!(atr.period, 14);
}



