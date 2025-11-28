//! Hyperliquid REST API client for fetching historical candles

use crate::config;
use crate::models::indicators::Candle;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
struct HyperliquidCandleResponse {
    #[serde(rename = "t")]
    #[allow(dead_code)]
    start_time: u64,
    #[serde(rename = "T")]
    end_time: u64,
    #[serde(rename = "s")]
    #[allow(dead_code)]
    coin: String,
    #[serde(rename = "i")]
    #[allow(dead_code)]
    interval: String,
    #[serde(rename = "o")]
    open: String,
    #[serde(rename = "h")]
    high: String,
    #[serde(rename = "l")]
    low: String,
    #[serde(rename = "c")]
    close: String,
    #[serde(rename = "v")]
    volume: String,
    #[serde(rename = "n", default)]
    #[allow(dead_code)]
    trades: Option<u64>,
}


pub struct HyperliquidRestClient {
    base_url: String,
    client: reqwest::Client,
}

impl HyperliquidRestClient {
    pub fn new() -> Self {
        Self {
            base_url: config::get_hyperliquid_rest_url(),
            client: reqwest::Client::new(),
        }
    }

    /// Fetch historical candles for a symbol and interval
    /// 
    /// # Arguments
    /// * `coin` - The coin symbol (e.g., "BTC")
    /// * `interval` - The candle interval (e.g., "1m", "5m", "15m", "1h")
    /// * `count` - Number of candles to fetch
    /// 
    /// # Returns
    /// Vector of Candle objects sorted by timestamp (oldest first)
    pub async fn fetch_historical_candles(
        &self,
        coin: &str,
        interval: &str,
        count: usize,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        // Hyperliquid REST API uses POST with JSON body
        // Format: {"type":"candleSnapshot","req":{"coin":"BTC","interval":"1m","startTime":...,"endTime":...}}
        let url = format!("{}/info", self.base_url);
        
        // Calculate timestamps based on interval and count
        let now = Utc::now();
        let end_time = now.timestamp_millis() as u64;
        
        // Calculate start time based on interval duration
        let interval_seconds = match interval {
            "1m" => 60,
            "5m" => 300,
            "15m" => 900,
            "1h" => 3600,
            "4h" => 14400,
            "1d" => 86400,
            _ => 60, // default to 1 minute
        };
        
        // Add some buffer (extra 10% to ensure we get enough candles)
        let duration_ms = (interval_seconds * count as u64 * 110 / 100) * 1000;
        let start_time = end_time.saturating_sub(duration_ms);
        
        let request_body = serde_json::json!({
            "type": "candleSnapshot",
            "req": {
                "coin": coin,
                "interval": interval,
                "startTime": start_time,
                "endTime": end_time
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("HTTP request failed: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

        let status = response.status();
        let text = response.text().await.map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read response: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        if !status.is_success() {
            debug!(status = %status, response = %text, "Hyperliquid REST API error response");
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP error: {} - Response: {}", status, text),
            )) as Box<dyn std::error::Error + Send + Sync>);
        }

        // Parse the response - Hyperliquid may return different formats
        // Try parsing as array of candles first
        let candles: Vec<HyperliquidCandleResponse> = serde_json::from_str(&text)
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse candles response: {} - Response: {}", e, text),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

        let mut result = Vec::new();
        for candle in candles {
            let open: f64 = candle.open.parse().map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid open price: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
            let high: f64 = candle.high.parse().map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid high price: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
            let low: f64 = candle.low.parse().map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid low price: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
            let close: f64 = candle.close.parse().map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid close price: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;
            let volume: f64 = candle.volume.parse().map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid volume: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

            // Use end_time as the candle timestamp (when the candle closed)
            let timestamp = DateTime::from_timestamp(candle.end_time as i64 / 1000, 0)
                .unwrap_or_else(Utc::now);

            result.push(Candle::new(open, high, low, close, volume, timestamp));
        }

        // Sort by timestamp (oldest first)
        result.sort_by_key(|c| c.timestamp);

        Ok(result)
    }
}

impl Default for HyperliquidRestClient {
    fn default() -> Self {
        Self::new()
    }
}

