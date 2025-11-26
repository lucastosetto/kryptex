//! Unit tests for RSI indicator

use kryptex::indicators::momentum::{calculate_rsi, calculate_rsi_default};
use kryptex::models::indicators::Candle;
use chrono::Utc;

fn create_uptrend_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let price = 100.0 + (i as f64 * 0.5);
        candles.push(Candle::new(
            price,
            price + 0.1,
            price - 0.1,
            price,
            1000.0,
            Utc::now(),
        ));
    }
    candles
}

fn create_downtrend_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let price = 100.0 - (i as f64 * 0.5);
        candles.push(Candle::new(
            price,
            price + 0.1,
            price - 0.1,
            price,
            1000.0,
            Utc::now(),
        ));
    }
    candles
}

#[test]
fn test_rsi_insufficient_data() {
    let candles = create_uptrend_candles(10);
    assert!(calculate_rsi(&candles, 14).is_none());
}

#[test]
fn test_rsi_uptrend() {
    let candles = create_uptrend_candles(30);
    let result = calculate_rsi_default(&candles);
    assert!(result.is_some());
    let rsi = result.unwrap();
    assert!(rsi.value > 50.0);
    assert!(rsi.value <= 100.0);
}

#[test]
fn test_rsi_downtrend() {
    let candles = create_downtrend_candles(30);
    let result = calculate_rsi_default(&candles);
    assert!(result.is_some());
    let rsi = result.unwrap();
    assert!(rsi.value < 50.0);
    assert!(rsi.value >= 0.0);
}

