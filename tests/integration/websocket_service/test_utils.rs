//! Test utilities for WebSocket service integration tests

use perptrix::services::hyperliquid::{
    HyperliquidMarketDataProvider, HyperliquidRestClient, MockWebSocketClient,
};
use perptrix::services::websocket::WebSocketService;
use std::sync::Arc;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test helper for WebSocket service integration tests
pub struct TestWebSocketService {
    pub websocket: Arc<MockWebSocketClient>,
    pub hyperliquid_rest: MockServer,
    pub service: WebSocketService,
}

impl TestWebSocketService {
    pub async fn new() -> Self {
        let mock_server = MockServer::start().await;
        mock_hyperliquid_candles(&mock_server).await;
        mock_hyperliquid_funding_history(&mock_server).await;

        let websocket = Arc::new(MockWebSocketClient::new());
        let rest_client = Arc::new(HyperliquidRestClient::with_client(
            mock_server.uri(),
            reqwest::Client::new(),
        ));

        let provider = HyperliquidMarketDataProvider::with_clients(
            websocket.clone(),
            rest_client,
            vec!["1m".to_string()],
        );

        let service = WebSocketService::new(provider);

        Self {
            websocket,
            hyperliquid_rest: mock_server,
            service,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.service.start().await
    }

    pub fn get_provider(&self) -> Arc<HyperliquidMarketDataProvider> {
        self.service.get_provider()
    }
}

pub async fn mock_hyperliquid_candles(server: &MockServer) {
    let response = serde_json::json!([{
        "t": 0,
        "T": 60000,
        "s": "BTC",
        "i": "1m",
        "o": "100",
        "h": "110",
        "l": "90",
        "c": "105",
        "v": "10",
        "n": 1
    }]);

    Mock::given(method("POST"))
        .and(path("/info"))
        .and(body_string_contains("candleSnapshot"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(server)
        .await;
}

pub async fn mock_hyperliquid_funding_history(server: &MockServer) {
    let response = serde_json::json!([{
        "coin": "BTC",
        "fundingRate": "0.0001",
        "time": 0
    }]);

    Mock::given(method("POST"))
        .and(path("/info"))
        .and(body_string_contains("fundingHistory"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(server)
        .await;
}

