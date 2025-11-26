use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
}

impl Candle {
    pub fn new(open: f64, high: f64, low: f64, close: f64, volume: f64, timestamp: DateTime<Utc>) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
            timestamp,
        }
    }
}

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
pub struct BollingerBandsIndicator {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
    pub period: u32,
    pub std_dev: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtrIndicator {
    pub value: f64,
    pub period: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdxIndicator {
    pub value: f64,
    pub plus_di: f64,
    pub minus_di: f64,
    pub period: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperTrendIndicator {
    pub value: f64,
    pub trend: i32,
    pub upper_band: f64,
    pub lower_band: f64,
    pub period: u32,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportResistanceIndicator {
    pub support_level: Option<f64>,
    pub resistance_level: Option<f64>,
    pub support_distance_pct: Option<f64>,
    pub resistance_distance_pct: Option<f64>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bollinger_bands: Option<BollingerBandsIndicator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atr: Option<AtrIndicator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adx: Option<AdxIndicator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supertrend: Option<SuperTrendIndicator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_resistance: Option<SupportResistanceIndicator>,
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
            bollinger_bands: None,
            atr: None,
            adx: None,
            supertrend: None,
            support_resistance: None,
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
