//! Perptrix Signal Engine Server
//!
//! Starts the HTTP server with health check endpoint and optionally
//! runs periodic signal evaluation.

use dotenvy::dotenv;
use perptrix::cache::RedisCache;
use perptrix::core::http::start_server;
use perptrix::core::runtime::{RuntimeConfig, SignalRuntime};
use perptrix::db::QuestDatabase;
use perptrix::logging;
use perptrix::metrics::Metrics;
use perptrix::services::hyperliquid::HyperliquidMarketDataProvider;
use perptrix::services::market_data::MarketDataProvider;
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
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let eval_interval: u64 = env::var("EVAL_INTERVAL_SECONDS")
        .ok()
        .and_then(|i| i.parse().ok())
        .unwrap_or(0);

    let symbols: Option<Vec<String>> = env::var("SYMBOLS").ok().and_then(|s| {
        let v: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    });

    let env = perptrix::config::get_environment();
    info!("Starting Perptrix Signal Engine Server");
    info!(environment = %env, "Environment");
    info!(port = port, "HTTP Server: http://0.0.0.0:{}", port);

    if eval_interval > 0 {
        let symbols = symbols
            .ok_or("SYMBOLS environment variable is required when EVAL_INTERVAL_SECONDS > 0")?;
        info!(
            interval = eval_interval,
            "Signal Evaluation: every {} seconds", eval_interval
        );
        info!(symbols = ?symbols, "Symbols: {}", symbols.join(", "));

        let server_handle = tokio::spawn(async move {
            if let Err(e) = start_server(port).await {
                error!(error = %e, "HTTP server error");
            }
        });

        let runtime_config = RuntimeConfig {
            evaluation_interval_seconds: eval_interval,
            symbols: symbols.clone(),
        };

        // Initialize metrics
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
                warn!("Continuing without database - candles will only be stored in memory");
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

        // Initialize Hyperliquid provider
        info!("Initializing Hyperliquid WebSocket provider...");
        let mut provider = HyperliquidMarketDataProvider::new();

        if let Some(ref db) = database {
            provider = provider.with_database(db.clone());
        }
        if let Some(ref c) = cache {
            provider = provider.with_cache(c.clone());
        }

        // Wait for connection to establish (with timeout)
        info!("Waiting for WebSocket connection...");
        let client = provider.client();
        if client.wait_for_connection(Duration::from_secs(10)).await {
            info!("WebSocket connected");
            metrics.websocket_connected.set(1.0);
        } else {
            warn!("WebSocket connection timeout, subscriptions will be queued");
            metrics.websocket_connected.set(0.0);
        }

        // Subscribe to symbols (will queue if not connected yet)
        // This will also fetch historical candles if database is available
        for symbol in &symbols {
            match provider.subscribe(symbol).await {
                Ok(()) => {
                    info!(symbol = %symbol, "Subscribed to {} (or queued if not connected)", symbol);
                }
                Err(e) => {
                    error!(symbol = %symbol, error = %e, "Failed to subscribe to {}", symbol);
                }
            }
        }

        let mut runtime = SignalRuntime::with_provider(runtime_config, provider);
        if let Some(ref db) = database {
            runtime = runtime.with_database(db.clone());
        }
        runtime = runtime.with_metrics(metrics.clone());

        let runtime_handle = tokio::spawn(async move {
            if let Err(e) = runtime.run().await {
                error!(error = %e, "Signal runtime error");
            }
        });

        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Shutting down...");
            }
            _ = server_handle => {
                error!("HTTP server stopped");
            }
            _ = runtime_handle => {
                error!("Signal runtime stopped");
            }
        }
    } else {
        info!("Signal Evaluation: disabled (set EVAL_INTERVAL_SECONDS to enable)");

        let server_handle = tokio::spawn(async move {
            if let Err(e) = start_server(port).await {
                error!(error = %e, "HTTP server error");
            }
        });

        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Shutting down...");
            }
            _ = server_handle => {
                error!("HTTP server stopped");
            }
        }
    }

    Ok(())
}
