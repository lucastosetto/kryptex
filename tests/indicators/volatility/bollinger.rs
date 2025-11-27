//! Unit tests for Bollinger Bands indicator

use perptrix::indicators::volatility::{calculate_bollinger_bands, calculate_bollinger_bands_default};
use perptrix::models::indicators::Candle;
use chrono::Utc;

fn create_test_candles(count: usize, base_price: f64, volatility: f64) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let price = base_price + (i as f64 * 0.1) + (volatility * (i as f64 % 3.0 - 1.0));
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
fn test_bollinger_insufficient_data() {
    let candles = create_test_candles(10, 100.0, 0.5);
    assert!(calculate_bollinger_bands(&candles, 20, 2.0).is_none());
}

#[test]
fn test_bollinger_sufficient_data() {
    let candles = create_test_candles(50, 100.0, 0.5);
    let result = calculate_bollinger_bands_default(&candles);
    assert!(result.is_some());
    let bb = result.unwrap();
    assert!(bb.upper > bb.middle);
    assert!(bb.middle > bb.lower);
    assert_eq!(bb.period, 20);
    assert_eq!(bb.std_dev, 2.0);
}

