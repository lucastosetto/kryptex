//! Periodic task runner for continuous signal evaluation

use crate::services::market_data::{MarketDataProvider, PlaceholderMarketDataProvider};
use crate::signals::engine::SignalEngine;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct RuntimeConfig {
    pub evaluation_interval_seconds: u64,
    pub symbols: Vec<String>,
}

pub struct SignalRuntime {
    config: RuntimeConfig,
    data_provider: Arc<dyn MarketDataProvider + Send + Sync>,
}

impl SignalRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            data_provider: Arc::new(PlaceholderMarketDataProvider),
        }
    }

    pub fn with_provider<P: MarketDataProvider + Send + Sync + 'static>(
        config: RuntimeConfig,
        provider: P,
    ) -> Self {
        Self {
            config,
            data_provider: Arc::new(provider),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval_timer =
            interval(Duration::from_secs(self.config.evaluation_interval_seconds));

        println!(
            "Signal runtime started. Evaluating signals every {} seconds",
            self.config.evaluation_interval_seconds
        );

        loop {
            interval_timer.tick().await;

            for symbol in &self.config.symbols {
                match self.evaluate_signal(symbol).await {
                    Ok(Some(signal)) => {
                        println!(
                            "Signal for {}: {:?} (confidence: {:.2}%)",
                            symbol,
                            signal.direction,
                            signal.confidence * 100.0
                        );
                    }
                    Ok(None) => {
                        println!("No signal generated for {}", symbol);
                    }
                    Err(e) => {
                        eprintln!("Error evaluating signal for {}: {}", symbol, e);
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
            println!("  [DEBUG] {}: No candles available yet - waiting for WebSocket data", symbol);
            return Ok(None);
        }

        println!("  [DEBUG] {}: Evaluating with {} candles (need {})", symbol, candles.len(), crate::signals::engine::MIN_CANDLES);
        
        if candles.len() < crate::signals::engine::MIN_CANDLES {
            println!("  [DEBUG] {}: Not enough candles ({} < {}) - waiting for more candles to accumulate (1m candles arrive every minute)", symbol, candles.len(), crate::signals::engine::MIN_CANDLES);
            return Ok(None);
        }
        
        let signal = SignalEngine::evaluate(&candles, symbol);
        
        if signal.is_none() {
            println!("  [DEBUG] {}: Signal evaluation returned None (likely insufficient data or neutral score)", symbol);
        } else if let Some(ref sig) = signal {
            println!("  [DEBUG] {}: Signal generated - Direction: {:?}, Confidence: {:.2}%, Reasons: {:?}", 
                symbol,
                sig.direction,
                sig.confidence * 100.0,
                sig.reasons
            );
        }
        
        Ok(signal)
    }
}
