//! Bollinger Bands volatility squeeze detector.

use crate::models::indicators::{BollingerBandsIndicator, Candle};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BollingerSignal {
    Squeeze,
    UpperBreakout,
    LowerBreakout,
    WalkingBands,
    MeanReversion,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct BollingerBands {
    period: usize,
    std_dev: f64,
    prices: Vec<f64>,
    prev_bandwidth: Option<f64>,
}

impl BollingerBands {
    pub fn new(period: usize, std_dev: f64) -> Self {
        Self {
            period,
            std_dev,
            prices: Vec::new(),
            prev_bandwidth: None,
        }
    }

    pub fn update(&mut self, close: f64) -> (f64, f64, f64, BollingerSignal) {
        self.prices.push(close);
        if self.prices.len() > self.period {
            self.prices.remove(0);
        }

        let middle = self.prices.iter().sum::<f64>() / self.prices.len() as f64;
        let variance = self
            .prices
            .iter()
            .map(|price| (price - middle).powi(2))
            .sum::<f64>()
            / self.prices.len() as f64;
        let std = variance.sqrt();
        let upper = middle + (self.std_dev * std);
        let lower = middle - (self.std_dev * std);
        let bandwidth = if middle.abs() > f64::EPSILON {
            (upper - lower) / middle
        } else {
            0.0
        };

        let signal = if self.prices.len() == self.period {
            if bandwidth < 0.05 {
                BollingerSignal::Squeeze
            } else if close > upper {
                BollingerSignal::UpperBreakout
            } else if close < lower {
                BollingerSignal::LowerBreakout
            } else if let Some(prev_bw) = self.prev_bandwidth {
                if bandwidth < prev_bw && (close - middle).abs() < std * 0.5 {
                    BollingerSignal::MeanReversion
                } else if close >= upper - (std * 0.2) || close <= lower + (std * 0.2) {
                    BollingerSignal::WalkingBands
                } else {
                    BollingerSignal::Neutral
                }
            } else {
                BollingerSignal::Neutral
            }
        } else {
            BollingerSignal::Neutral
        };

        self.prev_bandwidth = Some(bandwidth);
        (upper, middle, lower, signal)
    }
}

pub fn calculate_bollinger_bands(
    candles: &[Candle],
    period: u32,
    std_dev: f64,
) -> Option<BollingerBandsIndicator> {
    if candles.len() < period as usize {
        return None;
    }

    let mut bb = BollingerBands::new(period as usize, std_dev);
    let mut latest = None;
    for candle in candles {
        latest = Some(bb.update(candle.close));
    }

    latest.map(|(upper, middle, lower, _)| BollingerBandsIndicator {
        upper,
        middle,
        lower,
        period,
        std_dev,
    })
}

pub fn calculate_bollinger_bands_default(candles: &[Candle]) -> Option<BollingerBandsIndicator> {
    calculate_bollinger_bands(candles, 20, 2.0)
}
