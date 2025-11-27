//! Unit tests for signal engine

use perptrix::signals::engine::SignalEngine;
use perptrix::models::indicators::Candle;
use chrono::Utc;

fn create_uptrend_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let price = 100.0 + (i as f64 * 0.5);
        candles.push(Candle::new(
            price,
            price + 0.3,
            price - 0.2,
            price + 0.1,
            1000.0,
            Utc::now(),
        ));
    }
    candles
}

#[test]
fn test_evaluate_insufficient_data() {
    let candles = create_uptrend_candles(10);
    assert!(SignalEngine::evaluate(&candles, "BTC").is_none());
}

#[test]
fn test_evaluate_sufficient_data() {
    let candles = create_uptrend_candles(250);
    let result = SignalEngine::evaluate(&candles, "BTC");
    assert!(result.is_some());
    let signal = result.unwrap();
    assert!(signal.confidence >= 0.0);
    assert!(signal.confidence <= 1.0);
}

