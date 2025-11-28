//! Unit tests for the open interest indicator.

use perptrix::indicators::perp::open_interest::{OpenInterest, OpenInterestSignal};

#[test]
fn open_interest_detects_expansions_and_squeezes() {
    let mut oi = OpenInterest::new();
    assert_eq!(oi.update(1000.0, 100.0), OpenInterestSignal::Neutral);
    assert_eq!(
        oi.update(1105.0, 101.0),
        OpenInterestSignal::BullishExpansion
    );
    assert_eq!(
        oi.update(1128.0, 99.0),
        OpenInterestSignal::BearishExpansion
    );
    assert_eq!(oi.update(1050.0, 98.0), OpenInterestSignal::LongSqueeze);
    assert_eq!(oi.update(1000.0, 100.0), OpenInterestSignal::ShortSqueeze);
}
