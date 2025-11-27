//! ATR (Average True Range) volatility regime detector.

use crate::models::indicators::{AtrIndicator, Candle};

#[derive(Debug, Clone)]
pub struct ATR {
    period: usize,
    true_ranges: Vec<f64>,
    current_atr: Option<f64>,
    prev_close: Option<f64>,
}

impl ATR {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            true_ranges: Vec::new(),
            current_atr: None,
            prev_close: None,
        }
    }

    pub fn update(&mut self, high: f64, low: f64, close: f64) -> f64 {
        let true_range = if let Some(prev_close) = self.prev_close {
            let tr1 = high - low;
            let tr2 = (high - prev_close).abs();
            let tr3 = (low - prev_close).abs();
            tr1.max(tr2).max(tr3)
        } else {
            high - low
        };

        self.true_ranges.push(true_range);
        if self.true_ranges.len() > self.period {
            self.true_ranges.remove(0);
        }

        let atr = match self.current_atr {
            Some(prev_atr) if self.true_ranges.len() == self.period => {
                (prev_atr * (self.period - 1) as f64 + true_range) / self.period as f64
            }
            _ if self.true_ranges.len() == self.period => {
                self.true_ranges.iter().sum::<f64>() / self.period as f64
            }
            _ => true_range,
        };

        self.current_atr = Some(atr);
        self.prev_close = Some(close);
        atr
    }

    pub fn current(&self) -> Option<f64> {
        self.current_atr
    }

    pub fn get_volatility_regime(&self, atr: f64, lookback_avg: f64) -> VolatilityRegime {
        if lookback_avg <= f64::EPSILON {
            return VolatilityRegime::Normal;
        }

        let ratio = atr / lookback_avg;
        if ratio > 1.5 {
            VolatilityRegime::High
        } else if ratio > 1.0 {
            VolatilityRegime::Elevated
        } else if ratio > 0.7 {
            VolatilityRegime::Normal
        } else {
            VolatilityRegime::Low
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VolatilityRegime {
    High,
    Elevated,
    Normal,
    Low,
}

pub fn calculate_atr(candles: &[Candle], period: u32) -> Option<AtrIndicator> {
    if candles.len() < period as usize {
        return None;
    }

    let mut atr = ATR::new(period as usize);
    for window in candles {
        atr.update(window.high, window.low, window.close);
    }

    atr.current().map(|value| AtrIndicator { value, period })
}

pub fn calculate_atr_default(candles: &[Candle]) -> Option<AtrIndicator> {
    calculate_atr(candles, 14)
}
