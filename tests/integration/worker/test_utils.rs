//! Test utilities for worker integration tests

use apalis_redis::RedisStorage;
use chrono::Utc;
use perptrix::cache::RedisCache;
use perptrix::db::QuestDatabase;
use perptrix::jobs::context::JobContext;
use perptrix::jobs::types::{EvaluateSignalJob, FetchCandlesJob, StoreSignalJob};
use perptrix::metrics::Metrics;
use perptrix::models::indicators::Candle;
use perptrix::services::hyperliquid::{
    HyperliquidMarketDataProvider, HyperliquidRestClient, MockWebSocketClient,
};
use perptrix::services::market_data::MarketDataProvider;
use std::sync::Arc;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test helper for worker integration tests
#[allow(dead_code)]
pub struct TestWorker {
    pub fetch_storage: Arc<RedisStorage<FetchCandlesJob>>,
    pub eval_storage: Arc<RedisStorage<EvaluateSignalJob>>,
    pub store_storage: Arc<RedisStorage<StoreSignalJob>>,
    pub job_context: Arc<JobContext>,
    pub websocket: Arc<MockWebSocketClient>,
    pub hyperliquid_rest: MockServer,
}

impl TestWorker {
    pub async fn new() -> Self {
        // Setup Redis storage (using test Redis or in-memory)
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());
        
        let conn = apalis_redis::connect(redis_url.clone())
            .await
            .expect("Should connect to Redis");
        
        let fetch_storage = Arc::new(RedisStorage::new(conn.clone()));
        let eval_storage = Arc::new(RedisStorage::new(conn.clone()));
        let store_storage = Arc::new(RedisStorage::new(conn));

        // Setup mocked dependencies
        let mock_server = MockServer::start().await;
        mock_hyperliquid_candles(&mock_server).await;
        mock_hyperliquid_funding_history(&mock_server).await;

        let websocket = Arc::new(MockWebSocketClient::new());
        let rest_client = Arc::new(HyperliquidRestClient::with_client(
            mock_server.uri(),
            reqwest::Client::new(),
        ));

        // Create read-only provider (workers don't create connections)
        let read_only_provider: Arc<dyn MarketDataProvider + Send + Sync> =
            Arc::new(HyperliquidMarketDataProvider::with_clients(
                websocket.clone(),
                rest_client,
                vec!["1m".to_string()],
            ));

        // Setup optional dependencies
        let database = match QuestDatabase::new().await {
            Ok(db) => Some(Arc::new(db)),
            Err(_) => None, // Database optional for tests
        };

        let _cache = match RedisCache::new().await {
            Ok(c) => Some(Arc::new(c)),
            Err(_) => None, // Cache optional for tests
        };

        let metrics = Arc::new(Metrics::new().expect("Should create metrics"));

        let job_context = Arc::new(JobContext::new(
            read_only_provider,
            database,
            Some(metrics),
        ));

        Self {
            fetch_storage,
            eval_storage,
            store_storage,
            job_context,
            websocket,
            hyperliquid_rest: mock_server,
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

/// Create test candles for testing
pub fn create_test_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let base = 100.0 + (i as f64 * 0.5);
        let candle = Candle::new(
            base,
            base + 0.3,
            base - 0.2,
            base + 0.1,
            1000.0 + (i as f64 * 10.0),
            Utc::now(),
        )
        .with_open_interest(10_000.0 + (i as f64 * 50.0))
        .with_funding_rate(0.0002);
        candles.push(candle);
    }
    candles
}

