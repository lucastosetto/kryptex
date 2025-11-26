use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacdIndicator {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<(u32, u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsiIndicator {
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaIndicator {
    pub value: f64,
    pub period: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmaIndicator {
    pub value: f64,
    pub period: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeIndicator {
    pub volume: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_ma: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_ma_period: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorSet {
    pub symbol: String,
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macd: Option<MacdIndicator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rsi: Option<RsiIndicator>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub emas: Vec<EmaIndicator>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub smas: Vec<SmaIndicator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<VolumeIndicator>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeframe: Option<String>,
}

impl IndicatorSet {
    pub fn new(symbol: String, price: f64) -> Self {
        Self {
            symbol,
            price,
            funding_rate: None,
            macd: None,
            rsi: None,
            emas: Vec::new(),
            smas: Vec::new(),
            volume: None,
            timestamp: Utc::now(),
            timeframe: None,
        }
    }

    pub fn with_macd(mut self, macd: MacdIndicator) -> Self {
        self.macd = Some(macd);
        self
    }

    pub fn with_rsi(mut self, rsi: RsiIndicator) -> Self {
        self.rsi = Some(rsi);
        self
    }

    pub fn with_funding_rate(mut self, funding_rate: f64) -> Self {
        self.funding_rate = Some(funding_rate);
        self
    }

    pub fn with_timeframe(mut self, timeframe: String) -> Self {
        self.timeframe = Some(timeframe);
        self
    }
}
