//! Hyperliquid market data provider implementation

use crate::cache::RedisCache;
use crate::config;
use crate::db::QuestDatabase;
use crate::models::indicators::Candle;
use crate::services::market_data::MarketDataProvider;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde_json;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, warn};

use super::client::{ClientEvent, HyperliquidClient, WebSocketClient};
use super::messages::{CandleData, CandleUpdate, RequestMessage, Subscription, WebSocketMessage};
use super::rest::HyperliquidRestClient;
use super::subscriptions::{SubscriptionKey, SubscriptionManager};

pub struct HyperliquidMarketDataProvider {
    pub(crate) client: Arc<dyn WebSocketClient>,
    subscriptions: Arc<SubscriptionManager>,
    candles: Arc<RwLock<HashMap<String, VecDeque<Candle>>>>,
    latest_prices: Arc<RwLock<HashMap<String, f64>>>,
    candle_intervals: Vec<String>,
    pending_subscriptions: Arc<RwLock<Vec<(String, String)>>>, // (coin, interval)
    rest_client: Arc<HyperliquidRestClient>,
    database: Option<Arc<QuestDatabase>>,
    cache: Option<Arc<RedisCache>>,
    funding_cache: Arc<RwLock<HashMap<String, FundingCacheEntry>>>,
}

impl HyperliquidMarketDataProvider {
    pub fn new() -> Self {
        Self::with_intervals(vec![
            "1m".to_string(),
            "5m".to_string(),
            "15m".to_string(),
            "1h".to_string(),
        ])
    }

    pub fn with_intervals(candle_intervals: Vec<String>) -> Self {
        let websocket: Arc<dyn WebSocketClient> = Arc::new(HyperliquidClient::new());
        let rest_client = Arc::new(HyperliquidRestClient::new());
        Self::with_clients(websocket, rest_client, candle_intervals)
    }

    pub fn with_clients(
        websocket_client: Arc<dyn WebSocketClient>,
        rest_client: Arc<HyperliquidRestClient>,
        candle_intervals: Vec<String>,
    ) -> Self {
        let provider = Self {
            client: websocket_client,
            subscriptions: Arc::new(SubscriptionManager::new()),
            candles: Arc::new(RwLock::new(HashMap::new())),
            latest_prices: Arc::new(RwLock::new(HashMap::new())),
            candle_intervals: candle_intervals.clone(),
            pending_subscriptions: Arc::new(RwLock::new(Vec::new())),
            rest_client,
            database: None,
            cache: None,
            funding_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        provider.spawn_background_tasks();
        provider
    }

    fn clone_for_task(&self) -> TaskProvider {
        TaskProvider {
            client: self.client.clone(),
            subscriptions: self.subscriptions.clone(),
            candles: self.candles.clone(),
            latest_prices: self.latest_prices.clone(),
            pending_subscriptions: self.pending_subscriptions.clone(),
            candle_intervals: self.candle_intervals.clone(),
            database: self.database.clone(),
            cache: self.cache.clone(),
            rest_client: self.rest_client.clone(),
            funding_cache: self.funding_cache.clone(),
        }
    }

    fn spawn_background_tasks(&self) {
        let client_clone = self.client.clone();
        tokio::spawn(async move {
            let _ = client_clone.connect().await;
        });

        let provider_clone = self.clone_for_task();
        tokio::spawn(async move {
            provider_clone.handle_messages().await;
        });
    }

    async fn subscribe_candle(
        &self,
        coin: &str,
        interval: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Fetch historical candles (even if database is unavailable, we can use Redis and in-memory)
        let historical_count = config::get_historical_candle_count();
        debug!(coin = %coin, interval = %interval, count = historical_count, "Fetching {} historical candles for {}/{}", historical_count, coin, interval);

        match self
            .rest_client
            .fetch_historical_candles(coin, interval, historical_count)
            .await
        {
            Ok(mut historical_candles) => {
                debug!(coin = %coin, interval = %interval, count = historical_candles.len(), "Fetched {} historical candles for {}/{}", historical_candles.len(), coin, interval);

                self.attach_funding_rates(coin, &mut historical_candles)
                    .await;

                // Store in QuestDB if available
                if let Some(ref db) = self.database {
                    if let Err(e) = db
                        .store_candles_batch(coin, interval, &historical_candles)
                        .await
                    {
                        warn!(coin = %coin, interval = %interval, error = %e, "Failed to store historical candles in QuestDB");
                    } else {
                        debug!(coin = %coin, interval = %interval, count = historical_candles.len(), "Stored {} historical candles in QuestDB", historical_candles.len());
                    }
                }

                // Cache in Redis if available
                if let Some(ref cache) = self.cache {
                    if let Err(e) = cache
                        .cache_candles(coin, interval, &historical_candles)
                        .await
                    {
                        warn!(coin = %coin, interval = %interval, error = %e, "Failed to cache historical candles in Redis");
                    } else {
                        debug!(coin = %coin, interval = %interval, count = historical_candles.len(), "Cached {} historical candles in Redis", historical_candles.len());
                    }
                }

                // Always update in-memory buffer
                let symbol_key = format!("{}_{}", coin, interval);
                let mut candles_map = self.candles.write().await;
                let candles = candles_map
                    .entry(symbol_key.clone())
                    .or_insert_with(VecDeque::new);
                for candle in historical_candles {
                    candles.push_back(candle);
                }
                // Keep only last 1000 in memory
                while candles.len() > 1000 {
                    candles.pop_front();
                }
                debug!(symbol = %symbol_key, count = candles.len(), "Loaded {} historical candles into memory buffer", candles.len());
            }
            Err(e) => {
                warn!(coin = %coin, interval = %interval, error = %e, "Failed to fetch historical candles for {}/{}", coin, interval);
            }
        }

        // Add to pending subscriptions
        {
            let mut pending = self.pending_subscriptions.write().await;
            if !pending.contains(&(coin.to_string(), interval.to_string())) {
                pending.push((coin.to_string(), interval.to_string()));
            }
        }

        // Try to subscribe if connected, otherwise it will be done on reconnect
        if self.client.is_connected().await {
            self.subscribe_candle_internal(coin, interval).await
        } else {
            debug!(coin = %coin, interval = %interval, "Not connected yet, subscription queued for {}/{}", coin, interval);
            Ok(())
        }
    }

    async fn subscribe_candle_internal(
        &self,
        coin: &str,
        interval: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = SubscriptionKey::candle(coin, interval);

        if self.subscriptions.contains(&key).await {
            return Ok(()); // Already subscribed
        }

        let subscription = Subscription::candle(coin, interval);
        let request = RequestMessage::Subscribe { subscription };

        let json = serde_json::to_string(&request).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        debug!(subscription = %json, "Sending subscription");

        self.client.send_text(json).await.map_err(|e| {
            Box::new(std::io::Error::other(format!(
                "WebSocket send error: {}",
                e
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

        self.subscriptions.add(key).await;
        Ok(())
    }

    async fn attach_funding_rates(&self, coin: &str, candles: &mut [Candle]) {
        if candles.is_empty() {
            return;
        }

        let first_ts = candles
            .first()
            .map(|c| c.timestamp.timestamp_millis().max(0) as u64)
            .unwrap_or(0);
        let last_ts = candles
            .last()
            .map(|c| c.timestamp.timestamp_millis().max(0) as u64)
            .unwrap_or(first_ts);
        let buffer_ms = 2 * 60 * 60 * 1000; // + / - 2h buffer
        let start_time = first_ts.saturating_sub(buffer_ms);
        let end_time = last_ts.saturating_add(buffer_ms);

        match self
            .rest_client
            .fetch_funding_history(coin, Some(start_time), Some(end_time))
            .await
        {
            Ok(mut funding_points) => {
                if funding_points.is_empty() {
                    debug!(coin = %coin, "No funding history returned for {}", coin);
                    return;
                }
                funding_points.sort_by_key(|p| p.timestamp);

                let mut idx = 0usize;
                let mut current_rate: Option<f64> = None;
                for candle in candles.iter_mut() {
                    while idx < funding_points.len()
                        && funding_points[idx].timestamp <= candle.timestamp
                    {
                        current_rate = Some(funding_points[idx].funding_rate);
                        idx += 1;
                    }

                    if let Some(rate) = current_rate {
                        candle.funding_rate = Some(rate);
                    }
                }
            }
            Err(e) => {
                warn!(coin = %coin, error = %e, "Failed to fetch funding history for {}", coin);
            }
        }
    }

    fn get_primary_interval(&self) -> &str {
        self.candle_intervals
            .first()
            .map(|s| s.as_str())
            .unwrap_or("1m")
    }

    pub fn client(&self) -> Arc<dyn WebSocketClient> {
        self.client.clone()
    }

    pub fn with_database(mut self, database: Arc<QuestDatabase>) -> Self {
        self.database = Some(database);
        self
    }

    pub fn with_cache(mut self, cache: Arc<RedisCache>) -> Self {
        self.cache = Some(cache);
        self
    }
}

#[derive(Clone)]
struct TaskProvider {
    client: Arc<dyn WebSocketClient>,
    #[allow(dead_code)] // Kept for future resubscription functionality
    subscriptions: Arc<SubscriptionManager>,
    candles: Arc<RwLock<HashMap<String, VecDeque<Candle>>>>,
    latest_prices: Arc<RwLock<HashMap<String, f64>>>,
    pending_subscriptions: Arc<RwLock<Vec<(String, String)>>>,
    #[allow(dead_code)] // Used for resubscription
    candle_intervals: Vec<String>,
    database: Option<Arc<QuestDatabase>>,
    cache: Option<Arc<RedisCache>>,
    rest_client: Arc<HyperliquidRestClient>,
    funding_cache: Arc<RwLock<HashMap<String, FundingCacheEntry>>>,
}

impl TaskProvider {
    async fn handle_messages(&self) {
        loop {
            while let Some(event) = self.client.receive().await {
                match event {
                    ClientEvent::Message(text) => {
                        if let Err(e) = self.process_message(&text).await {
                            error!(error = %e, "Error processing message");
                        }
                    }
                    ClientEvent::Connected => {
                        debug!("WebSocket connected, resubscribing...");
                        // Wait a moment for connection to stabilize
                        sleep(Duration::from_millis(500)).await;
                        // Resubscribe to all pending subscriptions
                        let pending = self.pending_subscriptions.read().await.clone();
                        debug!(
                            count = pending.len(),
                            "Resubscribing to {} pending subscriptions",
                            pending.len()
                        );
                        for (coin, interval) in pending {
                            if let Err(e) = self.subscribe_candle_internal(&coin, &interval).await {
                                debug!(coin = %coin, interval = %interval, error = %e, "Failed to resubscribe to {} {}", coin, interval);
                            } else {
                                debug!(coin = %coin, interval = %interval, "Resubscribed to {} {}", coin, interval);
                            }
                        }
                    }
                    ClientEvent::Disconnected => {
                        debug!("WebSocket disconnected");
                    }
                    ClientEvent::Error(e) => {
                        error!(error = %e, "WebSocket error");
                    }
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    async fn subscribe_candle_internal(
        &self,
        coin: &str,
        interval: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use super::messages::{RequestMessage, Subscription};
        use super::subscriptions::SubscriptionKey;

        let key = SubscriptionKey::candle(coin, interval);

        if self.subscriptions.contains(&key).await {
            return Ok(()); // Already subscribed
        }

        let subscription = Subscription::candle(coin, interval);
        let request = RequestMessage::Subscribe { subscription };

        let json = serde_json::to_string(&request).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        debug!(subscription = %json, "TaskProvider sending subscription");

        self.client.send_text(json).await.map_err(|e| {
            Box::new(std::io::Error::other(format!(
                "WebSocket send error: {}",
                e
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

        self.subscriptions.add(key).await;
        Ok(())
    }

    async fn process_message(
        &self,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Log all incoming messages for debugging (truncate very long messages)
        let display_text = if text.len() > 500 {
            format!("{}... (truncated)", &text[..500])
        } else {
            text.to_string()
        };
        debug!(message = %display_text, "Raw message received");

        // Try to parse as our known message types
        let msg: WebSocketMessage = match serde_json::from_str(text) {
            Ok(msg) => msg,
            Err(e) => {
                // If it's not a known format, check if it might be candle data with different structure
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                    if let Some(channel) = value.get("channel").and_then(|c| c.as_str()) {
                        debug!(channel = %channel, "Unknown message format with channel");
                        // Log the full structure for debugging
                        if let Some(data) = value.get("data") {
                            debug!(data = ?data, "Message data type");
                        }
                        // Try to parse as candle if channel looks like it
                        if channel.contains("candle") || channel == "candle" {
                            debug!("Attempting to parse as candle data...");
                            if let Ok(candle_data) =
                                serde_json::from_value::<CandleData>(value.clone())
                            {
                                debug!("Successfully parsed as candle data (fallback)");
                                if let Err(e) = self.process_candle_update(candle_data.data).await {
                                    error!(error = %e, "Error processing candle update");
                                }
                                return Ok(());
                            } else {
                                debug!(
                                    "Failed to parse as CandleData, trying alternative formats..."
                                );
                                // Try parsing data as direct CandleUpdate
                                if let Some(data_obj) = value.get("data") {
                                    if let Ok(candle_update) =
                                        serde_json::from_value::<CandleUpdate>(data_obj.clone())
                                    {
                                        debug!("Parsed data as direct CandleUpdate");
                                        if let Err(e) =
                                            self.process_candle_update(candle_update).await
                                        {
                                            error!(error = %e, "Error processing candle update");
                                        }
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    } else {
                        debug!("Message has no 'channel' field");
                    }
                }
                debug!(error = %e, message_preview = %text.chars().take(200).collect::<String>(), "Failed to parse message");
                return Ok(());
            }
        };

        match msg {
            WebSocketMessage::CandleData(candle_data) => {
                debug!(channel = %candle_data.channel, "Received candle data for channel");
                if let Err(e) = self.process_candle_update(candle_data.data).await {
                    error!(error = %e, "Error processing candle update");
                } else {
                    debug!("Successfully processed candle update");
                }
            }
            WebSocketMessage::AllMidsData(mids_data) => {
                debug!(
                    count = mids_data.data.len(),
                    "Received allMids data: {} prices",
                    mids_data.data.len()
                );
                for mid in mids_data.data {
                    let price: f64 = mid.px.parse().unwrap_or(0.0);
                    let mut prices = self.latest_prices.write().await;
                    prices.insert(mid.coin, price);
                }
            }
            WebSocketMessage::SubscriptionResponse(resp) => {
                let sub_info = match &resp.data.subscription {
                    Subscription::Candle { coin, interval, .. } => format!("{}/{}", coin, interval),
                    Subscription::AllMids { .. } => "allMids".to_string(),
                    Subscription::Notification { user, .. } => format!("notification/{}", user),
                };
                let snapshot_info = resp
                    .is_snapshot
                    .map(|s| if s { " (snapshot)" } else { "" })
                    .unwrap_or("");
                debug!(method = %resp.data.method, subscription = %sub_info, snapshot = resp.is_snapshot.is_some(), "Subscription response: {} for {}{}", resp.data.method, sub_info, snapshot_info);
            }
            WebSocketMessage::Error(err) => {
                error!(error = %err.data.error, "WebSocket error");
            }
        }

        Ok(())
    }

    async fn process_candle_update(
        &self,
        update: CandleUpdate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let coin = &update.coin;
        let interval = &update.interval;

        debug!(
            coin = %coin,
            interval = %interval,
            open = %update.open,
            high = %update.high,
            low = %update.low,
            close = %update.close,
            volume = %update.volume,
            "Processing candle: {} {} - O:{} H:{} L:{} C:{} V:{}",
            coin, interval, update.open, update.high, update.low, update.close, update.volume
        );

        let open: f64 = update.open.parse().map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid open price: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;
        let high: f64 = update.high.parse().map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid high price: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;
        let low: f64 = update.low.parse().map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid low price: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;
        let close: f64 = update.close.parse().map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid close price: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;
        let volume: f64 = update.volume.parse().map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid volume: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        // Use end_time as the candle timestamp (when the candle closed)
        let timestamp =
            DateTime::from_timestamp(update.end_time as i64 / 1000, 0).unwrap_or_else(Utc::now);

        let mut candle = Candle::new(open, high, low, close, volume, timestamp);

        self.attach_live_funding_rate(coin, &mut candle).await;

        // Store in QuestDB
        if let Some(ref db) = self.database {
            if let Err(e) = db.store_candle(coin, interval, &candle).await {
                warn!(coin = %coin, interval = %interval, error = %e, "Failed to store candle in QuestDB");
            }
        }

        // Update in-memory buffer
        let symbol_key = format!("{}_{}", coin, interval);
        let mut candles_map = self.candles.write().await;
        let candles = candles_map
            .entry(symbol_key.clone())
            .or_insert_with(VecDeque::new);

        // Remove any existing candle with the same timestamp (update existing candle)
        candles.retain(|c| c.timestamp != timestamp);
        candles.push_back(candle.clone());

        // Keep only last 1000 candles per symbol
        while candles.len() > 1000 {
            candles.pop_front();
        }

        debug!(symbol = %symbol_key, count = candles.len(), "Stored candle for {}: total candles = {}", symbol_key, candles.len());

        // Update Redis cache - get current cached candles, add new one, and update cache
        if let Some(ref cache) = self.cache {
            if let Ok(Some(mut cached_candles)) = cache.get_cached_candles(coin, interval).await {
                // Remove duplicate timestamp
                cached_candles.retain(|c| c.timestamp != timestamp);
                cached_candles.push(candle.clone());
                // Keep only last 200 in cache
                if cached_candles.len() > 200 {
                    cached_candles.remove(0);
                }
                // Update cache
                if let Err(e) = cache.cache_candles(coin, interval, &cached_candles).await {
                    warn!(coin = %coin, interval = %interval, error = %e, "Failed to update Redis cache");
                }
            } else {
                // Cache miss, just cache this single candle
                if let Err(e) = cache
                    .cache_candles(coin, interval, std::slice::from_ref(&candle))
                    .await
                {
                    warn!(coin = %coin, interval = %interval, error = %e, "Failed to cache candle in Redis");
                }
            }
        }

        let mut prices = self.latest_prices.write().await;
        prices.insert(coin.clone(), close);

        Ok(())
    }

    async fn attach_live_funding_rate(&self, coin: &str, candle: &mut Candle) {
        if candle.funding_rate.is_some() {
            return;
        }

        let mut needs_fetch = true;
        {
            let cache = self.funding_cache.read().await;
            if let Some(entry) = cache.get(coin) {
                if candle.timestamp <= entry.timestamp + ChronoDuration::hours(1) {
                    candle.funding_rate = Some(entry.rate);
                    needs_fetch = false;
                }
            }
        }

        if !needs_fetch {
            return;
        }

        match self.rest_client.fetch_latest_funding_rate(coin).await {
            Ok(Some(point)) => {
                candle.funding_rate = Some(point.funding_rate);
                let mut cache = self.funding_cache.write().await;
                cache.insert(
                    coin.to_string(),
                    FundingCacheEntry {
                        timestamp: point.timestamp,
                        rate: point.funding_rate,
                    },
                );
            }
            Ok(None) => {
                debug!(coin = %coin, "No latest funding rate returned for {}", coin);
            }
            Err(e) => {
                warn!(coin = %coin, error = %e, "Failed to fetch latest funding rate for {}", coin);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct FundingCacheEntry {
    timestamp: DateTime<Utc>,
    rate: f64,
}

#[async_trait::async_trait]
impl MarketDataProvider for HyperliquidMarketDataProvider {
    async fn get_candles(
        &self,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        let interval = self.get_primary_interval();
        let symbol_key = format!("{}_{}", symbol, interval);

        // Try Redis cache first
        if let Some(ref cache) = self.cache {
            if let Ok(Some(mut cached_candles)) = cache.get_cached_candles(symbol, interval).await {
                cached_candles.sort_by_key(|c| c.timestamp);
                if cached_candles.len() > limit {
                    let start = cached_candles.len() - limit;
                    cached_candles = cached_candles[start..].to_vec();
                }
                debug!(symbol = %symbol_key, count = cached_candles.len(), "get_candles for {}: found {} candles in Redis cache", symbol_key, cached_candles.len());
                return Ok(cached_candles);
            }
        }

        // Fallback to QuestDB
        if let Some(ref db) = self.database {
            match db.get_candles(symbol, interval, Some(limit)).await {
                Ok(mut db_candles) => {
                    db_candles.sort_by_key(|c| c.timestamp);
                    debug!(symbol = %symbol_key, count = db_candles.len(), "get_candles for {}: found {} candles in QuestDB", symbol_key, db_candles.len());

                    // Update Redis cache with these candles
                    if let Some(ref cache) = self.cache {
                        let _ = cache.cache_candles(symbol, interval, &db_candles).await;
                    }

                    return Ok(db_candles);
                }
                Err(e) => {
                    warn!(symbol = %symbol, interval = %interval, error = %e, "Failed to get candles from QuestDB");
                }
            }
        }

        // Fallback to in-memory buffer
        let candles_map = self.candles.read().await;
        if let Some(candles) = candles_map.get(&symbol_key) {
            let mut result: Vec<Candle> = candles.iter().cloned().collect();
            result.sort_by_key(|c| c.timestamp);

            debug!(symbol = %symbol_key, count = result.len(), "get_candles for {}: found {} candles in memory buffer", symbol_key, result.len());

            // Return last `limit` candles
            if result.len() > limit {
                result = result.into_iter().rev().take(limit).collect();
                result.reverse();
            }

            Ok(result)
        } else {
            debug!(symbol = %symbol_key, "get_candles for {}: no candles found, subscribing...", symbol_key);
            // Try to subscribe if we don't have data yet
            drop(candles_map); // Release lock before async call
            if let Err(e) = self.subscribe_candle(symbol, interval).await {
                error!(symbol = %symbol, error = %e, "Failed to subscribe to {}", symbol);
            }
            Ok(Vec::new())
        }
    }

    async fn get_latest_price(
        &self,
        symbol: &str,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let prices = self.latest_prices.read().await;
        if let Some(&price) = prices.get(symbol) {
            Ok(price)
        } else {
            // Subscribe to get price updates
            if let Err(e) = self
                .subscribe_candle(symbol, self.get_primary_interval())
                .await
            {
                error!(symbol = %symbol, error = %e, "Failed to subscribe to {}", symbol);
            }
            // Wait a bit for price to arrive
            tokio::time::sleep(Duration::from_millis(500)).await;
            let prices = self.latest_prices.read().await;
            Ok(prices.get(symbol).copied().unwrap_or(0.0))
        }
    }

    async fn subscribe(
        &self,
        symbol: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Subscribe to all intervals for this symbol
        for interval in &self.candle_intervals {
            if let Err(e) = self.subscribe_candle(symbol, interval).await {
                error!(symbol = %symbol, interval = %interval, error = %e, "Failed to subscribe to {} {}", symbol, interval);
                // Continue with other intervals even if one fails
            }
        }
        Ok(())
    }
}

impl Default for HyperliquidMarketDataProvider {
    fn default() -> Self {
        Self::new()
    }
}
