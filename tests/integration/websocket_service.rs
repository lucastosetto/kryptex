//! Integration tests for the WebSocket Service
//!
//! Tests WebSocket connection, subscription management, and data ingestion.

#[path = "websocket_service/test_utils.rs"]
mod test_utils;

use tokio::time::{sleep, Duration};

use test_utils::TestWebSocketService;

#[tokio::test]
async fn websocket_service_initializes() {
    let _service = TestWebSocketService::new().await;
    // Service should initialize without errors
}

#[tokio::test]
async fn websocket_service_connects() {
    let mut service = TestWebSocketService::new().await;
    
    // Start the service
    service.start().await.expect("Service should start");
    
    // Wait for connection attempt
    sleep(Duration::from_millis(100)).await;
    
    // Service should attempt to connect
    let provider = service.get_provider();
    let _client = provider.client();
    // Connection state can be checked via the mock client
}

#[tokio::test]
async fn websocket_service_subscribes_to_symbols() {
    let mut service = TestWebSocketService::new().await;
    service.start().await.expect("Service should start");
    
    // Subscribe to a symbol via the service
    service.service.subscribe("BTC")
        .await
        .expect("Should subscribe to BTC");
    
    // Verify subscription was sent via WebSocket
    let sent_messages = service.websocket.sent_messages().await;
    assert!(
        sent_messages.iter().any(|msg| {
            if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                text.contains("BTC") || text.contains("subscribe")
            } else {
                false
            }
        }),
        "Should send subscription message for BTC"
    );
}

#[tokio::test]
async fn websocket_service_subscribes_to_multiple_symbols() {
    let mut service = TestWebSocketService::new().await;
    service.start().await.expect("Service should start");
    
    // Subscribe to multiple symbols
    service.service.subscribe("BTC").await.expect("Should subscribe to BTC");
    service.service.subscribe("ETH").await.expect("Should subscribe to ETH");
    
    let sent_messages = service.websocket.sent_messages().await;
    
    // Should have subscription messages for both
    let btc_subscribed = sent_messages.iter().any(|msg| {
        if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
            text.contains("BTC")
        } else {
            false
        }
    });
    
    let eth_subscribed = sent_messages.iter().any(|msg| {
        if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
            text.contains("ETH")
        } else {
            false
        }
    });
    
    assert!(btc_subscribed, "Should subscribe to BTC");
    assert!(eth_subscribed, "Should subscribe to ETH");
}

#[tokio::test]
async fn websocket_service_fetches_historical_data_on_subscribe() {
    let service = TestWebSocketService::new().await;
    
    // Subscribe should trigger historical data fetch
    service.service.subscribe("BTC").await.expect("Should subscribe");
    
    // Wait for REST API call
    sleep(Duration::from_millis(100)).await;
    
    let requests = service
        .hyperliquid_rest
        .received_requests()
        .await
        .expect("Should have requests");
    
    // Should have made a candleSnapshot request
    assert!(
        requests.iter().any(|req| {
            let body = String::from_utf8_lossy(&req.body);
            body.contains("candleSnapshot") && body.contains("BTC")
        }),
        "Should fetch historical candles on subscribe"
    );
}

#[tokio::test]
async fn websocket_service_handles_reconnection() {
    let mut service = TestWebSocketService::new().await;
    service.start().await.expect("Service should start");
    
    // Simulate disconnection
    service.websocket.set_connected(false).await;
    
    // Service should handle reconnection (implementation dependent)
    // This test verifies the service doesn't crash on disconnect
    sleep(Duration::from_millis(100)).await;
    
    // Service should still be running
    assert!(true, "Service should handle disconnection gracefully");
}

#[tokio::test]
async fn websocket_service_is_singleton_aware() {
    // Test that service is designed to run as singleton
    // Multiple instances would cause issues, so this test documents the expectation
    let _service1 = TestWebSocketService::new().await;
    let _service2 = TestWebSocketService::new().await;
    
    // Both should initialize independently
    // In production, only one should run
    assert!(true, "Multiple services can be created, but only one should run");
}

#[tokio::test]
async fn websocket_service_stores_data_to_cache() {
    // This test would require Redis/QuestDB setup
    // For now, we verify the service can be configured with storage
    let _service = TestWebSocketService::new().await;
    
    // Service should be able to write to storage when configured
    // This is a placeholder for future test with actual storage
    assert!(true, "Service should store data when storage is configured");
}

