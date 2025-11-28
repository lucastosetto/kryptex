//! Unit tests for the funding rate indicator.

use perptrix::indicators::perp::funding_rate::{FundingRate, FundingSignal};

#[test]
fn funding_rate_thresholds_work() {
    let mut funding = FundingRate::new(5);
    let (signal, _) = funding.update(0.0012);
    assert_eq!(signal, FundingSignal::ExtremeLongBias);

    let (signal, _) = funding.update(-0.0011);
    assert_eq!(signal, FundingSignal::ExtremShortBias);

    let (signal, _) = funding.update(0.0006);
    assert_eq!(signal, FundingSignal::HighLongBias);

    let (signal, avg) = funding.update(0.0);
    assert_eq!(signal, FundingSignal::Neutral);
    assert!(avg.abs() < 0.001);
}
