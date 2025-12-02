//! HTTP endpoint server using Axum

use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{error, info, Level};

use crate::db::QuestDatabase;
use crate::metrics::Metrics;
use crate::models::strategy::{Strategy, StrategyConfig};

#[derive(Clone)]
pub struct AppState {
    pub health: Arc<RwLock<HealthStatus>>,
    pub metrics: Arc<Metrics>,
    pub start_time: Arc<Instant>,
    pub database: Option<Arc<QuestDatabase>>,
}

#[derive(Clone, Debug)]
pub struct HealthStatus {
    pub status: String,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
        }
    }
}

pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let health = state.health.read().await;
    let uptime_seconds = state.start_time.elapsed().as_secs();
    Ok(Json(json!({
        "status": health.status,
        "uptime_seconds": uptime_seconds,
        "service": "perptrix-signal-engine"
    })))
}

pub async fn metrics_handler(State(state): State<AppState>) -> Result<String, StatusCode> {
    state
        .metrics
        .export()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Middleware to track HTTP request metrics
async fn metrics_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // Increment in-flight requests
    state.metrics.http_requests_in_flight.inc();

    // Process request
    let response = next.run(request).await;
    let status = response.status();
    let duration = start.elapsed();

    // Decrement in-flight requests
    state.metrics.http_requests_in_flight.dec();

    // Record metrics
    state.metrics.http_requests_total.inc();
    state
        .metrics
        .http_request_duration_seconds
        .observe(duration.as_secs_f64());

    // Log if error status
    if status.is_server_error() {
        tracing::error!(
            method = %method,
            path = %path,
            status = %status,
            duration_ms = duration.as_millis(),
            "HTTP request error"
        );
    }

    response
}

#[derive(Debug, Deserialize)]
struct StrategyQuery {
    symbol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateStrategyRequest {
    name: String,
    symbol: String,
    config: StrategyConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateStrategyRequest {
    name: Option<String>,
    symbol: Option<String>,
    config: Option<StrategyConfig>,
}

#[derive(Debug, Serialize)]
struct StrategyResponse {
    id: i64,
    name: String,
    symbol: String,
    config: StrategyConfig,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Strategy> for StrategyResponse {
    fn from(strategy: Strategy) -> Self {
        Self {
            id: strategy.id.unwrap_or(0),
            name: strategy.name,
            symbol: strategy.symbol,
            config: strategy.config,
            created_at: strategy.created_at,
            updated_at: strategy.updated_at,
        }
    }
}

/// List all strategies, optionally filtered by symbol
async fn list_strategies(
    State(state): State<AppState>,
    Query(params): Query<StrategyQuery>,
) -> Result<Json<Value>, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let strategies = db
        .get_strategies(params.symbol.as_deref())
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to load strategies");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let responses: Vec<StrategyResponse> = strategies.into_iter().map(Into::into).collect();
    Ok(Json(json!(responses)))
}

/// Get a strategy by ID
async fn get_strategy(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<StrategyResponse>, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let strategy = db.get_strategy(id).await.map_err(|e| {
        error!(error = %e, strategy_id = id, "Failed to load strategy");
        if e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(strategy.into()))
}

/// Create a new strategy
async fn create_strategy(
    State(state): State<AppState>,
    Json(request): Json<CreateStrategyRequest>,
) -> Result<Json<StrategyResponse>, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let now = chrono::Utc::now();
    let strategy = Strategy {
        id: None,
        name: request.name,
        symbol: request.symbol,
        config: request.config,
        created_at: now,
        updated_at: now,
    };

    let id = db.create_strategy(&strategy).await.map_err(|e| {
        error!(error = %e, "Failed to create strategy");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let created_strategy = db.get_strategy(id).await.map_err(|e| {
        error!(error = %e, strategy_id = id, "Failed to load created strategy");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(created_strategy.into()))
}

/// Update a strategy
async fn update_strategy(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateStrategyRequest>,
) -> Result<Json<StrategyResponse>, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let mut strategy = db.get_strategy(id).await.map_err(|e| {
        error!(error = %e, strategy_id = id, "Failed to load strategy");
        if e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // Update fields if provided
    if let Some(name) = request.name {
        strategy.name = name;
    }
    if let Some(symbol) = request.symbol {
        strategy.symbol = symbol;
    }
    if let Some(config) = request.config {
        strategy.config = config;
    }
    strategy.updated_at = chrono::Utc::now();

    db.update_strategy(id, &strategy).await.map_err(|e| {
        error!(error = %e, strategy_id = id, "Failed to update strategy");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(strategy.into()))
}

/// Delete a strategy
async fn delete_strategy(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    db.delete_strategy(id).await.map_err(|e| {
        error!(error = %e, strategy_id = id, "Failed to delete strategy");
        if e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(StatusCode::NO_CONTENT)
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/api/strategies", get(list_strategies))
        .route("/api/strategies", post(create_strategy))
        .route("/api/strategies/{id}", get(get_strategy))
        .route("/api/strategies/{id}", put(update_strategy))
        .route("/api/strategies/{id}", delete(delete_strategy))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
                        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
                        .on_response(DefaultOnResponse::new().level(Level::DEBUG)),
                )
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    metrics_middleware,
                ))
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let metrics = Arc::new(Metrics::new()?);
    let start_time = Arc::new(Instant::now());
    
    // Initialize database connection (optional - API works without it but strategy endpoints won't)
    let database = match crate::db::QuestDatabase::new().await {
        Ok(db) => {
            info!("QuestDB connected for API server");
            Some(Arc::new(db))
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to connect to QuestDB for API server - strategy endpoints will be unavailable");
            None
        }
    };
    
    let state = AppState {
        health: Arc::new(RwLock::new(HealthStatus::default())),
        metrics: metrics.clone(),
        start_time: start_time.clone(),
        database,
    };
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!(port = port, "HTTP server listening on port {}", port);
    info!(
        "Metrics endpoint available at http://0.0.0.0:{}/metrics",
        port
    );
    axum::serve(listener, app).await?;

    Ok(())
}
