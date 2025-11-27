//! Unit tests for market data provider

use perptrix::services::market_data::{MarketDataProvider, PlaceholderMarketDataProvider};

#[test]
fn test_placeholder_provider() {
    let provider = PlaceholderMarketDataProvider;
    assert!(provider.get_candles("BTC", 100).is_ok());
    assert!(provider.get_latest_price("BTC").is_ok());
    assert!(provider.subscribe("BTC").is_ok());
}



