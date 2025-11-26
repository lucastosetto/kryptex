//! Unit tests for signal decision logic

use kryptex::models::signal::SignalDirection;
use kryptex::signals::decision::{DirectionThresholds, StopLossTakeProfit};

#[test]
fn test_determine_direction_long() {
    let score = 0.65;
    assert_eq!(DirectionThresholds::determine_direction(score), SignalDirection::Long);
}

#[test]
fn test_determine_direction_short() {
    let score = 0.35;
    assert_eq!(DirectionThresholds::determine_direction(score), SignalDirection::Short);
}

#[test]
fn test_determine_direction_neutral() {
    let score = 0.50;
    assert_eq!(DirectionThresholds::determine_direction(score), SignalDirection::Neutral);
}

#[test]
fn test_to_percentage() {
    assert_eq!(DirectionThresholds::to_percentage(-1.0), 0.0);
    assert_eq!(DirectionThresholds::to_percentage(0.0), 0.5);
    assert_eq!(DirectionThresholds::to_percentage(1.0), 1.0);
}

#[test]
fn test_calculate_sl_tp() {
    let atr = 10.0;
    let price = 100.0;
    let (sl, tp) = StopLossTakeProfit::calculate_from_atr(atr, price);
    assert_eq!(sl, 12.0);
    assert_eq!(tp, 20.0);
}

