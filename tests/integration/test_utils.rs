use std::sync::Arc;
use std::time::Instant;

use axum_test::TestServer;
use perptrix::core::http::{create_router, AppState, HealthStatus};
use perptrix::metrics::Metrics;
use perptrix::services::hyperliquid::{
    HyperliquidMarketDataProvider, HyperliquidRestClient, MockWebSocketClient,
};
use tokio::sync::RwLock;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper structure bundling together the HTTP server and mocked dependencies.
#[allow(dead_code)]
pub struct TestApp {
    pub server: TestServer,
    pub metrics: Arc<Metrics>,
    pub hyperliquid_rest: MockServer,
    pub websocket: Arc<MockWebSocketClient>,
    pub provider: HyperliquidMarketDataProvider,
}

impl TestApp {
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

        let metrics = Arc::new(Metrics::new().expect("metrics initialization"));
        let state = AppState {
            health: Arc::new(RwLock::new(HealthStatus::default())),
            metrics: metrics.clone(),
            start_time: Arc::new(Instant::now()),
            database: None,
        };

        let router = create_router(state);
        let server = TestServer::new(router).expect("start test server");

        Self {
            server,
            metrics,
            hyperliquid_rest: mock_server,
            websocket,
            provider,
        }
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

