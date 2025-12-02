//! Perptrix Worker
//!
//! Processes signal evaluation jobs from the Redis queue.
//! Can be run as a separate process/instance from the web server.

use dotenvy::dotenv;
use perptrix::cache::RedisCache;
use perptrix::core::runtime::{RuntimeConfig, SignalRuntime};
use perptrix::core::scheduler::JobScheduler;
use perptrix::db::QuestDatabase;
use perptrix::jobs::context::JobContext;
use perptrix::jobs::types::{EvaluateSignalJob, FetchCandlesJob, StoreSignalJob};
use perptrix::logging;
use perptrix::metrics::Metrics;
use perptrix::services::hyperliquid::HyperliquidMarketDataProvider;
use perptrix::services::market_data::MarketDataProvider;
use apalis_redis::RedisStorage;
use std::env;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env if present
    dotenv().ok();

    // Initialize logging based on environment
    logging::init_logging();

    let eval_interval: u64 = env::var("EVAL_INTERVAL_SECONDS")
        .ok()
        .and_then(|i| i.parse().ok())
        .unwrap_or(0);

    let env = perptrix::config::get_environment();
    info!("Starting Perptrix Worker");
    info!(environment = %env, "Environment");

    if eval_interval == 0 {
        return Err("EVAL_INTERVAL_SECONDS must be > 0 for worker".into());
    }

    // Initialize metrics
    let metrics = Arc::new(Metrics::new()?);

    // Initialize QuestDB (required for loading strategies)
    info!("Initializing QuestDB connection...");
    let database = match QuestDatabase::new().await {
        Ok(db) => {
            info!("QuestDB connected");
            metrics.database_connected.set(1.0);
            Some(Arc::new(db))
        }
        Err(e) => {
            warn!(error = %e, "Failed to connect to QuestDB");
            warn!("Worker requires QuestDB for loading strategies - exiting");
            return Err(format!("QuestDB connection required for worker: {}", e).into());
        }
    };
    
    let db = database.as_ref().unwrap();
    
    // Load strategies from database
    info!("Loading strategies from database...");
    let strategies = db.get_strategies(None).await.map_err(|e| {
        format!("Failed to load strategies: {}", e)
    })?;
    
    if strategies.is_empty() {
        warn!("No strategies found in database - worker will start but no evaluation jobs will be scheduled");
        warn!("Use the API to create strategies: POST /api/strategies");
    } else {
        info!(strategy_count = strategies.len(), "Loaded {} strategies from database", strategies.len());
    }
    
    // Extract unique symbols from strategies
    let mut symbols: Vec<String> = strategies.iter().map(|s| s.symbol.clone()).collect();
    symbols.sort();
    symbols.dedup();
    
    let concurrency: usize = env::var("WORKER_CONCURRENCY")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or_else(|| symbols.len().max(1));
    
    info!(concurrency = concurrency, "Worker concurrency: {}", concurrency);
    info!(
        interval = eval_interval,
        "Signal Evaluation: every {} seconds", eval_interval
    );
    if symbols.is_empty() {
        warn!("No symbols to evaluate - no strategies configured");
    } else {
        info!(symbols = ?symbols, "Symbols from strategies: {}", symbols.join(", "));
    }

    let runtime_config = RuntimeConfig {
        evaluation_interval_seconds: eval_interval,
        symbols: symbols.clone(), // Used for scheduling - actual evaluation uses strategies from DB
    };

    // Initialize Redis cache (for reading candles)
    info!("Initializing Redis connection...");
    let cache = match RedisCache::new().await {
        Ok(c) => {
            info!("Redis connected");
            metrics.cache_connected.set(1.0);
            Some(Arc::new(c))
        }
        Err(e) => {
            warn!(error = %e, "Failed to connect to Redis");
            warn!("Worker requires Redis - exiting");
            return Err(format!("Redis connection required for worker: {}", e).into());
        }
    };

    // Create read-only data provider for jobs
    // This reads from Redis/QuestDB cache (WebSocket service writes to it)
    info!("Initializing read-only market data provider...");
    let mut read_only_provider = HyperliquidMarketDataProvider::new();
    if let Some(ref db) = database {
        read_only_provider = read_only_provider.with_database(db.clone());
    }
    if let Some(ref c) = cache {
        read_only_provider = read_only_provider.with_cache(c.clone());
    }
    let read_only_provider: Arc<dyn MarketDataProvider + Send + Sync> =
        Arc::new(read_only_provider);

    // Initialize Apalis storage backends
    info!("Initializing Apalis Redis storage...");
    let redis_url = perptrix::config::get_redis_url();
    let conn = apalis_redis::connect(redis_url.clone()).await?;
    let fetch_storage: Arc<RedisStorage<FetchCandlesJob>> =
        Arc::new(RedisStorage::new(conn.clone()));
    let eval_storage: Arc<RedisStorage<EvaluateSignalJob>> =
        Arc::new(RedisStorage::new(conn.clone()));
    let store_storage: Arc<RedisStorage<StoreSignalJob>> =
        Arc::new(RedisStorage::new(conn));
    info!("Apalis Redis storage initialized");

    // Create job context
    let job_context = Arc::new(JobContext::new(
        read_only_provider,
        database.clone(),
        Some(metrics.clone()),
    ));

    // Initialize and start job runtime (workers)
    info!("Starting Apalis workers...");
    let runtime = SignalRuntime::new(
        runtime_config.clone(),
        job_context,
        fetch_storage.clone(),
        eval_storage.clone(),
        store_storage.clone(),
    )
    .with_concurrency(concurrency);
    let worker_handles = runtime.start_workers().await.map_err(|e| format!("Failed to start workers: {}", e))?;

    // Initialize and start scheduler
    info!("Starting job scheduler...");
    let scheduler = JobScheduler::new(fetch_storage, symbols.clone(), eval_interval)
        .map_err(|e| format!("Failed to create scheduler: {}", e))?;
    scheduler.start().await.map_err(|e| format!("Failed to start scheduler: {}", e))?;

    // Graceful shutdown
    info!("Worker started, waiting for shutdown signal...");
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Shutting down worker...");
            scheduler.stop().await;
            for handle in worker_handles {
                handle.abort();
            }
            info!("Worker stopped");
        }
    }

    Ok(())
}

