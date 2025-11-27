//! RSI (Relative Strength Index) indicator implementations.

use crate::models::indicators::{Candle, RsiIndicator};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RSISignal {
    Oversold,
    Overbought,
    BullishDivergence,
    BearishDivergence,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct RSI {
    period: usize,
    gains: Vec<f64>,
    losses: Vec<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    prev_close: Option<f64>,
    prev_rsi: Option<f64>,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            gains: Vec::new(),
            losses: Vec::new(),
            avg_gain: None,
            avg_loss: None,
            prev_close: None,
            prev_rsi: None,
        }
    }

    pub fn update(&mut self, close: f64) -> Option<f64> {
        if let Some(prev) = self.prev_close {
            let change = close - prev;
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            self.gains.push(gain);
            self.losses.push(loss);

            if self.gains.len() > self.period {
                self.gains.remove(0);
                self.losses.remove(0);
            }

            if self.gains.len() == self.period {
                let avg_gain = match self.avg_gain {
                    Some(prev_avg) => {
                        (prev_avg * (self.period - 1) as f64 + gain) / self.period as f64
                    }
                    None => self.gains.iter().sum::<f64>() / self.period as f64,
                };

                let avg_loss = match self.avg_loss {
                    Some(prev_avg) => {
                        (prev_avg * (self.period - 1) as f64 + loss) / self.period as f64
                    }
                    None => self.losses.iter().sum::<f64>() / self.period as f64,
                };

                self.avg_gain = Some(avg_gain);
                self.avg_loss = Some(avg_loss);

                let rs = if avg_loss == 0.0 {
                    100.0
                } else {
                    avg_gain / avg_loss
                };
                let rsi = 100.0 - (100.0 / (1.0 + rs));
                self.prev_rsi = Some(rsi);
                self.prev_close = Some(close);

                return Some(rsi);
            }
        }

        self.prev_close = Some(close);
        None
    }

    pub fn get_signal(&self, rsi: f64, price_change: f64) -> RSISignal {
        if rsi < 30.0 {
            if let Some(prev_rsi) = self.prev_rsi {
                if price_change < 0.0 && rsi > prev_rsi {
                    return RSISignal::BullishDivergence;
                }
            }
            RSISignal::Oversold
        } else if rsi > 70.0 {
            if let Some(prev_rsi) = self.prev_rsi {
                if price_change > 0.0 && rsi < prev_rsi {
                    return RSISignal::BearishDivergence;
                }
            }
            RSISignal::Overbought
        } else {
            RSISignal::Neutral
        }
    }

    pub fn last(&self) -> Option<f64> {
        self.prev_rsi
    }
}

pub fn calculate_rsi(candles: &[Candle], period: u32) -> Option<RsiIndicator> {
    if candles.len() < period as usize {
        return None;
    }

    let mut rsi = RSI::new(period as usize);
    for candle in candles {
        rsi.update(candle.close);
    }

    rsi.last().map(|value| RsiIndicator {
        value,
        period: Some(period),
    })
}

pub fn calculate_rsi_default(candles: &[Candle]) -> Option<RsiIndicator> {
    calculate_rsi(candles, 14)
}
