//! EMA (Exponential Moving Average) indicator utilities and stateful trackers.

use crate::common::math;
use crate::models::indicators::{Candle, EmaIndicator};

/// Stateful EMA calculator that can be updated tick-by-tick.
#[derive(Debug, Clone)]
pub struct EMA {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
}

impl EMA {
    /// Create a new EMA calculator.
    pub fn new(period: usize) -> Self {
        let multiplier = 2.0 / (period as f64 + 1.0);
        Self {
            period,
            multiplier,
            current_ema: None,
        }
    }

    /// Update the EMA with the latest price and return the computed value.
    pub fn update(&mut self, price: f64) -> f64 {
        match self.current_ema {
            None => {
                self.current_ema = Some(price);
                price
            }
            Some(prev) => {
                let ema = (price * self.multiplier) + (prev * (1.0 - self.multiplier));
                self.current_ema = Some(ema);
                ema
            }
        }
    }

    /// Get the last computed EMA value.
    pub fn get(&self) -> Option<f64> {
        self.current_ema
    }

    /// Access the configured period.
    pub fn period(&self) -> usize {
        self.period
    }
}

/// Signals derived from EMA crossovers and structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EMATrendSignal {
    BullishCross,
    BearishCross,
    StrongUptrend,
    StrongDowntrend,
    Neutral,
}

/// Tracks two EMAs (fast/slow) and emits crossover signals.
#[derive(Debug, Clone)]
pub struct EMACrossover {
    ema_fast: EMA,
    ema_slow: EMA,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
}

impl EMACrossover {
    /// Create a new EMA crossover tracker (e.g., 20/50).
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            ema_fast: EMA::new(fast_period),
            ema_slow: EMA::new(slow_period),
            prev_fast: None,
            prev_slow: None,
        }
    }

    /// Update both EMAs with the latest price and classify the trend state.
    pub fn update(&mut self, price: f64) -> EMATrendSignal {
        let fast = self.ema_fast.update(price);
        let slow = self.ema_slow.update(price);

        let signal = if let (Some(prev_fast), Some(prev_slow)) = (self.prev_fast, self.prev_slow) {
            if prev_fast <= prev_slow && fast > slow {
                EMATrendSignal::BullishCross
            } else if prev_fast >= prev_slow && fast < slow {
                EMATrendSignal::BearishCross
            } else if price > fast && fast > slow && (fast - prev_fast) > 0.0 {
                EMATrendSignal::StrongUptrend
            } else if price < fast && fast < slow && (fast - prev_fast) < 0.0 {
                EMATrendSignal::StrongDowntrend
            } else {
                EMATrendSignal::Neutral
            }
        } else {
            EMATrendSignal::Neutral
        };

        self.prev_fast = Some(fast);
        self.prev_slow = Some(slow);
        signal
    }

    /// Latest fast EMA value.
    pub fn fast(&self) -> Option<f64> {
        self.prev_fast
    }

    /// Latest slow EMA value.
    pub fn slow(&self) -> Option<f64> {
        self.prev_slow
    }
}

pub fn calculate_ema(candles: &[Candle], period: u32) -> Option<EmaIndicator> {
    if candles.len() < period as usize {
        return None;
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let ema_value = math::ema(&closes, period as usize)?;

    Some(EmaIndicator {
        value: ema_value,
        period,
    })
}

/// Calculate multiple EMAs at once.
pub fn calculate_emas(candles: &[Candle], periods: &[u32]) -> Vec<EmaIndicator> {
    periods
        .iter()
        .filter_map(|&period| calculate_ema(candles, period))
        .collect()
}

/// Check for EMA cross (e.g., EMA 12 crossing above/below EMA 26).
pub fn check_ema_cross(candles: &[Candle], fast_period: u32, slow_period: u32) -> Option<i32> {
    let fast_ema = calculate_ema(candles, fast_period)?;
    let slow_ema = calculate_ema(candles, slow_period)?;

    if fast_ema.value > slow_ema.value {
        Some(1) // Bullish cross
    } else if fast_ema.value < slow_ema.value {
        Some(-1) // Bearish cross
    } else {
        Some(0) // No cross
    }
}
