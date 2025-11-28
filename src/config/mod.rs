use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryWeights {
    #[serde(default = "CategoryWeights::default_momentum")]
    pub momentum: f64,
    #[serde(default = "CategoryWeights::default_trend")]
    pub trend: f64,
    #[serde(default = "CategoryWeights::default_volatility")]
    pub volatility: f64,
    #[serde(default = "CategoryWeights::default_volume")]
    pub volume: f64,
    #[serde(default = "CategoryWeights::default_perp")]
    pub perp: f64,
}

impl CategoryWeights {
    fn default_momentum() -> f64 {
        0.25
    }

    fn default_trend() -> f64 {
        0.30
    }

    fn default_volatility() -> f64 {
        0.15
    }

    fn default_volume() -> f64 {
        0.15
    }

    fn default_perp() -> f64 {
        0.15
    }
}

impl Default for CategoryWeights {
    fn default() -> Self {
        Self {
            momentum: 0.25,
            trend: 0.30,
            volatility: 0.15,
            volume: 0.15,
            perp: 0.15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_sl_pct: f64,
    pub default_tp_pct: f64,
    pub rsi_overbought: f64,
    pub rsi_oversold: f64,
    pub min_confidence: f64,
    pub macd_scale: f64,
    pub hist_scale: f64,
    #[serde(default)]
    pub category_weights: CategoryWeights,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_sl_pct: 0.02,
            default_tp_pct: 0.04,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
            min_confidence: 0.5,
            macd_scale: 50.0,
            hist_scale: 25.0,
            category_weights: CategoryWeights::default(),
        }
    }
}

impl Config {
    pub fn new(
        default_sl_pct: f64,
        default_tp_pct: f64,
        rsi_overbought: f64,
        rsi_oversold: f64,
        min_confidence: f64,
        macd_scale: f64,
        hist_scale: f64,
    ) -> Self {
        Self {
            default_sl_pct,
            default_tp_pct,
            rsi_overbought,
            rsi_oversold,
            min_confidence,
            macd_scale,
            hist_scale,
            category_weights: CategoryWeights::default(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Get the Hyperliquid WebSocket URL based on environment
pub fn get_hyperliquid_ws_url() -> String {
    let env = std::env::var("PERPTRIX_ENV")
        .unwrap_or_else(|_| "production".to_string())
        .to_lowercase();

    match env.as_str() {
        "sandbox" | "testnet" => "wss://api.hyperliquid-testnet.xyz/ws".to_string(),
        _ => "wss://api.hyperliquid.xyz/ws".to_string(),
    }
}

/// Get the current environment (sandbox or production)
pub fn get_environment() -> String {
    std::env::var("PERPTRIX_ENV")
        .unwrap_or_else(|_| "production".to_string())
        .to_lowercase()
}

/// Get the QuestDB connection string
pub fn get_questdb_url() -> String {
    std::env::var("QUESTDB_URL")
        .unwrap_or_else(|_| "postgresql://admin:quest@localhost:8812/qdb".to_string())
}

/// Get the Redis connection string
pub fn get_redis_url() -> String {
    std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string())
}

/// Get the Hyperliquid REST API URL based on environment
pub fn get_hyperliquid_rest_url() -> String {
    let env = get_environment();
    match env.as_str() {
        "sandbox" | "testnet" => "https://api.hyperliquid-testnet.xyz".to_string(),
        _ => "https://api.hyperliquid.xyz".to_string(),
    }
}

/// Get the number of historical candles to fetch on startup
pub fn get_historical_candle_count() -> usize {
    std::env::var("HISTORICAL_CANDLE_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200)
}
