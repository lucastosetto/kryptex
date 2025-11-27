//! Unit tests for Support/Resistance indicator

use perptrix::indicators::structure::{calculate_support_resistance, calculate_support_resistance_default};
use perptrix::models::indicators::Candle;
use chrono::Utc;

fn create_range_candles(count: usize, min: f64, max: f64) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let price = min + ((i as f64 % 10.0) / 10.0) * (max - min);
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
fn test_support_resistance_insufficient_data() {
    let candles = create_range_candles(10, 100.0, 110.0);
    assert!(calculate_support_resistance(&candles, 20, 105.0).is_none());
}

#[test]
fn test_support_resistance_sufficient_data() {
    let candles = create_range_candles(50, 100.0, 110.0);
    let result = calculate_support_resistance_default(&candles, 105.0);
    assert!(result.is_some());
    let sr = result.unwrap();
    assert!(sr.support_level.is_some() || sr.resistance_level.is_some());
}

