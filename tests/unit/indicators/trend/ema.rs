//! Unit tests for EMA indicator

use perptrix::indicators::trend::{calculate_ema, calculate_emas, check_ema_cross};
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
fn test_ema_insufficient_data() {
    let candles = create_test_candles(10, 100.0);
    assert!(calculate_ema(&candles, 20).is_none());
}

#[test]
fn test_ema_sufficient_data() {
    let candles = create_test_candles(50, 100.0);
    let result = calculate_ema(&candles, 12);
    assert!(result.is_some());
    let ema = result.unwrap();
    assert_eq!(ema.period, 12);
    assert!(ema.value.is_finite());
}

#[test]
fn test_calculate_multiple_emas() {
    let candles = create_test_candles(250, 100.0);
    let periods = vec![12, 26, 50, 200];
    let emas = calculate_emas(&candles, &periods);
    assert_eq!(emas.len(), 4);
}

#[test]
fn test_ema_cross() {
    let candles = create_test_candles(50, 100.0);
    let cross = check_ema_cross(&candles, 12, 26);
    assert!(cross.is_some());
}

