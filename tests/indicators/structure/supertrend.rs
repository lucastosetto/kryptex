//! Unit tests for SuperTrend indicator

use kryptex::indicators::structure::{calculate_supertrend, calculate_supertrend_default};
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
            base + 0.3,
            base - 0.2,
            base + 0.1,
            1000.0,
            Utc::now(),
        ));
    }
    candles
}

#[test]
fn test_supertrend_insufficient_data() {
    let candles = create_trending_candles(10, true);
    assert!(calculate_supertrend(&candles, 10, 3.0).is_none());
}

#[test]
fn test_supertrend_sufficient_data() {
    let candles = create_trending_candles(30, true);
    let result = calculate_supertrend_default(&candles);
    assert!(result.is_some());
    let st = result.unwrap();
    assert!(st.value > 0.0);
    assert!(st.trend == 1 || st.trend == -1);
    assert_eq!(st.period, 10);
    assert_eq!(st.multiplier, 3.0);
}

