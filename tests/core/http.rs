//! Unit tests for HTTP server

use perptrix::core::http::{health_check, AppState, HealthStatus};
use axum::extract::State;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_health_check() {
    let state = AppState {
        health: Arc::new(RwLock::new(HealthStatus::default())),
    };
    let result = health_check(State(state)).await;
    assert!(result.is_ok());
}

