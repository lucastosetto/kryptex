//! MACD (Moving Average Convergence Divergence) indicator implementations.

use super::super::trend::ema::EMA;
use crate::models::indicators::{Candle, MacdIndicator};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MACDSignal {
    BullishCross,
    BearishCross,
    BullishMomentum,
    BearishMomentum,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct MACD {
    ema_fast: EMA,
    ema_slow: EMA,
    signal_line: EMA,
    prev_macd: Option<f64>,
    prev_signal: Option<f64>,
}

impl MACD {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            ema_fast: EMA::new(fast_period),
            ema_slow: EMA::new(slow_period),
            signal_line: EMA::new(signal_period),
            prev_macd: None,
            prev_signal: None,
        }
    }

    pub fn update(&mut self, close: f64) -> (f64, f64, f64, MACDSignal) {
        let fast = self.ema_fast.update(close);
        let slow = self.ema_slow.update(close);
        let macd = fast - slow;

        let signal = self.signal_line.update(macd);
        let histogram = macd - signal;

        let macd_signal =
            if let (Some(prev_macd), Some(prev_signal)) = (self.prev_macd, self.prev_signal) {
                if prev_macd <= prev_signal && macd > signal {
                    MACDSignal::BullishCross
                } else if prev_macd >= prev_signal && macd < signal {
                    MACDSignal::BearishCross
                } else if macd > 0.0 && macd > prev_macd {
                    MACDSignal::BullishMomentum
                } else if macd < 0.0 && macd < prev_macd {
                    MACDSignal::BearishMomentum
                } else {
                    MACDSignal::Neutral
                }
            } else {
                MACDSignal::Neutral
            };

        self.prev_macd = Some(macd);
        self.prev_signal = Some(signal);
        (macd, signal, histogram, macd_signal)
    }
}

pub fn calculate_macd(
    candles: &[Candle],
    fast_period: u32,
    slow_period: u32,
    signal_period: u32,
) -> Option<MacdIndicator> {
    if candles.is_empty() {
        return None;
    }

    let mut macd = MACD::new(
        fast_period as usize,
        slow_period as usize,
        signal_period as usize,
    );
    let mut latest = None;

    for candle in candles {
        latest = Some(macd.update(candle.close));
    }

    latest.map(|(macd_val, signal, histogram, _)| MacdIndicator {
        macd: macd_val,
        signal,
        histogram,
        period: Some((fast_period, slow_period, signal_period)),
    })
}

pub fn calculate_macd_default(candles: &[Candle]) -> Option<MacdIndicator> {
    calculate_macd(candles, 12, 26, 9)
}
