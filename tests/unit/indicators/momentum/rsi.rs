//! Unit tests for the RSI momentum indicator.

use chrono::Utc;
use perptrix::indicators::momentum::calculate_rsi_default;
use perptrix::indicators::momentum::rsi::{RSISignal, RSI};
use perptrix::models::indicators::Candle;

fn candle(price: f64) -> Candle {
    Candle::new(price, price + 0.3, price - 0.3, price, 800.0, Utc::now())
}

#[test]
fn rsi_detects_extremes() {
    let mut rsi = RSI::new(5);
    let mut prev_close = None;
    let mut saw_overbought = false;
    let mut saw_oversold = false;

    for price in [50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0] {
        if let Some(value) = rsi.update(price) {
            if let Some(prev) = prev_close {
                let signal = rsi.get_signal(value, price - prev);
                if signal == RSISignal::Overbought {
                    saw_overbought = true;
                }
            }
        }
        prev_close = Some(price);
    }

    for price in [56.0, 55.0, 54.0, 53.0, 52.0, 51.0, 50.0, 49.5] {
        if let Some(value) = rsi.update(price) {
            if let Some(prev) = prev_close {
                let signal = rsi.get_signal(value, price - prev);
                if signal == RSISignal::Oversold {
                    saw_oversold = true;
                }
            }
        }
        prev_close = Some(price);
    }

    assert!(saw_overbought, "RSI should flag overbought on a long rally");
    assert!(saw_oversold, "RSI should flag oversold on a long selloff");
}

#[test]
fn legacy_rsi_wrapper_still_available() {
    let candles: Vec<_> = (0..40).map(|i| candle(100.0 + i as f64 * 0.2)).collect();
    let indicator = calculate_rsi_default(&candles).expect("RSI result");
    assert!(indicator.value.is_finite());
}
