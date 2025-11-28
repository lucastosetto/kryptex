//! Unit tests for the SuperTrend indicator.

use perptrix::indicators::trend::supertrend::{SuperTrend, SuperTrendSignal};

#[test]
fn supertrend_flips_with_price_direction() {
    let mut bullish = SuperTrend::new(3, 2.0);
    let mut bullish_signal = SuperTrendSignal::Bullish;
    for (high, low, close) in [
        (102.0, 100.0, 101.5),
        (103.5, 101.5, 103.0),
        (105.0, 103.0, 104.8),
        (106.0, 104.0, 105.5),
    ] {
        bullish_signal = bullish.update(high, low, close);
    }
    assert!(
        matches!(
            bullish_signal,
            SuperTrendSignal::Bullish | SuperTrendSignal::BullishFlip
        ),
        "Expected bullish regime during up-move"
    );

    let mut bearish = SuperTrend::new(3, 2.0);
    let mut bearish_seen = false;
    for (high, low, close) in [
        (105.0, 99.0, 100.0),
        (103.0, 97.0, 98.0),
        (101.0, 95.0, 96.0),
        (99.0, 93.0, 94.0),
        (97.0, 91.0, 92.0),
        (95.0, 89.0, 90.0),
        (93.0, 87.0, 88.0),
    ] {
        let signal = bearish.update(high, low, close);
        if matches!(
            signal,
            SuperTrendSignal::Bearish | SuperTrendSignal::BearishFlip
        ) {
            bearish_seen = true;
        }
    }
    assert!(bearish_seen, "Expected bearish regime during down-move");
}
