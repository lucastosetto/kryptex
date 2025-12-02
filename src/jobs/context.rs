//! Job context for dependency injection

use crate::db::QuestDatabase;
use crate::metrics::Metrics;
use crate::services::market_data::MarketDataProvider;
use std::sync::Arc;

/// Context passed to job handlers via Apalis Data<T> pattern
/// 
/// Contains read-only access to:
/// - Market data provider (reads from Redis/QuestDB cache)
/// - Database (for storing signals)
/// - Metrics (for tracking evaluation statistics)
/// 
/// Note: WebSocket service is NOT included - jobs never create connections,
/// they only read from stored data.
pub struct JobContext {
    pub data_provider: Arc<dyn MarketDataProvider + Send + Sync>,
    pub database: Option<Arc<QuestDatabase>>,
    pub metrics: Option<Arc<Metrics>>,
}

impl JobContext {
    pub fn new(
        data_provider: Arc<dyn MarketDataProvider + Send + Sync>,
        database: Option<Arc<QuestDatabase>>,
        metrics: Option<Arc<Metrics>>,
    ) -> Self {
        Self {
            data_provider,
            database,
            metrics,
        }
    }
}




