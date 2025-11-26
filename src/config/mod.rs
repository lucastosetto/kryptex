use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_sl_pct: f64,
    pub default_tp_pct: f64,
    pub rsi_overbought: f64,
    pub rsi_oversold: f64,
    pub min_confidence: f64,
    pub default_symbol: String,
    pub macd_scale: f64,
    pub hist_scale: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_sl_pct: 0.02,
            default_tp_pct: 0.04,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
            min_confidence: 0.5,
            default_symbol: "BTC".to_string(),
            macd_scale: 50.0,
            hist_scale: 25.0,
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
        default_symbol: String,
        macd_scale: f64,
        hist_scale: f64,
    ) -> Self {
        Self {
            default_sl_pct,
            default_tp_pct,
            rsi_overbought,
            rsi_oversold,
            min_confidence,
            default_symbol,
            macd_scale,
            hist_scale,
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

