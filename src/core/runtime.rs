//! Periodic task runner for continuous signal evaluation

use crate::services::market_data::{MarketDataProvider, PlaceholderMarketDataProvider};
use crate::signals::engine::SignalEngine;
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// Runtime configuration
pub struct RuntimeConfig {
    pub evaluation_interval_seconds: u64,
    pub symbols: Vec<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            evaluation_interval_seconds: 60,
            symbols: vec!["BTC".to_string()],
        }
    }
}

/// Signal engine runtime
pub struct SignalRuntime {
    config: RuntimeConfig,
    data_provider: Arc<dyn MarketDataProvider + Send + Sync>,
}

impl SignalRuntime {
    /// Create new runtime
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            data_provider: Arc::new(PlaceholderMarketDataProvider),
        }
    }

    /// Create runtime with custom data provider
    pub fn with_provider<P: MarketDataProvider + Send + Sync + 'static>(
        config: RuntimeConfig,
        provider: P,
    ) -> Self {
        Self {
            config,
            data_provider: Arc::new(provider),
        }
    }

    /// Run periodic signal evaluation
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval_timer = interval(Duration::from_secs(self.config.evaluation_interval_seconds));
        
        println!("Signal runtime started. Evaluating signals every {} seconds", 
                 self.config.evaluation_interval_seconds);
        
        loop {
            interval_timer.tick().await;
            
            for symbol in &self.config.symbols {
                match self.evaluate_signal(symbol).await {
                    Ok(Some(signal)) => {
                        println!("Signal for {}: {:?} (confidence: {:.2}%)", 
                                 symbol, signal.direction, signal.confidence * 100.0);
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
    async fn evaluate_signal(&self, symbol: &str) -> Result<Option<crate::models::signal::SignalOutput>, Box<dyn std::error::Error>> {
        // Get candles from data provider
        let candles = self.data_provider.get_candles(symbol, 250)?;
        
        if candles.is_empty() {
            return Ok(None);
        }
        
        // Evaluate signal
        let signal = SignalEngine::evaluate(&candles, symbol);
        Ok(signal)
    }
}


