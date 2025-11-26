use crate::config::Config;
use crate::signals::types::*;

pub struct SignalGenerator {
    config: Config,
}

impl SignalGenerator {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn generate_signal(&self, input: &IndicatorInput) -> SignalOutput {
        let symbol = input
            .symbol
            .clone()
            .unwrap_or_else(|| self.config.default_symbol.clone());

        let mut reasons = Vec::new();
        let mut macd_confidence = 0.0;
        let mut rsi_confidence = 0.0;
        let mut funding_confidence = 0.0;
        let mut hist_confidence = 0.0;

        let macd_bullish = input.macd.macd > input.macd.signal;
        let macd_bearish = input.macd.macd < input.macd.signal;

        if macd_bullish || macd_bearish {
            let crossover_size = (input.macd.macd - input.macd.signal).abs();
            macd_confidence = (crossover_size / self.config.macd_scale).min(1.0) * 0.4;

            let direction_str = if macd_bullish { "bullish" } else { "bearish" };
            reasons.push(SignalReason {
                description: format!(
                    "MACD {} crossover",
                    direction_str
                ),
                weight: macd_confidence,
            });
        }

        if macd_bullish {
            if input.rsi < 30.0 {
                rsi_confidence = ((30.0 - input.rsi) / 30.0).max(0.0) * 0.3;
                reasons.push(SignalReason {
                    description: format!(
                        "RSI oversold",
                    ),
                    weight: rsi_confidence,
                });
            } else if input.rsi < 50.0 {
                let partial = ((50.0 - input.rsi) / 50.0) * 0.15;
                rsi_confidence = partial;
                reasons.push(SignalReason {
                    description: format!(
                        "RSI near oversold",
                    ),
                    weight: rsi_confidence,
                });
            }
        } else if macd_bearish {
            if input.rsi > 70.0 {
                rsi_confidence = ((input.rsi - 70.0) / 30.0).max(0.0) * 0.3;
                reasons.push(SignalReason {
                    description: format!(
                        "RSI overbought",
                    ),
                    weight: rsi_confidence,
                });
            } else if input.rsi > 50.0 {
                let partial = ((input.rsi - 50.0) / 50.0) * 0.15;
                rsi_confidence = partial;
                reasons.push(SignalReason {
                    description: format!(
                        "RSI near overbought",
                    ),
                    weight: rsi_confidence,
                });
            }
        }

        if let Some(funding_rate) = input.funding_rate {
            funding_confidence = (funding_rate.abs() / 0.1).min(1.0) * 0.15;
            if funding_confidence > 0.0 {
                let direction_str = if funding_rate > 0.0 { "positive" } else { "negative" };
                reasons.push(SignalReason {
                    description: format!(
                        "Funding rate {}",
                        direction_str
                    ),
                    weight: funding_confidence,
                });
            }
        }

        if macd_bullish || macd_bearish {
            let histogram_abs = input.macd.histogram.abs();
            hist_confidence = (histogram_abs / self.config.hist_scale).min(1.0) * 0.15;
            
            if hist_confidence > 0.0 {
                reasons.push(SignalReason {
                    description: format!(
                        "Histogram supports MACD",
                    ),
                    weight: hist_confidence,
                });
            }
        }

        let total_confidence = macd_confidence + rsi_confidence + funding_confidence + hist_confidence;

        let direction = if total_confidence < 0.1 {
            SignalDirection::None
        } else if macd_bullish {
            SignalDirection::Long
        } else if macd_bearish {
            SignalDirection::Short
        } else {
            SignalDirection::None
        };

        let (sl_pct, tp_pct) = if direction != SignalDirection::None {
            let recommended_sl = self.config.default_sl_pct * (1.0 - total_confidence);
            let recommended_tp = self.config.default_tp_pct * total_confidence;
            (recommended_sl, recommended_tp)
        } else {
            (self.config.default_sl_pct, self.config.default_tp_pct)
        };

        SignalOutput::new(
            direction,
            total_confidence,
            sl_pct,
            tp_pct,
            reasons,
            symbol,
            input.price,
        )
    }
}
