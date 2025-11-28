//! HTTP endpoint server using Axum

use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub health: Arc<RwLock<HealthStatus>>,
}

#[derive(Clone, Debug)]
pub struct HealthStatus {
    pub status: String,
    pub uptime_seconds: u64,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
            uptime_seconds: 0,
        }
    }
}

pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let health = state.health.read().await;
    Ok(Json(json!({
        "status": health.status,
        "uptime_seconds": health.uptime_seconds,
        "service": "perptrix-signal-engine"
    })))
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(state)
}

pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        health: Arc::new(RwLock::new(HealthStatus::default())),
    };
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!(port = port, "HTTP server listening on port {}", port);
    axum::serve(listener, app).await?;

    Ok(())
}
