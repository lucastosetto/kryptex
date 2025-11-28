//! Main signal evaluation engine powered by the multi-dimensional indicator stack.

use std::collections::VecDeque;

use crate::engine::aggregator::{IndicatorSignals, SignalAggregator};
use crate::indicators::momentum::{macd, rsi};
use crate::indicators::perp::{funding_rate, open_interest};
use crate::indicators::trend::{ema, supertrend};
use crate::indicators::volatility::{atr, bollinger};
use crate::indicators::volume::{obv, volume_profile};
use crate::models::indicators::{Candle, IndicatorSet};
use crate::models::signal::{SignalDirection, SignalOutput, SignalReason};
use crate::signals::decision::StopLossTakeProfit;

const ATR_LOOKBACK: usize = 14;
const VOLUME_PROFILE_LOOKBACK: usize = 240;
const VOLUME_PROFILE_TICK: f64 = 10.0;
pub const MIN_CANDLES: usize = 50;

pub struct SignalEngine;

impl SignalEngine {
    /// Evaluate signal from candles augmented with perp metrics.
    pub fn evaluate(candles: &[Candle], symbol: &str) -> Option<SignalOutput> {
        if candles.len() < MIN_CANDLES {
            return None;
        }

        let current_price = candles.last()?.close;

        let mut ema_cross = ema::EMACrossover::new(20, 50);
        let mut supertrend = supertrend::SuperTrend::new(10, 3.0);
        let mut rsi = rsi::RSI::new(14);
        let mut macd = macd::MACD::new(12, 26, 9);
        let mut atr = atr::ATR::new(14);
        let mut bollinger = bollinger::BollingerBands::new(20, 2.0);
        let mut obv = obv::OBV::new();
        let mut volume_profile =
            volume_profile::VolumeProfile::new(VOLUME_PROFILE_TICK, VOLUME_PROFILE_LOOKBACK);
        let mut open_interest = open_interest::OpenInterest::new();
        let mut funding_rate = funding_rate::FundingRate::new(24);
        let mut atr_history: VecDeque<f64> = VecDeque::new();
        let mut prev_close: Option<f64> = None;

        let mut ema_signal = ema::EMATrendSignal::Neutral;
        let mut supertrend_signal = supertrend::SuperTrendSignal::Bearish;
        let mut rsi_signal = rsi::RSISignal::Neutral;
        let mut macd_signal = macd::MACDSignal::Neutral;
        let mut bollinger_signal = bollinger::BollingerSignal::Neutral;
        let mut volatility_regime = atr::VolatilityRegime::Normal;
        let mut obv_signal = obv::OBVSignal::Neutral;
        let mut volume_profile_signal = volume_profile::VolumeProfileSignal::Neutral;
        let mut oi_signal = open_interest::OpenInterestSignal::Neutral;
        let mut funding_signal = funding_rate::FundingSignal::Neutral;

        for candle in candles {
            ema_signal = ema_cross.update(candle.close);
            supertrend_signal = supertrend.update(candle.high, candle.low, candle.close);

            if let Some(rsi_value) = rsi.update(candle.close) {
                if let Some(prev) = prev_close {
                    let price_change = candle.close - prev;
                    rsi_signal = rsi.get_signal(rsi_value, price_change);
                }
            }

            let (_, _, _, macd_sig) = macd.update(candle.close);
            macd_signal = macd_sig;

            let (_, _, _, bb_sig) = bollinger.update(candle.close);
            bollinger_signal = bb_sig;

            let atr_value = atr.update(candle.high, candle.low, candle.close);
            atr_history.push_back(atr_value);
            if atr_history.len() > ATR_LOOKBACK {
                atr_history.pop_front();
            }
            let lookback_avg = if atr_history.is_empty() {
                atr_value
            } else {
                atr_history.iter().sum::<f64>() / atr_history.len() as f64
            };
            volatility_regime = atr.get_volatility_regime(atr_value, lookback_avg);

            let (_, obv_sig) = obv.update(candle.close, candle.volume);
            obv_signal = obv_sig;

            volume_profile.update(candle.close, candle.volume);
            let (_, _, vp_sig) = volume_profile.get_profile();
            volume_profile_signal = vp_sig;

            if let Some(oi) = candle.open_interest {
                oi_signal = open_interest.update(oi, candle.close);
            }

            if let Some(funding) = candle.funding_rate {
                let (funding_sig, _) = funding_rate.update(funding);
                funding_signal = funding_sig;
            }

            prev_close = Some(candle.close);
        }

        let indicator_signals = IndicatorSignals {
            ema_signal,
            supertrend_signal,
            rsi_signal,
            macd_signal,
            bollinger_signal,
            volatility_regime,
            obv_signal,
            volume_profile_signal,
            oi_signal,
            funding_signal,
        };

        let trading_signal = SignalAggregator::new().aggregate(indicator_signals);

        let direction = match trading_signal.position {
            crate::engine::signal::Position::Long => SignalDirection::Long,
            crate::engine::signal::Position::Short => SignalDirection::Short,
            crate::engine::signal::Position::Neutral => SignalDirection::Neutral,
        };

        let atr_value = atr.current().unwrap_or(0.0);
        let (sl_pct, tp_pct) = match direction {
            SignalDirection::Long | SignalDirection::Short if atr_value > 0.0 => {
                StopLossTakeProfit::calculate_from_atr(atr_value, current_price)
            }
            _ => (0.0, 0.0),
        };

        let mut reasons: Vec<SignalReason> = trading_signal
            .reasons
            .iter()
            .map(|reason| SignalReason {
                description: reason.clone(),
                weight: 1.0,
            })
            .collect();
        reasons.push(SignalReason {
            description: format!("Risk level: {:?}", trading_signal.risk_level),
            weight: 0.5,
        });

        // Debug: Log score even if neutral
        // The actual decision uses integer score thresholds: >= 3 for Long, <= -3 for Short
        // Normalize to 0-1 range for display (assuming max possible score is around ±12)
        let max_possible_score = 12.0;
        let normalized_score = (trading_signal.score_breakdown.total_score as f64 / max_possible_score + 1.0) / 2.0;
        let normalized_score = normalized_score.max(0.0).min(1.0);
        
        if direction == crate::models::signal::SignalDirection::Neutral {
            eprintln!("  [DEBUG] {}: Neutral signal - Integer Score: {} (needs >= 3 for Long, <= -3 for Short). Normalized: {:.2}. Breakdown: T={}, M={}, V={}, Vol={}, P={}, Total={}", 
                symbol,
                trading_signal.score_breakdown.total_score,
                normalized_score,
                trading_signal.score_breakdown.trend_score,
                trading_signal.score_breakdown.momentum_score,
                trading_signal.score_breakdown.volatility_score,
                trading_signal.score_breakdown.volume_score,
                trading_signal.score_breakdown.perp_score,
                trading_signal.score_breakdown.total_score
            );
        }

        Some(SignalOutput::new(
            direction,
            trading_signal.confidence,
            sl_pct,
            tp_pct,
            reasons,
            symbol.to_string(),
            current_price,
        ))
    }

    /// Evaluate signal and return full indicator set (for API responses/debugging)
    pub fn evaluate_with_indicators(
        candles: &[Candle],
        symbol: &str,
    ) -> Option<(SignalOutput, IndicatorSet)> {
        let signal = Self::evaluate(candles, symbol)?;
        let mut indicator_set = IndicatorSet::new(symbol.to_string(), signal.price);

        if let Some(funding_rate) = candles.last().and_then(|c| c.funding_rate) {
            indicator_set = indicator_set.with_funding_rate(funding_rate);
        }

        if let Some(open_interest) = candles.last().and_then(|c| c.open_interest) {
            indicator_set = indicator_set.with_open_interest(open_interest);
        }

        Some((signal, indicator_set))
    }
}
