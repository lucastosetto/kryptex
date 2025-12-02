//! Apalis worker setup for signal evaluation jobs

use crate::jobs::context::JobContext;
use crate::jobs::handlers;
use crate::jobs::types::{EvaluateSignalJob, FetchCandlesJob, StoreSignalJob};
use apalis::prelude::*;
use apalis_redis::RedisStorage;
use std::sync::Arc;
use tracing::info;

/// Configuration for the job runtime
#[derive(Clone)]
pub struct RuntimeConfig {
    pub evaluation_interval_seconds: u64,
    pub symbols: Vec<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            evaluation_interval_seconds: 60,
            symbols: vec!["BTC-PERP".to_string()],
        }
    }
}

/// Signal runtime that sets up Apalis workers
pub struct SignalRuntime {
    _config: RuntimeConfig,
    job_context: Arc<JobContext>,
    fetch_storage: Arc<RedisStorage<FetchCandlesJob>>,
    eval_storage: Arc<RedisStorage<EvaluateSignalJob>>,
    store_storage: Arc<RedisStorage<StoreSignalJob>>,
    concurrency: usize,
}

impl SignalRuntime {
    /// Create a new runtime with job context and storage backends
    pub fn new(
        config: RuntimeConfig,
        job_context: Arc<JobContext>,
        fetch_storage: Arc<RedisStorage<FetchCandlesJob>>,
        eval_storage: Arc<RedisStorage<EvaluateSignalJob>>,
        store_storage: Arc<RedisStorage<StoreSignalJob>>,
    ) -> Self {
        let concurrency = config.symbols.len().max(1);
        Self {
            _config: config,
            job_context,
            fetch_storage,
            eval_storage,
            store_storage,
            concurrency,
        }
    }

    /// Set custom concurrency (default is number of symbols)
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Start all workers and return handles for graceful shutdown
    pub async fn start_workers(
        &self,
    ) -> Result<Vec<tokio::task::JoinHandle<()>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut handles = Vec::new();

        info!(
            concurrency = self.concurrency,
            "SignalRuntime: starting Apalis workers with concurrency {}",
            self.concurrency
        );

        // Worker for FetchCandlesJob
        let fetch_storage = (*self.fetch_storage).clone();
        let eval_storage = self.eval_storage.clone();
        let job_context = self.job_context.clone();
        let fetch_handle = tokio::spawn(async move {
            let worker = WorkerBuilder::new("fetch-candles-worker")
                .data(job_context.clone())
                .data(eval_storage.clone())
                .backend(fetch_storage)
                .build_fn(handlers::handle_fetch_candles);

            info!("SignalRuntime: FetchCandlesJob worker started");
            worker.run().await;
        });
        handles.push(fetch_handle);

        // Worker for EvaluateSignalJob
        let eval_storage_worker = (*self.eval_storage).clone();
        let store_storage = self.store_storage.clone();
        let job_context_eval = self.job_context.clone();
        let eval_handle = tokio::spawn(async move {
            let worker = WorkerBuilder::new("evaluate-signal-worker")
                .data(job_context_eval.clone())
                .data(store_storage.clone())
                .backend(eval_storage_worker)
                .build_fn(handlers::handle_evaluate_signal);

            info!("SignalRuntime: EvaluateSignalJob worker started");
            worker.run().await;
        });
        handles.push(eval_handle);

        // Worker for StoreSignalJob
        let store_storage_worker = (*self.store_storage).clone();
        let job_context_store = self.job_context.clone();
        let store_handle = tokio::spawn(async move {
            let worker = WorkerBuilder::new("store-signal-worker")
                .data(job_context_store.clone())
                .backend(store_storage_worker)
                .build_fn(handlers::handle_store_signal);

            info!("SignalRuntime: StoreSignalJob worker started");
            worker.run().await;
        });
        handles.push(store_handle);

        info!("SignalRuntime: all workers started");
        Ok(handles)
    }
}
