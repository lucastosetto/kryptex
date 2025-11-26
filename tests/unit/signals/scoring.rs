//! Unit tests for signal scoring

use kryptex::signals::scoring::*;

#[test]
fn test_normalize_score() {
    assert_eq!(normalize_score(50.0, 0.0, 100.0), 0.0);
    assert_eq!(normalize_score(0.0, 0.0, 100.0), -1.0);
    assert_eq!(normalize_score(100.0, 0.0, 100.0), 1.0);
}

#[test]
fn test_normalize_rsi() {
    assert_eq!(normalize_rsi(50.0), 0.0);
    assert_eq!(normalize_rsi(0.0), -1.0);
    assert_eq!(normalize_rsi(100.0), 1.0);
}

#[test]
fn test_normalize_ema_cross() {
    assert_eq!(normalize_ema_cross(1), 1.0);
    assert_eq!(normalize_ema_cross(-1), -1.0);
    assert_eq!(normalize_ema_cross(0), 0.0);
}

#[test]
fn test_normalize_supertrend() {
    assert_eq!(normalize_supertrend(1), 1.0);
    assert_eq!(normalize_supertrend(-1), -1.0);
}

#[test]
fn test_calculate_confidence() {
    assert_eq!(calculate_confidence(0.5), 0.5);
    assert_eq!(calculate_confidence(-0.5), 0.5);
    assert_eq!(calculate_confidence(0.0), 0.0);
}

