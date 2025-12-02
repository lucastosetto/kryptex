//! Job types for the signal evaluation workflow

use crate::models::indicators::Candle;
use crate::models::signal::SignalOutput;
use serde::{Deserialize, Serialize};

/// Job to fetch candles for a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchCandlesJob {
    pub symbol: String,
}

/// Job to evaluate a signal from candles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateSignalJob {
    pub symbol: String,
    pub candles: Vec<Candle>,
}

/// Job to store a signal in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSignalJob {
    pub symbol: String,
    pub signal: SignalOutput,
    pub strategy_id: i64,
}




