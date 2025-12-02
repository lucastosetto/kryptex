//! Strategy evaluation engine that replaces hardcoded signal evaluation

use crate::indicators::momentum::{macd, rsi};
use crate::indicators::perp::{funding_rate, open_interest};
use crate::indicators::trend::{ema, supertrend};
use crate::indicators::volatility::{atr, bollinger};
use crate::indicators::volume::{obv, volume_profile};
use crate::models::indicators::Candle;
use crate::models::signal::{SignalDirection, SignalOutput, SignalReason};
use crate::models::strategy::{
    AggregationConfig, AggregationMethod, Comparison, Condition, IndicatorType, LogicalOperator,
    Rule, RuleResult, RuleType, Strategy,
};
use crate::signals::decision::StopLossTakeProfit;
use chrono::Utc;
use std::collections::VecDeque;

const MIN_CANDLES: usize = 50;
const VOLUME_PROFILE_LOOKBACK: usize = 240;
const VOLUME_PROFILE_TICK: f64 = 10.0;

/// Container for all computed indicator values
#[derive(Debug, Clone)]
pub struct IndicatorValues {
    // RSI
    pub rsi_value: Option<f64>,
    pub rsi_signal: Option<rsi::RSISignal>,
    
    // MACD
    pub macd_value: Option<f64>,
    pub macd_signal_value: Option<f64>,
    pub macd_histogram: Option<f64>,
    pub macd_signal: Option<macd::MACDSignal>,
    
    // EMA
    pub ema_fast: Option<f64>,
    pub ema_slow: Option<f64>,
    pub ema_signal: Option<ema::EMATrendSignal>,
    
    // SuperTrend
    pub supertrend_value: Option<f64>,
    pub supertrend_signal: Option<supertrend::SuperTrendSignal>,
    
    // Bollinger Bands
    pub bollinger_upper: Option<f64>,
    pub bollinger_middle: Option<f64>,
    pub bollinger_lower: Option<f64>,
    pub bollinger_signal: Option<bollinger::BollingerSignal>,
    
    // ATR
    pub atr_value: Option<f64>,
    pub volatility_regime: Option<atr::VolatilityRegime>,
    
    // OBV
    pub obv_signal: Option<obv::OBVSignal>,
    
    // Volume Profile
    pub volume_profile_signal: Option<volume_profile::VolumeProfileSignal>,
    
    // Open Interest
    pub oi_signal: Option<open_interest::OpenInterestSignal>,
    
    // Funding Rate
    pub funding_signal: Option<funding_rate::FundingSignal>,
    pub funding_rate_value: Option<f64>,
    
    // Current price
    pub current_price: f64,
}

impl IndicatorValues {
    pub fn new(current_price: f64) -> Self {
        Self {
            rsi_value: None,
            rsi_signal: None,
            macd_value: None,
            macd_signal_value: None,
            macd_histogram: None,
            macd_signal: None,
            ema_fast: None,
            ema_slow: None,
            ema_signal: None,
            supertrend_value: None,
            supertrend_signal: None,
            bollinger_upper: None,
            bollinger_middle: None,
            bollinger_lower: None,
            bollinger_signal: None,
            atr_value: None,
            volatility_regime: None,
            obv_signal: None,
            volume_profile_signal: None,
            oi_signal: None,
            funding_signal: None,
            funding_rate_value: None,
            current_price,
        }
    }
}

pub struct StrategyEvaluator;

impl StrategyEvaluator {
    /// Evaluate a strategy against candles
    pub fn evaluate_strategy(
        strategy: &Strategy,
        candles: &[Candle],
    ) -> Option<SignalOutput> {
        if candles.len() < MIN_CANDLES {
            return None;
        }

        let current_price = candles.last()?.close;
        let indicator_values = Self::compute_indicators(candles, current_price);

        // Evaluate all rules
        let mut rule_results = Vec::new();
        for rule in &strategy.config.rules {
            if let Some(result) = Self::evaluate_rule(rule, &indicator_values) {
                rule_results.push(result);
            }
        }

        if rule_results.is_empty() {
            return None;
        }

        // Aggregate results
        let total_score = Self::aggregate_results(&rule_results, &strategy.config.aggregation);
        
        // Determine signal direction from score
        let direction = if total_score >= strategy.config.aggregation.thresholds.long_min {
            SignalDirection::Long
        } else if total_score <= strategy.config.aggregation.thresholds.short_max {
            SignalDirection::Short
        } else {
            SignalDirection::Neutral
        };

        // Calculate confidence (simplified - based on score magnitude)
        let max_possible_score = rule_results.iter().map(|r| r.weight.abs() as i32).sum::<i32>().max(1);
        let confidence = (total_score.abs() as f64 / max_possible_score as f64).min(1.0);

        // Calculate SL/TP from ATR if available
        let (sl_pct, tp_pct) = if let Some(atr) = indicator_values.atr_value {
            if atr > 0.0 {
                StopLossTakeProfit::calculate_from_atr(atr, current_price)
            } else {
                (0.0, 0.0)
            }
        } else {
            (0.0, 0.0)
        };

        // Build reasons
        let reasons: Vec<SignalReason> = rule_results
            .iter()
            .filter(|r| r.passed)
            .map(|r| SignalReason {
                description: format!("Rule {} passed (score: {})", r.rule_id, r.score),
                weight: r.weight,
            })
            .collect();

        Some(SignalOutput {
            direction,
            confidence,
            recommended_sl_pct: sl_pct,
            recommended_tp_pct: tp_pct,
            reasons,
            symbol: strategy.symbol.clone(),
            price: current_price,
            timestamp: Utc::now(),
        })
    }

    /// Compute all indicator values from candles
    fn compute_indicators(candles: &[Candle], current_price: f64) -> IndicatorValues {
        let mut values = IndicatorValues::new(current_price);

        // Initialize indicators with default parameters
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

        for candle in candles {
            // Update indicators
            values.ema_signal = Some(ema_cross.update(candle.close));
            values.supertrend_signal = Some(supertrend.update(candle.high, candle.low, candle.close));
            
            if let Some(rsi_value) = rsi.update(candle.close) {
                values.rsi_value = Some(rsi_value);
                if let Some(prev) = prev_close {
                    let price_change = candle.close - prev;
                    values.rsi_signal = Some(rsi.get_signal(rsi_value, price_change));
                }
            }

            let (macd_val, macd_sig_val, macd_hist, macd_sig) = macd.update(candle.close);
            values.macd_value = Some(macd_val);
            values.macd_signal_value = Some(macd_sig_val);
            values.macd_histogram = Some(macd_hist);
            values.macd_signal = Some(macd_sig);

            let (bb_upper, bb_middle, bb_lower, bb_sig) = bollinger.update(candle.close);
            values.bollinger_upper = Some(bb_upper);
            values.bollinger_middle = Some(bb_middle);
            values.bollinger_lower = Some(bb_lower);
            values.bollinger_signal = Some(bb_sig);

            let atr_value = atr.update(candle.high, candle.low, candle.close);
            atr_history.push_back(atr_value);
            if atr_history.len() > 14 {
                atr_history.pop_front();
            }
            values.atr_value = Some(atr_value);
            let lookback_avg = if atr_history.is_empty() {
                atr_value
            } else {
                atr_history.iter().sum::<f64>() / atr_history.len() as f64
            };
            values.volatility_regime = Some(atr.get_volatility_regime(atr_value, lookback_avg));

            let (_, obv_sig) = obv.update(candle.close, candle.volume);
            values.obv_signal = Some(obv_sig);

            volume_profile.update(candle.close, candle.volume);
            let (_, _, vp_sig) = volume_profile.get_profile();
            values.volume_profile_signal = Some(vp_sig);

            if let Some(oi) = candle.open_interest {
                values.oi_signal = Some(open_interest.update(oi, candle.close));
            }

            if let Some(funding) = candle.funding_rate {
                let (funding_sig, _) = funding_rate.update(funding);
                values.funding_signal = Some(funding_sig);
                values.funding_rate_value = Some(funding);
            }

            // Store EMA values
            if let (Some(fast), Some(slow)) = (ema_cross.fast(), ema_cross.slow()) {
                values.ema_fast = Some(fast);
                values.ema_slow = Some(slow);
            }

            // Store SuperTrend value
            values.supertrend_value = supertrend.value();

            prev_close = Some(candle.close);
        }

        values
    }

    /// Evaluate a rule (condition or group)
    fn evaluate_rule(rule: &Rule, indicator_values: &IndicatorValues) -> Option<RuleResult> {
        match rule.rule_type {
            RuleType::Condition => {
                if let Some(ref condition) = rule.condition {
                    let passed = Self::evaluate_condition(condition, indicator_values);
                    let score = if passed {
                        rule.weight.unwrap_or(1.0) as i32
                    } else {
                        -(rule.weight.unwrap_or(1.0) as i32)
                    };
                    Some(RuleResult::new(
                        rule.id.clone(),
                        passed,
                        score,
                        rule.weight.unwrap_or(1.0),
                    ))
                } else {
                    None
                }
            }
            RuleType::Group | RuleType::WeightedGroup => {
                if let Some(ref children) = rule.children {
                    let mut child_results = Vec::new();
                    for child in children {
                        if let Some(result) = Self::evaluate_rule(child, indicator_values) {
                            child_results.push(result);
                        }
                    }

                    if child_results.is_empty() {
                        return None;
                    }

                    let passed = if let Some(op) = rule.operator {
                        match op {
                            LogicalOperator::AND => child_results.iter().all(|r| r.passed),
                            LogicalOperator::OR => child_results.iter().any(|r| r.passed),
                        }
                    } else {
                        // Default to AND if no operator specified
                        child_results.iter().all(|r| r.passed)
                    };

                    let score: i32 = child_results.iter().map(|r| r.score).sum();
                    let weight = rule.weight.unwrap_or(1.0);

                    Some(RuleResult::new(rule.id.clone(), passed, score, weight))
                } else {
                    None
                }
            }
        }
    }

    /// Evaluate a condition against indicator values
    fn evaluate_condition(condition: &Condition, indicator_values: &IndicatorValues) -> bool {
        match condition.comparison {
            Comparison::SignalState => {
                if let Some(ref signal_state) = condition.signal_state {
                    Self::check_signal_state(condition.indicator, signal_state, indicator_values)
                } else {
                    false
                }
            }
            _ => {
                // For numeric comparisons, get the indicator value
                let value = Self::get_indicator_value(condition.indicator, indicator_values);
                if let Some(val) = value {
                    Self::compare_value(val, condition.comparison, condition.threshold)
                } else {
                    false
                }
            }
        }
    }

    /// Get numeric value for an indicator
    fn get_indicator_value(indicator: IndicatorType, values: &IndicatorValues) -> Option<f64> {
        match indicator {
            IndicatorType::RSI => values.rsi_value,
            IndicatorType::MACD => values.macd_value,
            IndicatorType::EMA => values.ema_fast,
            IndicatorType::ATR => values.atr_value,
            IndicatorType::Bollinger => values.bollinger_middle,
            IndicatorType::SuperTrend => values.supertrend_value,
            IndicatorType::FundingRate => values.funding_rate_value,
            _ => None, // OBV, VolumeProfile, OpenInterest don't have simple numeric values
        }
    }

    /// Check signal state for an indicator
    fn check_signal_state(
        indicator: IndicatorType,
        signal_state: &str,
        values: &IndicatorValues,
    ) -> bool {
        match indicator {
            IndicatorType::RSI => {
                if let Some(signal) = values.rsi_signal {
                    match signal_state {
                        "Oversold" => matches!(signal, rsi::RSISignal::Oversold),
                        "Overbought" => matches!(signal, rsi::RSISignal::Overbought),
                        "BullishDivergence" => matches!(signal, rsi::RSISignal::BullishDivergence),
                        "BearishDivergence" => matches!(signal, rsi::RSISignal::BearishDivergence),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            IndicatorType::EMA => {
                if let Some(signal) = values.ema_signal {
                    match signal_state {
                        "BullishCross" => matches!(signal, ema::EMATrendSignal::BullishCross),
                        "BearishCross" => matches!(signal, ema::EMATrendSignal::BearishCross),
                        "StrongUptrend" => matches!(signal, ema::EMATrendSignal::StrongUptrend),
                        "StrongDowntrend" => matches!(signal, ema::EMATrendSignal::StrongDowntrend),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            IndicatorType::MACD => {
                if let Some(signal) = values.macd_signal {
                    match signal_state {
                        "BullishCross" => matches!(signal, macd::MACDSignal::BullishCross),
                        "BearishCross" => matches!(signal, macd::MACDSignal::BearishCross),
                        "BullishMomentum" => matches!(signal, macd::MACDSignal::BullishMomentum),
                        "BearishMomentum" => matches!(signal, macd::MACDSignal::BearishMomentum),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false, // Other indicators not yet implemented
        }
    }

    /// Compare a value using the specified comparison operator
    fn compare_value(value: f64, comparison: Comparison, threshold: Option<f64>) -> bool {
        if let Some(thresh) = threshold {
            match comparison {
                Comparison::GreaterThan => value > thresh,
                Comparison::LessThan => value < thresh,
                Comparison::GreaterEqual => value >= thresh,
                Comparison::LessEqual => value <= thresh,
                Comparison::Equal => (value - thresh).abs() < 0.0001,
                Comparison::NotEqual => (value - thresh).abs() >= 0.0001,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Aggregate rule results according to aggregation config
    fn aggregate_results(results: &[RuleResult], config: &AggregationConfig) -> i32 {
        match config.method {
            AggregationMethod::Sum => results.iter().map(|r| r.score).sum(),
            AggregationMethod::WeightedSum => {
                results.iter().map(|r| (r.score as f64 * r.weight) as i32).sum()
            }
            AggregationMethod::Majority => {
                let positive = results.iter().filter(|r| r.score > 0).count();
                let negative = results.iter().filter(|r| r.score < 0).count();
                if positive > negative {
                    positive as i32
                } else if negative > positive {
                    -(negative as i32)
                } else {
                    0
                }
            }
            AggregationMethod::All => {
                if results.iter().all(|r| r.passed) {
                    results.iter().map(|r| r.score).sum()
                } else {
                    0
                }
            }
            AggregationMethod::Any => {
                if results.iter().any(|r| r.passed) {
                    results.iter().map(|r| r.score).sum()
                } else {
                    0
                }
            }
        }
    }
}

