//! Unit tests for EMA trackers and crossover signals.

use chrono::Utc;
use perptrix::indicators::trend::calculate_ema;
use perptrix::indicators::trend::ema::{EMACrossover, EMATrendSignal, EMA};
use perptrix::models::indicators::Candle;

fn build_candle(price: f64) -> Candle {
    Candle::new(price, price + 0.5, price - 0.5, price, 1000.0, Utc::now())
}

#[test]
fn ema_updates_smoothly() {
    let mut ema = EMA::new(5);
    let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0];
    let mut last = 0.0;
    for price in prices {
        last = ema.update(price);
    }
    assert!(last > 100.0);
    assert!(ema.get().is_some());
}

#[test]
fn ema_crossover_detects_signals() {
    let mut crossover = EMACrossover::new(3, 6);
    let mut bullish_seen = false;
    let mut bearish_seen = false;

    for price in [100.0, 101.0, 102.0, 103.0, 104.0, 105.0] {
        if crossover.update(price) == EMATrendSignal::BullishCross {
            bullish_seen = true;
        }
    }

    for price in [105.0, 104.0, 103.0, 102.0, 101.0, 100.0] {
        if crossover.update(price) == EMATrendSignal::BearishCross {
            bearish_seen = true;
        }
    }

    assert!(
        bullish_seen,
        "Expected a bullish crossover in the uptrend phase"
    );
    assert!(
        bearish_seen,
        "Expected a bearish crossover in the downtrend phase"
    );
}

#[test]
fn legacy_calculate_ema_still_works() {
    let candles: Vec<_> = (0..30).map(|i| build_candle(100.0 + i as f64)).collect();
    let ema_indicator = calculate_ema(&candles, 12).expect("EMA should be returned");
    assert_eq!(ema_indicator.period, 12);
    assert!(ema_indicator.value.is_finite());
}
