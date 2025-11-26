use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacdSignal {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorInput {
    pub macd: MacdSignal,
    pub rsi: f64,
    pub funding_rate: Option<f64>,
    pub price: f64,
    pub symbol: Option<String>,
}

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
    pub timestamp: chrono::DateTime<chrono::Utc>,
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
            timestamp: chrono::Utc::now(),
        }
    }
}

