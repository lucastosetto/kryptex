use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::indicators::IndicatorSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalDirection {
    Long,
    Short,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalReason {
    pub description: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalOutput {
    pub direction: SignalDirection,
    pub confidence: f64,
    pub recommended_sl_pct: f64,
    pub recommended_tp_pct: f64,
    pub reasons: Vec<SignalReason>,
    pub symbol: String,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

impl SignalOutput {
    pub fn new(
        direction: SignalDirection,
        confidence: f64,
        recommended_sl_pct: f64,
        recommended_tp_pct: f64,
        reasons: Vec<SignalReason>,
        symbol: String,
        price: f64,
    ) -> Self {
        Self {
            direction,
            confidence,
            recommended_sl_pct,
            recommended_tp_pct,
            reasons,
            symbol,
            price,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEvaluation {
    pub signal: SignalOutput,
    pub indicators: IndicatorSet,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluator_version: Option<String>,
    pub evaluated_at: DateTime<Utc>,
}

impl SignalEvaluation {
    pub fn new(signal: SignalOutput, indicators: IndicatorSet) -> Self {
        Self {
            signal,
            indicators,
            evaluator_version: None,
            evaluated_at: Utc::now(),
        }
    }

    pub fn with_evaluator_version(mut self, version: String) -> Self {
        self.evaluator_version = Some(version);
        self
    }
}
