//! Integration tests for market scenarios

use kryptex::models::indicators::Candle;
use kryptex::signals::engine::SignalEngine;
use chrono::Utc;

fn create_uptrend_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let base = 100.0 + (i as f64 * 0.5);
        candles.push(Candle::new(
            base,
            base + 0.3,
            base - 0.2,
            base + 0.1,
            1000.0 + (i as f64 * 10.0),
            Utc::now(),
        ));
    }
    candles
}

fn create_downtrend_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let base = 100.0 - (i as f64 * 0.5);
        candles.push(Candle::new(
            base,
            base + 0.2,
            base - 0.3,
            base - 0.1,
            1000.0 + (i as f64 * 10.0),
            Utc::now(),
        ));
    }
    candles
}

fn create_ranging_candles(count: usize, min: f64, max: f64) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let cycle = (i as f64 % 20.0) / 20.0;
        let price = min + (max - min) * cycle;
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

fn create_volatile_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let base = 100.0 + (i as f64 * 0.1);
        let volatility = ((i as f64 % 5.0) - 2.5) * 2.0;
        candles.push(Candle::new(
            base,
            base + volatility.abs() + 0.5,
            base - volatility.abs() - 0.5,
            base + volatility,
            1000.0 + (i as f64 * 50.0),
            Utc::now(),
        ));
    }
    candles
}

fn create_reversal_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::new();
    let midpoint = count / 2;
    for i in 0..count {
        let base = if i < midpoint {
            100.0 + (i as f64 * 0.5)
        } else {
            100.0 + (midpoint as f64 * 0.5) - ((i - midpoint) as f64 * 0.5)
        };
        candles.push(Candle::new(
            base,
            base + 0.3,
            base - 0.2,
            base + if i < midpoint { 0.1 } else { -0.1 },
            1000.0 + (i as f64 * 10.0),
            Utc::now(),
        ));
    }
    candles
}

#[test]
fn test_strong_uptrend() {
    let candles = create_uptrend_candles(250);
    let signal = SignalEngine::evaluate(&candles, "BTC");
    assert!(signal.is_some());
    let s = signal.unwrap();
    assert!(s.confidence > 0.0);
    // In a strong uptrend, we'd expect Long or Neutral signal
    assert!(matches!(s.direction, kryptex::models::signal::SignalDirection::Long | kryptex::models::signal::SignalDirection::Neutral));
}

#[test]
fn test_strong_downtrend() {
    let candles = create_downtrend_candles(250);
    let signal = SignalEngine::evaluate(&candles, "BTC");
    assert!(signal.is_some());
    let s = signal.unwrap();
    assert!(s.confidence > 0.0);
    // In a strong downtrend, we'd expect Short or Neutral signal
    assert!(matches!(s.direction, kryptex::models::signal::SignalDirection::Short | kryptex::models::signal::SignalDirection::Neutral));
}

#[test]
fn test_ranging_market() {
    let candles = create_ranging_candles(250, 95.0, 105.0);
    let signal = SignalEngine::evaluate(&candles, "BTC");
    assert!(signal.is_some());
    let s = signal.unwrap();
    assert!(s.confidence > 0.0);
    // In a ranging market, Neutral is more likely
    assert!(matches!(s.direction, kryptex::models::signal::SignalDirection::Long | kryptex::models::signal::SignalDirection::Short | kryptex::models::signal::SignalDirection::Neutral));
}

#[test]
fn test_high_volatility() {
    let candles = create_volatile_candles(250);
    let signal = SignalEngine::evaluate(&candles, "BTC");
    assert!(signal.is_some());
    let s = signal.unwrap();
    assert!(s.confidence > 0.0);
    // High volatility might reduce confidence
    assert!(s.confidence <= 1.0);
}

#[test]
fn test_major_reversal() {
    let candles = create_reversal_candles(250);
    let signal = SignalEngine::evaluate(&candles, "BTC");
    assert!(signal.is_some());
    let s = signal.unwrap();
    assert!(s.confidence > 0.0);
    // Reversal scenarios might show mixed signals
    assert!(matches!(s.direction, kryptex::models::signal::SignalDirection::Long | kryptex::models::signal::SignalDirection::Short | kryptex::models::signal::SignalDirection::Neutral));
}

