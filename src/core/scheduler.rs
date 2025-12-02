//! Cron-based scheduler for enqueuing signal evaluation jobs

use crate::jobs::types::FetchCandlesJob;
use apalis::prelude::*;
use apalis_redis::RedisStorage;
use cron::Schedule;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Scheduler that periodically enqueues FetchCandlesJob for each symbol
pub struct JobScheduler {
    storage: Arc<RedisStorage<FetchCandlesJob>>,
    symbols: Vec<String>,
    schedule: Schedule,
    handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl JobScheduler {
    /// Create a new scheduler
    /// 
    /// # Arguments
    /// * `storage` - Redis storage backend for jobs
    /// * `symbols` - List of symbols to evaluate
    /// * `interval_seconds` - Evaluation interval in seconds (0 = disabled)
    pub fn new(
        storage: Arc<RedisStorage<FetchCandlesJob>>,
        symbols: Vec<String>,
        interval_seconds: u64,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if interval_seconds == 0 {
            return Err("Scheduler disabled: interval_seconds is 0".into());
        }

        // Convert interval to cron expression: every N seconds
        // Cron format: second minute hour day month weekday
        let cron_expr = if interval_seconds >= 60 {
            // For intervals >= 60 seconds, use minute-based cron
            let minutes = interval_seconds / 60;
            format!("0 */{} * * * *", minutes)
        } else {
            // For intervals < 60 seconds, use second-based cron
            format!("*/{} * * * * *", interval_seconds)
        };

        let schedule = Schedule::from_str(&cron_expr).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid cron expression '{}': {}", cron_expr, e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        info!(
            interval = interval_seconds,
            cron = %cron_expr,
            symbols = ?symbols,
            "JobScheduler: created with interval {}s (cron: {})",
            interval_seconds,
            cron_expr
        );

        Ok(Self {
            storage,
            symbols,
            schedule,
            handle: Arc::new(RwLock::new(None)),
        })
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let storage = self.storage.clone();
        let symbols = self.symbols.clone();
        let schedule = self.schedule.clone();
        let handle_arc = self.handle.clone();

        let handle = tokio::spawn(async move {
            info!("JobScheduler: started, waiting for cron schedule...");

            loop {
                // Get the next scheduled time
                let mut upcoming = schedule.upcoming(chrono::Utc);
                if let Some(next_tick) = upcoming.next() {
                    let now = chrono::Utc::now();
                    if next_tick > now {
                        let duration = (next_tick - now).to_std().unwrap_or_default();
                        tokio::time::sleep(duration).await;
                    }
                } else {
                    // No more scheduled times, wait a bit and check again
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    continue;
                }

                info!(
                    symbol_count = symbols.len(),
                    "JobScheduler: cron tick, enqueuing FetchCandlesJob for {} symbols",
                    symbols.len()
                );

                for symbol in &symbols {
                    let job = FetchCandlesJob {
                        symbol: symbol.clone(),
                    };

                    let mut storage_clone = (*storage).clone();
                    match storage_clone.push(job).await {
                        Ok(_) => {
                            debug!(symbol = %symbol, "JobScheduler: enqueued FetchCandlesJob for {}", symbol);
                        }
                        Err(e) => {
                            error!(
                                symbol = %symbol,
                                error = %e,
                                "JobScheduler: failed to enqueue FetchCandlesJob for {}",
                                symbol
                            );
                        }
                    }
                }
            }
        });

        {
            let mut h = handle_arc.write().await;
            *h = Some(handle);
        }

        info!("JobScheduler: started successfully");
        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) {
        let mut handle = self.handle.write().await;
        if let Some(h) = handle.take() {
            h.abort();
            info!("JobScheduler: stopped");
        }
    }

    /// Check if the scheduler is running
    pub async fn is_running(&self) -> bool {
        let handle = self.handle.read().await;
        handle.is_some()
    }
}
