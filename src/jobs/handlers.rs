//! Job handlers for signal evaluation workflow

use crate::jobs::context::JobContext;
use crate::jobs::types::{EvaluateSignalJob, FetchCandlesJob, StoreSignalJob};
use crate::signals::engine::MIN_CANDLES;
use apalis::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info};

/// Handler for fetching candles job
/// 
/// Reads candles from the data provider (which reads from Redis/QuestDB cache).
/// If candles are available, enqueues EvaluateSignalJob.
pub async fn handle_fetch_candles(
    job: FetchCandlesJob,
    ctx: Data<Arc<JobContext>>,
    eval_storage: Data<apalis_redis::RedisStorage<EvaluateSignalJob>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!(symbol = %job.symbol, "FetchCandlesJob: fetching candles for {}", job.symbol);

    let candles = ctx
        .data_provider
        .get_candles(&job.symbol, 250)
        .await
        .map_err(|e| {
            Box::new(std::io::Error::other(format!(
                "Market data error: {}",
                e
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

    if candles.is_empty() {
        debug!(symbol = %job.symbol, "FetchCandlesJob: no candles available yet for {}", job.symbol);
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No candles available for {}", job.symbol),
        )) as Box<dyn std::error::Error + Send + Sync>);
    }

    debug!(
        symbol = %job.symbol,
        count = candles.len(),
        "FetchCandlesJob: fetched {} candles for {}",
        candles.len(),
        job.symbol
    );

    if candles.len() < MIN_CANDLES {
        debug!(
            symbol = %job.symbol,
            count = candles.len(),
            min = MIN_CANDLES,
            "FetchCandlesJob: not enough candles ({} < {}) for {}",
            candles.len(),
            MIN_CANDLES,
            job.symbol
        );
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Not enough candles: {} < {}",
                candles.len(),
                MIN_CANDLES
            ),
        )) as Box<dyn std::error::Error + Send + Sync>);
    }

    // Enqueue next job: EvaluateSignalJob
    let next_job = EvaluateSignalJob {
        symbol: job.symbol.clone(),
        candles,
    };
    let mut storage = (*eval_storage).clone();
    storage.push(next_job).await.map_err(|e| {
        Box::new(std::io::Error::other(format!(
            "Failed to enqueue EvaluateSignalJob: {}",
            e
        ))) as Box<dyn std::error::Error + Send + Sync>
    })?;

    debug!(symbol = %job.symbol, "FetchCandlesJob: enqueued EvaluateSignalJob for {}", job.symbol);
    Ok(())
}

/// Handler for evaluating signal job
/// 
/// Loads strategies for the symbol and evaluates each one.
/// If signals are generated, enqueues StoreSignalJob for each.
pub async fn handle_evaluate_signal(
    job: EvaluateSignalJob,
    ctx: Data<Arc<JobContext>>,
    store_storage: Data<apalis_redis::RedisStorage<StoreSignalJob>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!(
        symbol = %job.symbol,
        candle_count = job.candles.len(),
        "EvaluateSignalJob: evaluating signals for {} with {} candles",
        job.symbol,
        job.candles.len()
    );

    // Load strategies for this symbol
    let strategies = if let Some(ref db) = ctx.database {
        db.get_strategies(Some(&job.symbol)).await.map_err(|e| {
            Box::new(std::io::Error::other(format!(
                "Failed to load strategies: {}",
                e
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?
    } else {
        debug!(
            symbol = %job.symbol,
            "EvaluateSignalJob: no database available, skipping strategy evaluation"
        );
        return Ok(());
    };

    // If no strategies found, gracefully skip (no error)
    if strategies.is_empty() {
        debug!(
            symbol = %job.symbol,
            "EvaluateSignalJob: no strategies found for {}, skipping evaluation",
            job.symbol
        );
        return Ok(());
    }

    // Evaluate each strategy
    let mut signals_generated = 0;
    for strategy in &strategies {
        if let Some(signal) = crate::signals::engine::SignalEngine::evaluate(&job.candles, strategy) {
            let confidence_pct = (signal.confidence * 10000.0).round() / 100.0;
            info!(
                symbol = %job.symbol,
                strategy_id = strategy.id.unwrap_or(0),
                strategy_name = %strategy.name,
                direction = ?signal.direction,
                confidence = confidence_pct,
                "EvaluateSignalJob: signal generated for {} using strategy '{}' - Direction: {:?}, Confidence: {:.2}%",
                job.symbol,
                strategy.name,
                signal.direction,
                confidence_pct
            );

            // Enqueue next job: StoreSignalJob
            let next_job = StoreSignalJob {
                symbol: job.symbol.clone(),
                signal,
                strategy_id: strategy.id.unwrap_or(0),
            };
            let mut storage = (*store_storage).clone();
            storage.push(next_job).await.map_err(|e| {
                Box::new(std::io::Error::other(format!(
                    "Failed to enqueue StoreSignalJob: {}",
                    e
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            signals_generated += 1;
        } else {
            debug!(
                symbol = %job.symbol,
                strategy_id = strategy.id.unwrap_or(0),
                strategy_name = %strategy.name,
                "EvaluateSignalJob: no signal generated for {} using strategy '{}'",
                job.symbol,
                strategy.name
            );
        }
    }

    debug!(
        symbol = %job.symbol,
        strategies_evaluated = strategies.len(),
        signals_generated = signals_generated,
        "EvaluateSignalJob: evaluated {} strategies, generated {} signals for {}",
        strategies.len(),
        signals_generated,
        job.symbol
    );

    Ok(())
}

/// Handler for storing signal job
/// 
/// Stores the signal in the database and updates metrics.
/// This is the final step in the workflow.
pub async fn handle_store_signal(
    job: StoreSignalJob,
    ctx: Data<Arc<JobContext>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();
    let symbol = &job.symbol;

    // Track active evaluation
    if let Some(ref metrics) = ctx.metrics {
        metrics.signal_evaluations_active.inc();
    }

    // Log at different levels based on signal strength
    let confidence_pct = (job.signal.confidence * 10000.0).round() / 100.0;
    if job.signal.direction == crate::models::signal::SignalDirection::Neutral {
        debug!(
            symbol = %symbol,
            direction = ?job.signal.direction,
            confidence = confidence_pct,
            "StoreSignalJob: storing neutral signal for {} (confidence: {:.2}%)",
            symbol,
            confidence_pct
        );
    } else {
        info!(
            symbol = %symbol,
            direction = ?job.signal.direction,
            confidence = confidence_pct,
            "StoreSignalJob: storing signal for {}: {:?} (confidence: {:.2}%)",
            symbol,
            job.signal.direction,
            confidence_pct
        );
    }

    // Record successful evaluation
    if let Some(ref metrics) = ctx.metrics {
        metrics.signal_evaluations_total.inc();
    }

    // Store signal in database if available
    if let Some(ref db) = ctx.database {
        if let Err(e) = db.store_signal(&job.signal, job.strategy_id).await {
            error!(
                symbol = %symbol,
                strategy_id = job.strategy_id,
                error = %e,
                "StoreSignalJob: failed to store signal in database for {} (strategy_id: {})",
                symbol,
                job.strategy_id
            );
            // Still count as evaluation (storage failure is separate from evaluation success)
        } else {
            debug!(
                symbol = %symbol,
                strategy_id = job.strategy_id,
                "StoreSignalJob: stored signal in database for {} (strategy_id: {})",
                symbol,
                job.strategy_id
            );
        }
    }

    // Record duration and decrement active
    if let Some(ref metrics) = ctx.metrics {
        let duration = start.elapsed();
        metrics
            .signal_evaluation_duration_seconds
            .observe(duration.as_secs_f64());
        metrics.signal_evaluations_active.dec();
    }

    Ok(())
}

