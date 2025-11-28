//! Periodic task runner for continuous signal evaluation

use crate::services::market_data::{MarketDataProvider, PlaceholderMarketDataProvider};
use crate::signals::engine::SignalEngine;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info};

pub struct RuntimeConfig {
    pub evaluation_interval_seconds: u64,
    pub symbols: Vec<String>,
}

pub struct SignalRuntime {
    config: RuntimeConfig,
    data_provider: Arc<dyn MarketDataProvider + Send + Sync>,
    database: Option<Arc<crate::db::QuestDatabase>>,
}

impl SignalRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            data_provider: Arc::new(PlaceholderMarketDataProvider),
            database: None,
        }
    }

    pub fn with_provider<P: MarketDataProvider + Send + Sync + 'static>(
        config: RuntimeConfig,
        provider: P,
    ) -> Self {
        Self {
            config,
            data_provider: Arc::new(provider),
            database: None,
        }
    }

    pub fn with_database(mut self, database: Arc<crate::db::QuestDatabase>) -> Self {
        self.database = Some(database);
        self
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval_timer =
            interval(Duration::from_secs(self.config.evaluation_interval_seconds));

        info!(
            interval = self.config.evaluation_interval_seconds,
            "Signal runtime started. Evaluating signals every {} seconds",
            self.config.evaluation_interval_seconds
        );

        loop {
            interval_timer.tick().await;

            for symbol in &self.config.symbols {
                match self.evaluate_signal(symbol).await {
                    Ok(Some(signal)) => {
                        info!(
                            symbol = %symbol,
                            direction = ?signal.direction,
                            confidence = signal.confidence * 100.0,
                            "Signal for {}: {:?} (confidence: {:.2}%)",
                            symbol,
                            signal.direction,
                            signal.confidence * 100.0
                        );
                        
                        // Store signal in database if available
                        if let Some(ref db) = self.database {
                            if let Err(e) = db.store_signal(&signal).await {
                                error!(symbol = %symbol, error = %e, "Failed to store signal in database");
                            }
                        }
                    }
                    Ok(None) => {
                        debug!(symbol = %symbol, "No signal generated for {}", symbol);
                    }
                    Err(e) => {
                        error!(symbol = %symbol, error = %e, "Error evaluating signal for {}", symbol);
                    }
                }
            }
        }
    }

    /// Evaluate signal for a symbol
    async fn evaluate_signal(
        &self,
        symbol: &str,
    ) -> Result<Option<crate::models::signal::SignalOutput>, Box<dyn std::error::Error + Send + Sync>> {
        let candles = self.data_provider.get_candles(symbol, 250).await
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Market data error: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

        if candles.is_empty() {
            debug!(symbol = %symbol, "No candles available yet - waiting for WebSocket data");
            return Ok(None);
        }

        debug!(
            symbol = %symbol,
            candle_count = candles.len(),
            min_candles = crate::signals::engine::MIN_CANDLES,
            "Evaluating with {} candles (need {})",
            candles.len(),
            crate::signals::engine::MIN_CANDLES
        );
        
        if candles.len() < crate::signals::engine::MIN_CANDLES {
            debug!(
                symbol = %symbol,
                candle_count = candles.len(),
                min_candles = crate::signals::engine::MIN_CANDLES,
                "Not enough candles ({} < {}) - waiting for more candles to accumulate (1m candles arrive every minute)",
                candles.len(),
                crate::signals::engine::MIN_CANDLES
            );
            return Ok(None);
        }
        
        let signal = SignalEngine::evaluate(&candles, symbol);
        
        if signal.is_none() {
            debug!(symbol = %symbol, "Signal evaluation returned None (likely insufficient data or neutral score)");
        } else if let Some(ref sig) = signal {
            debug!(
                symbol = %symbol,
                direction = ?sig.direction,
                confidence = sig.confidence * 100.0,
                reasons = ?sig.reasons,
                "Signal generated - Direction: {:?}, Confidence: {:.2}%, Reasons: {:?}",
                sig.direction,
                sig.confidence * 100.0,
                sig.reasons
            );
        }
        
        Ok(signal)
    }
}

