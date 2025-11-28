use perptrix::indicators::validation::*;
use perptrix::models::indicators::*;

#[test]
fn test_validate_rsi_valid() {
    assert!(validate_rsi(0.0).is_ok());
    assert!(validate_rsi(50.0).is_ok());
    assert!(validate_rsi(100.0).is_ok());
    assert!(validate_rsi(70.5).is_ok());
}

#[test]
fn test_validate_rsi_invalid() {
    assert!(validate_rsi(-1.0).is_err());
    assert!(validate_rsi(101.0).is_err());
    assert!(validate_rsi(f64::INFINITY).is_err());
    assert!(validate_rsi(f64::NAN).is_err());
    assert!(validate_rsi(f64::NEG_INFINITY).is_err());
}

#[test]
fn test_validate_period_valid() {
    assert!(validate_period(1).is_ok());
    assert!(validate_period(14).is_ok());
    assert!(validate_period(1000).is_ok());
}

#[test]
fn test_validate_period_invalid() {
    assert!(validate_period(0).is_err());
    assert!(validate_period(1001).is_err());
}

#[test]
fn test_validate_price_valid() {
    assert!(validate_price(0.01).is_ok());
    assert!(validate_price(45000.0).is_ok());
    assert!(validate_price(1e10).is_ok());
}

#[test]
fn test_validate_price_invalid() {
    assert!(validate_price(0.0).is_err());
    assert!(validate_price(-100.0).is_err());
    assert!(validate_price(f64::NAN).is_err());
    assert!(validate_price(f64::INFINITY).is_err());
}

#[test]
fn test_validate_volume_valid() {
    assert!(validate_volume(0.0).is_ok());
    assert!(validate_volume(1000.0).is_ok());
    assert!(validate_volume(1e10).is_ok());
}

#[test]
fn test_validate_volume_invalid() {
    assert!(validate_volume(-100.0).is_err());
    assert!(validate_volume(f64::NAN).is_err());
    assert!(validate_volume(f64::INFINITY).is_err());
}

#[test]
fn test_validate_funding_rate_valid() {
    assert!(validate_funding_rate(-1.0).is_ok());
    assert!(validate_funding_rate(0.0).is_ok());
    assert!(validate_funding_rate(1.0).is_ok());
    assert!(validate_funding_rate(-0.0002).is_ok());
    assert!(validate_funding_rate(0.0005).is_ok());
}

#[test]
fn test_validate_funding_rate_invalid() {
    assert!(validate_funding_rate(-1.1).is_err());
    assert!(validate_funding_rate(1.1).is_err());
    assert!(validate_funding_rate(f64::NAN).is_err());
    assert!(validate_funding_rate(f64::INFINITY).is_err());
}

#[test]
fn test_validate_macd_valid() {
    let macd = MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.2,
        period: None,
    };
    assert!(validate_macd(&macd).is_ok());
}

#[test]
fn test_validate_macd_invalid_histogram() {
    let macd = MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.5,
        period: None,
    };
    assert!(validate_macd(&macd).is_err());
}

#[test]
fn test_validate_macd_invalid_period_order() {
    let macd = MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.2,
        period: Some((26, 12, 9)),
    };
    assert!(validate_macd(&macd).is_err());
}

#[test]
fn test_validate_macd_invalid_period_zero() {
    let macd = MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.2,
        period: Some((0, 26, 9)),
    };
    assert!(validate_macd(&macd).is_err());
}

#[test]
fn test_validate_macd_nan() {
    let macd = MacdIndicator {
        macd: f64::NAN,
        signal: 0.3,
        histogram: 0.2,
        period: None,
    };
    assert!(validate_macd(&macd).is_err());
}

#[test]
fn test_validate_rsi_indicator_valid() {
    let rsi = RsiIndicator {
        value: 50.0,
        period: Some(14),
    };
    assert!(validate_rsi_indicator(&rsi).is_ok());
}

#[test]
fn test_validate_rsi_indicator_invalid_value() {
    let rsi = RsiIndicator {
        value: 150.0,
        period: Some(14),
    };
    assert!(validate_rsi_indicator(&rsi).is_err());
}

#[test]
fn test_validate_rsi_indicator_invalid_period() {
    let rsi = RsiIndicator {
        value: 50.0,
        period: Some(0),
    };
    assert!(validate_rsi_indicator(&rsi).is_err());
}

#[test]
fn test_validate_ema_valid() {
    let ema = EmaIndicator {
        value: 100.0,
        period: 20,
    };
    assert!(validate_ema(&ema).is_ok());
}

#[test]
fn test_validate_ema_invalid() {
    let ema = EmaIndicator {
        value: f64::NAN,
        period: 20,
    };
    assert!(validate_ema(&ema).is_err());

    let ema = EmaIndicator {
        value: 100.0,
        period: 0,
    };
    assert!(validate_ema(&ema).is_err());
}

#[test]
fn test_validate_sma_valid() {
    let sma = SmaIndicator {
        value: 100.0,
        period: 20,
    };
    assert!(validate_sma(&sma).is_ok());
}

#[test]
fn test_validate_sma_invalid() {
    let sma = SmaIndicator {
        value: f64::NAN,
        period: 20,
    };
    assert!(validate_sma(&sma).is_err());
}

#[test]
fn test_validate_volume_indicator_valid() {
    let volume = VolumeIndicator {
        volume: 1000.0,
        volume_ma: Some(950.0),
        volume_ma_period: Some(20),
    };
    assert!(validate_volume_indicator(&volume).is_ok());
}

#[test]
fn test_validate_volume_indicator_invalid() {
    let volume = VolumeIndicator {
        volume: -100.0,
        volume_ma: None,
        volume_ma_period: None,
    };
    assert!(validate_volume_indicator(&volume).is_err());
}

#[test]
fn test_validate_indicator_set_valid() {
    let mut set = IndicatorSet::new("BTC".to_string(), 45000.0);
    set = set.with_macd(MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.2,
        period: None,
    });
    set = set.with_rsi(RsiIndicator {
        value: 50.0,
        period: None,
    });

    assert!(validate_indicator_set(&set).is_ok());
}

#[test]
fn test_validate_indicator_set_empty_symbol() {
    let set = IndicatorSet::new("".to_string(), 45000.0);
    assert!(validate_indicator_set(&set).is_err());
}

#[test]
fn test_validate_indicator_set_invalid_price() {
    let set = IndicatorSet::new("BTC".to_string(), -100.0);
    assert!(validate_indicator_set(&set).is_err());
}

#[test]
fn test_validate_indicator_set_invalid_macd() {
    let mut set = IndicatorSet::new("BTC".to_string(), 45000.0);
    set = set.with_macd(MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.5,
        period: None,
    });
    assert!(validate_indicator_set(&set).is_err());
}

#[test]
fn test_validate_indicator_set_invalid_rsi() {
    let mut set = IndicatorSet::new("BTC".to_string(), 45000.0);
    set = set.with_rsi(RsiIndicator {
        value: 150.0,
        period: None,
    });
    assert!(validate_indicator_set(&set).is_err());
}

#[test]
fn test_validate_indicator_set_invalid_funding_rate() {
    let mut set = IndicatorSet::new("BTC".to_string(), 45000.0);
    set = set.with_funding_rate(2.0);
    assert!(validate_indicator_set(&set).is_err());
}
