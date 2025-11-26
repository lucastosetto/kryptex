//! Unit tests for indicator parser

use kryptex::indicators::parser::*;
use std::collections::HashMap;

#[test]
fn test_parse_f64_valid() {
    assert!(parse_f64("123.45").is_ok());
    assert!(parse_f64("-0.5").is_ok());
    assert!(parse_f64("0").is_ok());
}

#[test]
fn test_parse_f64_invalid() {
    assert!(parse_f64("abc").is_err());
    assert!(parse_f64("").is_err());
}

#[test]
fn test_parse_u32_valid() {
    assert!(parse_u32("42").is_ok());
    assert!(parse_u32("0").is_ok());
}

#[test]
fn test_parse_u32_invalid() {
    assert!(parse_u32("abc").is_err());
    assert!(parse_u32("-5").is_err());
    assert!(parse_u32("12.5").is_err());
}

#[test]
fn test_parse_rsi_valid() {
    assert!(parse_rsi(50.0, None).is_ok());
    assert!(parse_rsi(0.0, Some(14)).is_ok());
    assert!(parse_rsi(100.0, Some(14)).is_ok());
}

#[test]
fn test_parse_rsi_invalid() {
    assert!(parse_rsi(150.0, None).is_err());
    assert!(parse_rsi(-10.0, None).is_err());
}

#[test]
fn test_parse_macd_valid() {
    assert!(parse_macd(0.5, 0.3, Some(0.2), None).is_ok());
    assert!(parse_macd(0.5, 0.3, None, Some((12, 26, 9))).is_ok());
}

#[test]
fn test_parse_macd_invalid_histogram() {
    let result = parse_macd(0.5, 0.3, Some(0.5), None);
    assert!(result.is_err());
}

#[test]
fn test_parse_macd_invalid_period() {
    let result = parse_macd(0.5, 0.3, Some(0.2), Some((26, 12, 9)));
    assert!(result.is_err());
}

#[test]
fn test_parse_indicator_set_from_map_valid() {
    let mut data = HashMap::new();
    data.insert("symbol".to_string(), "BTC".to_string());
    data.insert("price".to_string(), "45000.0".to_string());
    data.insert("macd".to_string(), "0.5".to_string());
    data.insert("signal".to_string(), "0.3".to_string());
    data.insert("histogram".to_string(), "0.2".to_string());
    data.insert("rsi".to_string(), "50.0".to_string());

    assert!(parse_indicator_set_from_map(&data).is_ok());
}

#[test]
fn test_parse_indicator_set_from_map_missing_symbol() {
    let mut data = HashMap::new();
    data.insert("price".to_string(), "45000.0".to_string());

    assert!(parse_indicator_set_from_map(&data).is_err());
}

#[test]
fn test_parse_indicator_set_from_map_invalid_price() {
    let mut data = HashMap::new();
    data.insert("symbol".to_string(), "BTC".to_string());
    data.insert("price".to_string(), "-100.0".to_string());

    assert!(parse_indicator_set_from_map(&data).is_err());
}

#[test]
fn test_parse_rsi_from_map_valid() {
    let mut data = HashMap::new();
    data.insert("rsi".to_string(), "75.5".to_string());
    data.insert("rsi_period".to_string(), "14".to_string());

    assert!(parse_rsi_from_map(&data).is_ok());
}

#[test]
fn test_parse_rsi_from_map_missing() {
    let data = HashMap::new();
    assert!(parse_rsi_from_map(&data).is_err());
}

#[test]
fn test_parse_macd_from_map_valid() {
    let mut data = HashMap::new();
    data.insert("macd".to_string(), "0.5".to_string());
    data.insert("signal".to_string(), "0.3".to_string());
    data.insert("histogram".to_string(), "0.2".to_string());

    assert!(parse_macd_from_map(&data).is_ok());
}

#[test]
fn test_parse_macd_from_map_missing() {
    let mut data = HashMap::new();
    data.insert("macd".to_string(), "0.5".to_string());
    assert!(parse_macd_from_map(&data).is_err());
}

#[test]
fn test_parse_ema_valid() {
    assert!(parse_ema(100.0, 20).is_ok());
}

#[test]
fn test_parse_ema_invalid_period() {
    assert!(parse_ema(100.0, 0).is_err());
}

#[test]
fn test_parse_sma_valid() {
    assert!(parse_sma(100.0, 20).is_ok());
}

#[test]
fn test_parse_volume_valid() {
    assert!(parse_volume(1000.0, None, None).is_ok());
    assert!(parse_volume(1000.0, Some(950.0), Some(20)).is_ok());
}

#[test]
fn test_parse_volume_invalid() {
    assert!(parse_volume(-100.0, None, None).is_err());
}

