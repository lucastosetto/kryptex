use crate::indicators::error::IndicatorError;
use crate::models::indicators::*;

const RSI_MIN: f64 = 0.0;
const RSI_MAX: f64 = 100.0;
const MIN_PERIOD: u32 = 1;
const MAX_PERIOD: u32 = 1000;
const MIN_PRICE: f64 = 0.0;
const MIN_VOLUME: f64 = 0.0;
const FUNDING_RATE_MIN: f64 = -1.0;
const FUNDING_RATE_MAX: f64 = 1.0;
const MACD_HISTOGRAM_TOLERANCE: f64 = 0.0001;

pub fn validate_rsi(value: f64) -> Result<(), IndicatorError> {
    if value < RSI_MIN || value > RSI_MAX {
        return Err(IndicatorError::OutOfRange {
            field: "rsi".to_string(),
            value,
            min: RSI_MIN,
            max: RSI_MAX,
        });
    }
    if !value.is_finite() {
        return Err(IndicatorError::ValidationError(format!(
            "RSI value must be finite, got: {}",
            value
        )));
    }
    Ok(())
}

pub fn validate_period(period: u32) -> Result<(), IndicatorError> {
    if period < MIN_PERIOD || period > MAX_PERIOD {
        return Err(IndicatorError::InvalidPeriod {
            field: "period".to_string(),
            value: period,
        });
    }
    Ok(())
}

pub fn validate_price(price: f64) -> Result<(), IndicatorError> {
    if price <= MIN_PRICE {
        return Err(IndicatorError::OutOfRange {
            field: "price".to_string(),
            value: price,
            min: MIN_PRICE,
            max: f64::INFINITY,
        });
    }
    if !price.is_finite() {
        return Err(IndicatorError::ValidationError(format!(
            "Price must be finite, got: {}",
            price
        )));
    }
    Ok(())
}

pub fn validate_volume(volume: f64) -> Result<(), IndicatorError> {
    if volume < MIN_VOLUME {
        return Err(IndicatorError::OutOfRange {
            field: "volume".to_string(),
            value: volume,
            min: MIN_VOLUME,
            max: f64::INFINITY,
        });
    }
    if !volume.is_finite() {
        return Err(IndicatorError::ValidationError(format!(
            "Volume must be finite, got: {}",
            volume
        )));
    }
    Ok(())
}

pub fn validate_funding_rate(funding_rate: f64) -> Result<(), IndicatorError> {
    if funding_rate < FUNDING_RATE_MIN || funding_rate > FUNDING_RATE_MAX {
        return Err(IndicatorError::OutOfRange {
            field: "funding_rate".to_string(),
            value: funding_rate,
            min: FUNDING_RATE_MIN,
            max: FUNDING_RATE_MAX,
        });
    }
    if !funding_rate.is_finite() {
        return Err(IndicatorError::ValidationError(format!(
            "Funding rate must be finite, got: {}",
            funding_rate
        )));
    }
    Ok(())
}

pub fn validate_macd(macd: &MacdIndicator) -> Result<(), IndicatorError> {
    if !macd.macd.is_finite() {
        return Err(IndicatorError::ValidationError(
            "MACD value must be finite".to_string(),
        ));
    }
    if !macd.signal.is_finite() {
        return Err(IndicatorError::ValidationError(
            "MACD signal value must be finite".to_string(),
        ));
    }
    if !macd.histogram.is_finite() {
        return Err(IndicatorError::ValidationError(
            "MACD histogram value must be finite".to_string(),
        ));
    }

    let expected_histogram = macd.macd - macd.signal;
    let histogram_diff = (macd.histogram - expected_histogram).abs();
    if histogram_diff > MACD_HISTOGRAM_TOLERANCE {
        return Err(IndicatorError::ValidationError(format!(
            "MACD histogram inconsistency: expected {}, got {} (diff: {})",
            expected_histogram, macd.histogram, histogram_diff
        )));
    }

    if let Some((fast, slow, signal)) = macd.period {
        validate_period(fast)?;
        validate_period(slow)?;
        validate_period(signal)?;
        if fast >= slow {
            return Err(IndicatorError::ValidationError(format!(
                "MACD fast period ({}) must be less than slow period ({})",
                fast, slow
            )));
        }
    }

    Ok(())
}

pub fn validate_rsi_indicator(rsi: &RsiIndicator) -> Result<(), IndicatorError> {
    validate_rsi(rsi.value)?;
    if let Some(period) = rsi.period {
        validate_period(period)?;
    }
    Ok(())
}

pub fn validate_ema(ema: &EmaIndicator) -> Result<(), IndicatorError> {
    if !ema.value.is_finite() {
        return Err(IndicatorError::ValidationError(
            "EMA value must be finite".to_string(),
        ));
    }
    validate_period(ema.period)?;
    Ok(())
}

pub fn validate_sma(sma: &SmaIndicator) -> Result<(), IndicatorError> {
    if !sma.value.is_finite() {
        return Err(IndicatorError::ValidationError(
            "SMA value must be finite".to_string(),
        ));
    }
    validate_period(sma.period)?;
    Ok(())
}

pub fn validate_volume_indicator(volume: &VolumeIndicator) -> Result<(), IndicatorError> {
    validate_volume(volume.volume)?;
    if let Some(volume_ma) = volume.volume_ma {
        validate_volume(volume_ma)?;
    }
    if let Some(period) = volume.volume_ma_period {
        validate_period(period)?;
    }
    Ok(())
}

pub fn validate_indicator_set(set: &IndicatorSet) -> Result<(), IndicatorError> {
    if set.symbol.is_empty() {
        return Err(IndicatorError::ValidationError(
            "Symbol cannot be empty".to_string(),
        ));
    }
    validate_price(set.price)?;

    if let Some(ref funding_rate) = set.funding_rate {
        validate_funding_rate(*funding_rate)?;
    }

    if let Some(ref macd) = set.macd {
        validate_macd(macd)?;
    }

    if let Some(ref rsi) = set.rsi {
        validate_rsi_indicator(rsi)?;
    }

    for ema in &set.emas {
        validate_ema(ema)?;
    }

    for sma in &set.smas {
        validate_sma(sma)?;
    }

    if let Some(ref volume) = set.volume {
        validate_volume_indicator(volume)?;
    }

    Ok(())
}

