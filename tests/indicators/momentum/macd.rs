//! Unit tests for MACD indicator

use perptrix::indicators::momentum::{calculate_macd, calculate_macd_default};
use perptrix::models::indicators::Candle;
use chrono::Utc;

fn create_test_candles(count: usize, base_price: f64) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let price = base_price + (i as f64 * 0.1);
        candles.push(Candle::new(
            price,
            price + 0.05,
            price - 0.05,
            price,
            1000.0,
            Utc::now(),
        ));
    }
    candles
}

#[test]
fn test_macd_insufficient_data() {
    let candles = create_test_candles(10, 100.0);
    assert!(calculate_macd(&candles, 12, 26, 9).is_none());
}

#[test]
fn test_macd_sufficient_data() {
    let candles = create_test_candles(50, 100.0);
    let result = calculate_macd_default(&candles);
    assert!(result.is_some());
    let macd = result.unwrap();
    assert!(macd.macd.is_finite());
    assert!(macd.signal.is_finite());
    assert!(macd.histogram.is_finite());
}

