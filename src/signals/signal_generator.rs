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
        let mut long_score = 0.0;
        let mut short_score = 0.0;

        let macd_signal = self.analyze_macd(&input.macd);
        match macd_signal {
            MacdAnalysis::Bullish(weight) => {
                long_score += weight;
                reasons.push(SignalReason {
                    description: format!(
                        "MACD bullish: MACD={:.4}, Signal={:.4}, Histogram={:.4}",
                        input.macd.macd, input.macd.signal, input.macd.histogram
                    ),
                    weight,
                });
            }
            MacdAnalysis::Bearish(weight) => {
                short_score += weight;
                reasons.push(SignalReason {
                    description: format!(
                        "MACD bearish: MACD={:.4}, Signal={:.4}, Histogram={:.4}",
                        input.macd.macd, input.macd.signal, input.macd.histogram
                    ),
                    weight,
                });
            }
            MacdAnalysis::Neutral => {}
        }

        let rsi_signal = self.analyze_rsi(input.rsi);
        match rsi_signal {
            RsiAnalysis::Oversold(weight) => {
                long_score += weight;
                reasons.push(SignalReason {
                    description: format!("RSI oversold: {:.2}", input.rsi),
                    weight,
                });
            }
            RsiAnalysis::Overbought(weight) => {
                short_score += weight;
                reasons.push(SignalReason {
                    description: format!("RSI overbought: {:.2}", input.rsi),
                    weight,
                });
            }
            RsiAnalysis::Neutral => {}
        }

        let funding_signal = self.analyze_funding_rate(input.funding_rate);
        match funding_signal {
            FundingAnalysis::LongFavorable(weight) => {
                long_score += weight;
                reasons.push(SignalReason {
                    description: format!("Funding rate favorable for longs: {:.6}", input.funding_rate),
                    weight,
                });
            }
            FundingAnalysis::ShortFavorable(weight) => {
                short_score += weight;
                reasons.push(SignalReason {
                    description: format!("Funding rate favorable for shorts: {:.6}", input.funding_rate),
                    weight,
                });
            }
            FundingAnalysis::Neutral => {}
        }

        let (direction, confidence) = if long_score > short_score && long_score >= self.config.min_confidence {
            (SignalDirection::Long, long_score.min(1.0))
        } else if short_score > long_score && short_score >= self.config.min_confidence {
            (SignalDirection::Short, short_score.min(1.0))
        } else {
            (SignalDirection::None, 0.0)
        };

        let (sl_pct, tp_pct) = if direction != SignalDirection::None {
            let confidence_multiplier = 0.5 + (confidence * 0.5);
            let base_sl = self.config.default_sl_pct / confidence_multiplier;
            let base_tp = self.config.default_tp_pct * confidence_multiplier;
            (base_sl, base_tp)
        } else {
            (self.config.default_sl_pct, self.config.default_tp_pct)
        };

        SignalOutput::new(
            direction,
            confidence,
            sl_pct,
            tp_pct,
            reasons,
            symbol,
            input.price,
        )
    }

    fn analyze_macd(&self, macd: &MacdSignal) -> MacdAnalysis {
        if macd.macd > macd.signal && macd.histogram > 0.0 {
            let weight = (macd.histogram.abs() / (macd.macd.abs() + 0.001)).min(0.4);
            MacdAnalysis::Bullish(weight.max(0.2))
        } else if macd.macd < macd.signal && macd.histogram < 0.0 {
            let weight = (macd.histogram.abs() / (macd.macd.abs() + 0.001)).min(0.4);
            MacdAnalysis::Bearish(weight.max(0.2))
        } else {
            MacdAnalysis::Neutral
        }
    }

    fn analyze_rsi(&self, rsi: f64) -> RsiAnalysis {
        if rsi < self.config.rsi_oversold {
            let oversold_pct = (self.config.rsi_oversold - rsi) / self.config.rsi_oversold;
            let weight = (oversold_pct * 0.3).min(0.3);
            RsiAnalysis::Oversold(weight.max(0.15))
        } else if rsi > self.config.rsi_overbought {
            let overbought_pct = (rsi - self.config.rsi_overbought) / (100.0 - self.config.rsi_overbought);
            let weight = (overbought_pct * 0.3).min(0.3);
            RsiAnalysis::Overbought(weight.max(0.15))
        } else {
            RsiAnalysis::Neutral
        }
    }

    fn analyze_funding_rate(&self, funding_rate: f64) -> FundingAnalysis {
        let threshold = 0.0001;
        if funding_rate < -threshold {
            let weight = (funding_rate.abs() / 0.001).min(0.2);
            FundingAnalysis::LongFavorable(weight.max(0.1))
        } else if funding_rate > threshold {
            let weight = (funding_rate / 0.001).min(0.2);
            FundingAnalysis::ShortFavorable(weight.max(0.1))
        } else {
            FundingAnalysis::Neutral
        }
    }
}

enum MacdAnalysis {
    Bullish(f64),
    Bearish(f64),
    Neutral,
}

enum RsiAnalysis {
    Oversold(f64),
    Overbought(f64),
    Neutral,
}

enum FundingAnalysis {
    LongFavorable(f64),
    ShortFavorable(f64),
    Neutral,
}

