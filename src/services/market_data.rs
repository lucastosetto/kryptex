//! Market data provider interface for future data source integration.

use crate::models::indicators::Candle;

/// Trait for market data providers
pub trait MarketDataProvider {
    /// Get historical candles for a symbol
    fn get_candles(&self, symbol: &str, limit: usize) -> Result<Vec<Candle>, Box<dyn std::error::Error>>;
    
    /// Get the latest price for a symbol
    fn get_latest_price(&self, symbol: &str) -> Result<f64, Box<dyn std::error::Error>>;
    
    /// Subscribe to real-time candle updates (placeholder for future implementation)
    fn subscribe(&self, symbol: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// Placeholder market data provider for testing
pub struct PlaceholderMarketDataProvider;

impl MarketDataProvider for PlaceholderMarketDataProvider {
    fn get_candles(&self, _symbol: &str, _limit: usize) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }
    
    fn get_latest_price(&self, _symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
        Ok(0.0)
    }
    
    fn subscribe(&self, _symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}


