//! Unit tests for Bollinger Bands signals.

use chrono::Utc;
use perptrix::indicators::volatility::bollinger::{BollingerBands, BollingerSignal};
use perptrix::indicators::volatility::calculate_bollinger_bands_default;
use perptrix::models::indicators::Candle;

fn candle(price: f64) -> Candle {
    Candle::new(price, price + 0.5, price - 0.5, price, 1200.0, Utc::now())
}

#[test]
fn bollinger_detects_squeeze_and_breakout() {
    let mut bb = BollingerBands::new(5, 2.0);
    let mut squeeze = false;
    let mut breakout = false;

    for _ in 0..6 {
        let (_, _, _, signal) = bb.update(100.0);
        if signal == BollingerSignal::Squeeze {
            squeeze = true;
        }
    }

    let (_, _, _, signal) = bb.update(105.0);
    if matches!(
        signal,
        BollingerSignal::UpperBreakout | BollingerSignal::WalkingBands
    ) {
        breakout = true;
    }

    assert!(squeeze, "Expected squeeze after flat volatility");
    assert!(breakout, "Expected breakout when price rips above the band");
}

#[test]
fn legacy_wrapper_available() {
    let candles: Vec<_> = (0..30).map(|i| candle(100.0 + i as f64 * 0.3)).collect();
    let indicator = calculate_bollinger_bands_default(&candles).expect("Bollinger result");
    assert!(indicator.upper > indicator.lower);
}
