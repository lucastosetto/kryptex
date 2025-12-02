//! Unit tests for HTTP server

use axum::extract::State;
use perptrix::core::http::{health_check, AppState, HealthStatus};
use perptrix::metrics::Metrics;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_health_check() {
    let state = AppState {
        health: Arc::new(RwLock::new(HealthStatus::default())),
        metrics: Arc::new(Metrics::default()),
        start_time: Arc::new(Instant::now()),
        database: None,
    };
    let result = health_check(State(state)).await;
    assert!(result.is_ok());
}



