use std::collections::HashMap;
use crate::indicators::error::IndicatorError;
use crate::indicators::validation::*;
use crate::models::indicators::*;

pub fn parse_f64(value: &str) -> Result<f64, IndicatorError> {
    value
        .parse::<f64>()
        .map_err(|_| IndicatorError::InvalidNumericFormat(value.to_string()))
}

pub fn parse_u32(value: &str) -> Result<u32, IndicatorError> {
    value
        .parse::<u32>()
        .map_err(|_| IndicatorError::InvalidNumericFormat(value.to_string()))
}

pub fn parse_f64_from_any(value: &dyn std::fmt::Display) -> Result<f64, IndicatorError> {
    let s = value.to_string();
    parse_f64(&s)
}

pub fn parse_u32_from_any(value: &dyn std::fmt::Display) -> Result<u32, IndicatorError> {
    let s = value.to_string();
    parse_u32(&s)
}

pub fn parse_macd(
    macd_val: f64,
    signal_val: f64,
    histogram_val: Option<f64>,
    period: Option<(u32, u32, u32)>,
) -> Result<MacdIndicator, IndicatorError> {
    let histogram = histogram_val.unwrap_or_else(|| macd_val - signal_val);
    let macd = MacdIndicator {
        macd: macd_val,
        signal: signal_val,
        histogram,
        period,
    };
    validate_macd(&macd)?;
    Ok(macd)
}

pub fn parse_macd_from_map(
    data: &HashMap<String, String>,
) -> Result<MacdIndicator, IndicatorError> {
    let macd_val = data
        .get("macd")
        .ok_or_else(|| IndicatorError::MissingField("macd".to_string()))?
        .parse::<f64>()
        .map_err(|_| IndicatorError::InvalidNumericFormat("macd".to_string()))?;

    let signal_val = data
        .get("signal")
        .ok_or_else(|| IndicatorError::MissingField("signal".to_string()))?
        .parse::<f64>()
        .map_err(|_| IndicatorError::InvalidNumericFormat("signal".to_string()))?;

    let histogram_val = data.get("histogram").map(|s| {
        s.parse::<f64>()
            .map_err(|_| IndicatorError::InvalidNumericFormat("histogram".to_string()))
    });

    let histogram = match histogram_val {
        Some(Ok(h)) => Some(h),
        Some(Err(e)) => return Err(e),
        None => None,
    };

    let period = if let (Some(fast), Some(slow), Some(signal)) = (
        data.get("macd_fast_period"),
        data.get("macd_slow_period"),
        data.get("macd_signal_period"),
    ) {
        Some((
            parse_u32(fast)?,
            parse_u32(slow)?,
            parse_u32(signal)?,
        ))
    } else {
        None
    };

    parse_macd(macd_val, signal_val, histogram, period)
}

pub fn parse_rsi(value: f64, period: Option<u32>) -> Result<RsiIndicator, IndicatorError> {
    let rsi = RsiIndicator { value, period };
    validate_rsi_indicator(&rsi)?;
    Ok(rsi)
}

pub fn parse_rsi_from_map(data: &HashMap<String, String>) -> Result<RsiIndicator, IndicatorError> {
    let value = data
        .get("rsi")
        .ok_or_else(|| IndicatorError::MissingField("rsi".to_string()))?
        .parse::<f64>()
        .map_err(|_| IndicatorError::InvalidNumericFormat("rsi".to_string()))?;

    let period = data
        .get("rsi_period")
        .map(|s| parse_u32(s))
        .transpose()?;

    parse_rsi(value, period)
}

pub fn parse_ema(value: f64, period: u32) -> Result<EmaIndicator, IndicatorError> {
    let ema = EmaIndicator { value, period };
    validate_ema(&ema)?;
    Ok(ema)
}

pub fn parse_sma(value: f64, period: u32) -> Result<SmaIndicator, IndicatorError> {
    let sma = SmaIndicator { value, period };
    validate_sma(&sma)?;
    Ok(sma)
}

pub fn parse_volume(
    volume: f64,
    volume_ma: Option<f64>,
    volume_ma_period: Option<u32>,
) -> Result<VolumeIndicator, IndicatorError> {
    let vol = VolumeIndicator {
        volume,
        volume_ma,
        volume_ma_period,
    };
    validate_volume_indicator(&vol)?;
    Ok(vol)
}

pub fn parse_indicator_set_from_map(
    data: &HashMap<String, String>,
) -> Result<IndicatorSet, IndicatorError> {
    let symbol = data
        .get("symbol")
        .ok_or_else(|| IndicatorError::MissingField("symbol".to_string()))?
        .clone();

    let price = data
        .get("price")
        .ok_or_else(|| IndicatorError::MissingField("price".to_string()))?
        .parse::<f64>()
        .map_err(|_| IndicatorError::InvalidNumericFormat("price".to_string()))?;

    validate_price(price)?;

    let mut set = IndicatorSet::new(symbol, price);

    if let Some(funding_rate_str) = data.get("funding_rate") {
        let funding_rate = parse_f64(funding_rate_str)?;
        validate_funding_rate(funding_rate)?;
        set = set.with_funding_rate(funding_rate);
    }

    if let Ok(macd) = parse_macd_from_map(data) {
        set = set.with_macd(macd);
    }

    if let Ok(rsi) = parse_rsi_from_map(data) {
        set = set.with_rsi(rsi);
    }

    if let Some(timeframe) = data.get("timeframe") {
        set = set.with_timeframe(timeframe.clone());
    }

    validate_indicator_set(&set)?;
    Ok(set)
}


