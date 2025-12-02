//! Test utilities for API server integration tests

use axum_test::TestServer;
use perptrix::core::http::{create_router, AppState, HealthStatus};
use perptrix::metrics::Metrics;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Test helper for API server integration tests
#[allow(dead_code)]
pub struct TestApiServer {
    pub server: TestServer,
    pub metrics: Arc<Metrics>,
}

impl TestApiServer {
    pub async fn new() -> Self {
        let metrics = Arc::new(Metrics::new().expect("metrics initialization"));
        let state = AppState {
            health: Arc::new(RwLock::new(HealthStatus::default())),
            metrics: metrics.clone(),
            start_time: Arc::new(Instant::now()),
        };

        let app = create_router(state);
        let server = TestServer::new(app).expect("start test server");

        Self { server, metrics }
    }
}

