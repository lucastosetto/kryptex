use perptrix::indicators::parser::*;
use std::collections::HashMap;

#[test]
fn test_parse_f64_valid() {
    assert!(parse_f64("123.45").is_ok());
    assert!(parse_f64("-0.5").is_ok());
    assert!(parse_f64("0").is_ok());
    assert!(parse_f64("1e10").is_ok());
}

#[test]
fn test_parse_f64_invalid() {
    assert!(parse_f64("abc").is_err());
    assert!(parse_f64("").is_err());
    assert!(parse_f64("12.34.56").is_err());
}

#[test]
fn test_parse_u32_valid() {
    assert!(parse_u32("42").is_ok());
    assert!(parse_u32("0").is_ok());
    assert!(parse_u32("1000").is_ok());
}

#[test]
fn test_parse_u32_invalid() {
    assert!(parse_u32("abc").is_err());
    assert!(parse_u32("-5").is_err());
    assert!(parse_u32("12.5").is_err());
    assert!(parse_u32("").is_err());
}

#[test]
fn test_parse_rsi_valid() {
    assert!(parse_rsi(50.0, None).is_ok());
    assert!(parse_rsi(0.0, Some(14)).is_ok());
    assert!(parse_rsi(100.0, Some(14)).is_ok());
    assert!(parse_rsi(70.5, Some(21)).is_ok());
}

#[test]
fn test_parse_rsi_invalid() {
    assert!(parse_rsi(150.0, None).is_err());
    assert!(parse_rsi(-10.0, None).is_err());
    assert!(parse_rsi(f64::INFINITY, None).is_err());
    assert!(parse_rsi(f64::NAN, None).is_err());
}

#[test]
fn test_parse_macd_valid() {
    assert!(parse_macd(0.5, 0.3, Some(0.2), None).is_ok());
    assert!(parse_macd(0.5, 0.3, None, Some((12, 26, 9))).is_ok());
    assert!(parse_macd(-0.5, -0.3, Some(-0.2), None).is_ok());
}

#[test]
fn test_parse_macd_invalid_histogram() {
    let result = parse_macd(0.5, 0.3, Some(0.5), None);
    assert!(result.is_err());
    
    let result = parse_macd(0.5, 0.3, Some(0.1), None);
    assert!(result.is_err());
}

#[test]
fn test_parse_macd_invalid_period() {
    let result = parse_macd(0.5, 0.3, Some(0.2), Some((26, 12, 9)));
    assert!(result.is_err());
    
    let result = parse_macd(0.5, 0.3, Some(0.2), Some((0, 26, 9)));
    assert!(result.is_err());
}

#[test]
fn test_parse_macd_invalid_nan() {
    let result = parse_macd(f64::NAN, 0.3, Some(0.2), None);
    assert!(result.is_err());
    
    let result = parse_macd(0.5, f64::INFINITY, Some(0.2), None);
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

    let result = parse_indicator_set_from_map(&data);
    assert!(result.is_ok());
    let set = result.unwrap();
    assert_eq!(set.symbol, "BTC");
    assert_eq!(set.price, 45000.0);
    assert!(set.macd.is_some());
    assert!(set.rsi.is_some());
}

#[test]
fn test_parse_indicator_set_from_map_missing_symbol() {
    let mut data = HashMap::new();
    data.insert("price".to_string(), "45000.0".to_string());

    assert!(parse_indicator_set_from_map(&data).is_err());
}

#[test]
fn test_parse_indicator_set_from_map_missing_price() {
    let mut data = HashMap::new();
    data.insert("symbol".to_string(), "BTC".to_string());

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
fn test_parse_indicator_set_from_map_zero_price() {
    let mut data = HashMap::new();
    data.insert("symbol".to_string(), "BTC".to_string());
    data.insert("price".to_string(), "0.0".to_string());

    assert!(parse_indicator_set_from_map(&data).is_err());
}

#[test]
fn test_parse_indicator_set_from_map_with_funding_rate() {
    let mut data = HashMap::new();
    data.insert("symbol".to_string(), "BTC".to_string());
    data.insert("price".to_string(), "45000.0".to_string());
    data.insert("funding_rate".to_string(), "-0.0002".to_string());

    let result = parse_indicator_set_from_map(&data);
    assert!(result.is_ok());
    let set = result.unwrap();
    assert!(set.funding_rate.is_some());
    assert_eq!(set.funding_rate.unwrap(), -0.0002);
}

#[test]
fn test_parse_rsi_from_map_valid() {
    let mut data = HashMap::new();
    data.insert("rsi".to_string(), "75.5".to_string());
    data.insert("rsi_period".to_string(), "14".to_string());

    let result = parse_rsi_from_map(&data);
    assert!(result.is_ok());
    let rsi = result.unwrap();
    assert_eq!(rsi.value, 75.5);
    assert_eq!(rsi.period, Some(14));
}

#[test]
fn test_parse_rsi_from_map_missing() {
    let data = HashMap::new();
    assert!(parse_rsi_from_map(&data).is_err());
}

#[test]
fn test_parse_rsi_from_map_invalid_value() {
    let mut data = HashMap::new();
    data.insert("rsi".to_string(), "150.0".to_string());
    assert!(parse_rsi_from_map(&data).is_err());
}

#[test]
fn test_parse_macd_from_map_valid() {
    let mut data = HashMap::new();
    data.insert("macd".to_string(), "0.5".to_string());
    data.insert("signal".to_string(), "0.3".to_string());
    data.insert("histogram".to_string(), "0.2".to_string());

    let result = parse_macd_from_map(&data);
    assert!(result.is_ok());
    let macd = result.unwrap();
    assert_eq!(macd.macd, 0.5);
    assert_eq!(macd.signal, 0.3);
    assert_eq!(macd.histogram, 0.2);
}

#[test]
fn test_parse_macd_from_map_with_periods() {
    let mut data = HashMap::new();
    data.insert("macd".to_string(), "0.5".to_string());
    data.insert("signal".to_string(), "0.3".to_string());
    data.insert("macd_fast_period".to_string(), "12".to_string());
    data.insert("macd_slow_period".to_string(), "26".to_string());
    data.insert("macd_signal_period".to_string(), "9".to_string());

    let result = parse_macd_from_map(&data);
    assert!(result.is_ok());
    let macd = result.unwrap();
    assert_eq!(macd.period, Some((12, 26, 9)));
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
    assert!(parse_ema(0.0, 1).is_ok());
    assert!(parse_ema(-50.0, 50).is_ok());
}

#[test]
fn test_parse_ema_invalid_period() {
    assert!(parse_ema(100.0, 0).is_err());
    assert!(parse_ema(100.0, 1001).is_err());
}

#[test]
fn test_parse_sma_valid() {
    assert!(parse_sma(100.0, 20).is_ok());
    assert!(parse_sma(0.0, 1).is_ok());
}

#[test]
fn test_parse_sma_invalid_period() {
    assert!(parse_sma(100.0, 0).is_err());
    assert!(parse_sma(100.0, 1001).is_err());
}

#[test]
fn test_parse_volume_valid() {
    assert!(parse_volume(1000.0, None, None).is_ok());
    assert!(parse_volume(1000.0, Some(950.0), Some(20)).is_ok());
    assert!(parse_volume(0.0, None, None).is_ok());
}

#[test]
fn test_parse_volume_invalid() {
    assert!(parse_volume(-100.0, None, None).is_err());
    assert!(parse_volume(f64::NAN, None, None).is_err());
    assert!(parse_volume(f64::INFINITY, None, None).is_err());
}

#[test]
fn test_parse_volume_with_ma() {
    let result = parse_volume(1000.0, Some(-50.0), Some(20));
    assert!(result.is_err());
}

