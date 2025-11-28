//! Unit tests for the MACD momentum indicator.

use chrono::Utc;
use perptrix::indicators::momentum::calculate_macd_default;
use perptrix::indicators::momentum::macd::{MACDSignal, MACD};
use perptrix::models::indicators::Candle;

fn candle(price: f64) -> Candle {
    Candle::new(price, price + 0.5, price - 0.5, price, 500.0, Utc::now())
}

#[test]
fn macd_emits_crossovers() {
    let mut macd = MACD::new(3, 6, 3);
    let mut bullish = false;
    let mut bearish = false;

    for price in [100.0, 101.0, 102.0, 103.0, 104.0] {
        let (_, _, _, signal) = macd.update(price);
        if signal == MACDSignal::BullishCross {
            bullish = true;
        }
    }

    for price in [104.0, 103.0, 102.0, 101.0, 100.0, 99.0] {
        let (_, _, _, signal) = macd.update(price);
        if signal == MACDSignal::BearishCross {
            bearish = true;
        }
    }

    assert!(bullish, "Expected bullish crossover on rising prices");
    assert!(bearish, "Expected bearish crossover on falling prices");
}

#[test]
fn legacy_macd_calculation_available() {
    let candles: Vec<_> = (0..50).map(|i| candle(100.0 + i as f64)).collect();
    let indicator = calculate_macd_default(&candles).expect("MACD result");
    assert!(indicator.histogram.is_finite());
}
