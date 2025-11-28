//! Unit tests for the OBV indicator.

use perptrix::indicators::volume::obv::{OBVSignal, OBV};

#[test]
fn obv_detects_confirmation_and_divergence() {
    let mut obv = OBV::new();
    obv.update(100.0, 1000.0);

    let mut confirmation_seen = false;
    for (price, volume) in [(101.0, 1200.0), (102.0, 1300.0)] {
        let (_, signal) = obv.update(price, volume);
        if matches!(signal, OBVSignal::Confirmation) {
            confirmation_seen = true;
        }
    }

    let (_, signal) = obv.update(101.5, 200.0);
    assert!(confirmation_seen);
    assert_eq!(signal, OBVSignal::BullishDivergence);
}
