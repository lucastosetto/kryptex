//! Main signal evaluation engine powered by strategy-based evaluation.

use crate::models::indicators::{Candle, IndicatorSet};
use crate::models::signal::SignalOutput;
use crate::models::strategy::Strategy;
use crate::strategies::evaluator::StrategyEvaluator;

pub const MIN_CANDLES: usize = 50;

pub struct SignalEngine;

impl SignalEngine {
    /// Evaluate signal from candles using a strategy.
    /// This replaces the hardcoded evaluation logic.
    pub fn evaluate(candles: &[Candle], strategy: &Strategy) -> Option<SignalOutput> {
        StrategyEvaluator::evaluate_strategy(strategy, candles)
    }

    /// Evaluate signal and return full indicator set (for API responses/debugging)
    pub fn evaluate_with_indicators(
        candles: &[Candle],
        strategy: &Strategy,
    ) -> Option<(SignalOutput, IndicatorSet)> {
        let signal = Self::evaluate(candles, strategy)?;
        let mut indicator_set = IndicatorSet::new(strategy.symbol.clone(), signal.price);

        if let Some(funding_rate) = candles.last().and_then(|c| c.funding_rate) {
            indicator_set = indicator_set.with_funding_rate(funding_rate);
        }

        if let Some(open_interest) = candles.last().and_then(|c| c.open_interest) {
            indicator_set = indicator_set.with_open_interest(open_interest);
        }

        Some((signal, indicator_set))
    }
}
