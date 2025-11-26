//! Main signal evaluation engine

use crate::indicators::registry::IndicatorCategory;
use crate::models::indicators::{Candle, IndicatorSet};
use crate::models::signal::{SignalDirection, SignalOutput};
use crate::signals::aggregation::{Aggregator, IndicatorScore};
use crate::signals::decision::{DirectionThresholds, StopLossTakeProfit};
use crate::signals::scoring::*;

use crate::indicators::momentum::{calculate_macd_default, calculate_rsi_default};
use crate::indicators::trend::{calculate_ema, calculate_adx_default};
use crate::indicators::volatility::{calculate_bollinger_bands_default, calculate_atr_default};
use crate::indicators::structure::{calculate_supertrend_default, calculate_support_resistance_default};

/// Main signal evaluation engine
pub struct SignalEngine;

impl SignalEngine {
    /// Evaluate signal from candles
    pub fn evaluate(candles: &[Candle], symbol: &str) -> Option<SignalOutput> {
        if candles.is_empty() {
            return None;
        }

        let current_price = candles.last()?.close;
        let mut indicator_scores = Vec::new();

        // Calculate all indicators and normalize scores
        // Momentum indicators
        if let Some(rsi) = calculate_rsi_default(candles) {
            let score = normalize_rsi(rsi.value);
            indicator_scores.push(IndicatorScore {
                name: "RSI".to_string(),
                score,
                category: IndicatorCategory::Momentum,
                weight: 1.0,
            });
        }

        if let Some(macd) = calculate_macd_default(candles) {
            let scale = 1.0; // Adjust based on typical MACD values
            let score = normalize_macd_histogram(macd.histogram, scale);
            indicator_scores.push(IndicatorScore {
                name: "MACD".to_string(),
                score,
                category: IndicatorCategory::Momentum,
                weight: 1.0,
            });
        }

        // Trend indicators
        if let Some(ema12) = calculate_ema(candles, 12) {
            if let Some(ema26) = calculate_ema(candles, 26) {
                let cross = if ema12.value > ema26.value { 1 } else { -1 };
                let score = normalize_ema_cross(cross);
                indicator_scores.push(IndicatorScore {
                    name: "EMA 12/26 Cross".to_string(),
                    score,
                    category: IndicatorCategory::Trend,
                    weight: 1.0,
                });
            }
        }

        if let Some(ema50) = calculate_ema(candles, 50) {
            if let Some(ema200) = calculate_ema(candles, 200) {
                let cross = if ema50.value > ema200.value { 1 } else { -1 };
                let score = normalize_ema_cross(cross);
                indicator_scores.push(IndicatorScore {
                    name: "EMA 50/200 Cross".to_string(),
                    score,
                    category: IndicatorCategory::Trend,
                    weight: 1.0,
                });
            }
        }

        if let Some(adx) = calculate_adx_default(candles) {
            let score = normalize_adx(adx.value);
            indicator_scores.push(IndicatorScore {
                name: "ADX".to_string(),
                score,
                category: IndicatorCategory::Trend,
                weight: 1.0,
            });
        }

        // Volatility indicators
        if let Some(bb) = calculate_bollinger_bands_default(candles) {
            let score = normalize_bollinger_position(current_price, bb.lower, bb.upper);
            indicator_scores.push(IndicatorScore {
                name: "Bollinger Bands".to_string(),
                score,
                category: IndicatorCategory::Volatility,
                weight: 1.0,
            });
        }

        let atr_value = calculate_atr_default(candles);
        let atr_value_for_sl_tp = atr_value.clone();
        if let Some(ref atr) = atr_value {
            let score = normalize_atr(atr.value, current_price);
            indicator_scores.push(IndicatorScore {
                name: "ATR".to_string(),
                score,
                category: IndicatorCategory::Volatility,
                weight: 1.0,
            });
        }

        // Market structure indicators
        if let Some(st) = calculate_supertrend_default(candles) {
            let score = normalize_supertrend(st.trend);
            indicator_scores.push(IndicatorScore {
                name: "SuperTrend".to_string(),
                score,
                category: IndicatorCategory::MarketStructure,
                weight: 1.0,
            });
        }

        if let Some(sr) = calculate_support_resistance_default(candles, current_price) {
            let score = normalize_support_resistance(
                current_price,
                sr.support_level,
                sr.resistance_level,
            );
            indicator_scores.push(IndicatorScore {
                name: "Support/Resistance".to_string(),
                score,
                category: IndicatorCategory::MarketStructure,
                weight: 1.0,
            });
        }

        if indicator_scores.is_empty() {
            return None;
        }

        // Aggregate by category
        let category_scores = Aggregator::aggregate_by_category(&indicator_scores);

        // Calculate global score (normalized -1 to +1)
        let global_score_normalized = Aggregator::calculate_global_score(&category_scores);

        // Convert to percentage (0 to 1) for direction decision
        let global_score_pct = DirectionThresholds::to_percentage(global_score_normalized);

        // Determine direction
        let direction = DirectionThresholds::determine_direction(global_score_pct);

        // Calculate SL/TP from ATR if we have a directional signal
        let (sl_pct, tp_pct) = if let Some(atr) = atr_value_for_sl_tp {
            match direction {
                SignalDirection::Long | SignalDirection::Short => {
                    StopLossTakeProfit::calculate_from_atr(atr.value, current_price)
                }
                SignalDirection::Neutral => (0.0, 0.0),
            }
        } else {
            (0.0, 0.0)
        };

        // Calculate confidence
        let confidence = calculate_confidence(global_score_normalized);

        // Generate reasons
        let reasons = Aggregator::generate_reasons(&indicator_scores, &category_scores, global_score_normalized);

        Some(SignalOutput::new(
            direction,
            confidence,
            sl_pct,
            tp_pct,
            reasons,
            symbol.to_string(),
            current_price,
        ))
    }

    /// Evaluate signal and return full indicator set
    pub fn evaluate_with_indicators(candles: &[Candle], symbol: &str) -> Option<(SignalOutput, IndicatorSet)> {
        let signal = Self::evaluate(candles, symbol)?;
        
        let mut indicator_set = IndicatorSet::new(symbol.to_string(), signal.price);
        
        if let Some(rsi) = calculate_rsi_default(candles) {
            indicator_set = indicator_set.with_rsi(rsi);
        }
        
        if let Some(macd) = calculate_macd_default(candles) {
            indicator_set = indicator_set.with_macd(macd);
        }
        
        // Add other indicators to indicator_set as needed
        // For now, we'll focus on the core functionality
        
        Some((signal, indicator_set))
    }
}


