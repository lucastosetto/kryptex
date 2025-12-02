//! Perptrix WebSocket Service
//!
//! Maintains long-lived WebSocket connection to market data provider.
//! Receives real-time updates and stores them in Redis/QuestDB.
//! This service should run as a singleton (one instance).

use dotenvy::dotenv;
use perptrix::cache::RedisCache;
use perptrix::db::QuestDatabase;
use perptrix::logging;
use perptrix::metrics::Metrics;
use perptrix::services::hyperliquid::HyperliquidMarketDataProvider;
use perptrix::services::websocket::WebSocketService;
use std::env;
use std::sync::Arc;
use tokio::signal;
use tokio::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env if present
    dotenv().ok();

    // Initialize logging based on environment
    logging::init_logging();

    let symbols: Option<Vec<String>> = env::var("SYMBOLS").ok().and_then(|s| {
        let v: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    });

    let env = perptrix::config::get_environment();
    info!("Starting Perptrix WebSocket Service");
    info!(environment = %env, "Environment");
    info!("This service maintains the WebSocket connection to the market data provider");

    // Initialize metrics (for monitoring WebSocket health)
    let metrics = Arc::new(Metrics::new()?);

    // Initialize QuestDB
    info!("Initializing QuestDB connection...");
    let database = match QuestDatabase::new().await {
        Ok(db) => {
            info!("QuestDB connected");
            metrics.database_connected.set(1.0);
            Some(Arc::new(db))
        }
        Err(e) => {
            warn!(error = %e, "Failed to connect to QuestDB");
            warn!("Continuing without database - candles will only be stored in memory/Redis");
            metrics.database_connected.set(0.0);
            None
        }
    };

    // Initialize Redis cache
    info!("Initializing Redis connection...");
    let cache = match RedisCache::new().await {
        Ok(c) => {
            info!("Redis connected");
            metrics.cache_connected.set(1.0);
            Some(Arc::new(c))
        }
        Err(e) => {
            warn!(error = %e, "Failed to connect to Redis");
            warn!("Continuing without cache - will use database/memory only");
            metrics.cache_connected.set(0.0);
            None
        }
    };

    // Initialize WebSocket Service (long-lived, maintains connection)
    info!("Initializing WebSocket service...");
    let mut ws_provider = HyperliquidMarketDataProvider::new();
    if let Some(ref db) = database {
        ws_provider = ws_provider.with_database(db.clone());
    }
    if let Some(ref c) = cache {
        ws_provider = ws_provider.with_cache(c.clone());
    }

    let ws_service = WebSocketService::new(ws_provider);
    ws_service.start().await.map_err(|e| format!("Failed to start WebSocket service: {}", e))?;

    // Wait for connection to establish (with timeout)
    info!("Waiting for WebSocket connection...");
    let ws_client = ws_service.get_provider().client();
    if ws_client.wait_for_connection(Duration::from_secs(10)).await {
        info!("WebSocket connected");
        metrics.websocket_connected.set(1.0);
    } else {
        warn!("WebSocket connection timeout, subscriptions will be queued");
        metrics.websocket_connected.set(0.0);
    }

    // Subscribe to symbols if provided (will queue if not connected yet)
    if let Some(symbols) = symbols {
        info!(symbols = ?symbols, "Subscribing to symbols: {}", symbols.join(", "));
        for symbol in &symbols {
            match ws_service.subscribe(symbol).await {
                Ok(()) => {
                    info!(symbol = %symbol, "Subscribed to {} (or queued if not connected)", symbol);
                }
                Err(e) => {
                    error!(symbol = %symbol, error = %e, "Failed to subscribe to {}", symbol);
                }
            }
        }
    } else {
        info!("No symbols specified - WebSocket service running but not subscribed to any symbols");
        info!("Note: Use SYMBOLS environment variable to subscribe to symbols");
    }

    // Graceful shutdown
    info!("WebSocket service started and running. Waiting for shutdown signal...");
    info!("Note: This service should run as a singleton (one instance)");
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Shutting down WebSocket service...");
            ws_service.stop().await;
            info!("WebSocket service stopped");
        }
    }

    Ok(())
}

