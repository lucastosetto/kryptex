//! Hyperliquid market data provider implementation

use crate::models::indicators::Candle;
use crate::services::market_data::MarketDataProvider;
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

use super::client::{ClientEvent, HyperliquidClient};
use super::messages::{CandleData, CandleUpdate, RequestMessage, Subscription, WebSocketMessage};
use super::subscriptions::{SubscriptionKey, SubscriptionManager};

pub struct HyperliquidMarketDataProvider {
    pub(crate) client: Arc<HyperliquidClient>,
    subscriptions: Arc<SubscriptionManager>,
    candles: Arc<RwLock<HashMap<String, VecDeque<Candle>>>>,
    latest_prices: Arc<RwLock<HashMap<String, f64>>>,
    candle_intervals: Vec<String>,
    pending_subscriptions: Arc<RwLock<Vec<(String, String)>>>, // (coin, interval)
}

impl HyperliquidMarketDataProvider {
    pub fn new() -> Self {
        Self::with_intervals(vec!["1m".to_string(), "5m".to_string(), "15m".to_string(), "1h".to_string()])
    }

    pub fn with_intervals(candle_intervals: Vec<String>) -> Self {
        let provider = Self {
            client: Arc::new(HyperliquidClient::new()),
            subscriptions: Arc::new(SubscriptionManager::new()),
            candles: Arc::new(RwLock::new(HashMap::new())),
            latest_prices: Arc::new(RwLock::new(HashMap::new())),
            candle_intervals: candle_intervals.clone(),
            pending_subscriptions: Arc::new(RwLock::new(Vec::new())),
        };

        // Start connection task in background
        let client_clone = provider.client.clone();
        tokio::spawn(async move {
            let _ = client_clone.connect().await;
        });

        // Start message handler task
        let provider_clone = provider.clone_for_task();
        tokio::spawn(async move {
            provider_clone.handle_messages().await;
        });

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
        }
    }

    async fn subscribe_candle(&self, coin: &str, interval: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            println!("  [DEBUG] Not connected yet, subscription queued for {}/{}", coin, interval);
            Ok(())
        }
    }

    async fn subscribe_candle_internal(&self, coin: &str, interval: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = SubscriptionKey::candle(coin, interval);
        
        if self.subscriptions.contains(&key).await {
            return Ok(()); // Already subscribed
        }

        let subscription = Subscription::candle(coin, interval);
        let request = RequestMessage::Subscribe { subscription };

        let json = serde_json::to_string(&request)
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) as Box<dyn std::error::Error + Send + Sync>)?;
        
        println!("  [DEBUG] Sending subscription: {}", json);
        
        self.client.send_text(json).await
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("WebSocket send error: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

        self.subscriptions.add(key).await;
        Ok(())
    }


    fn get_primary_interval(&self) -> &str {
        self.candle_intervals.first().map(|s| s.as_str()).unwrap_or("1m")
    }

    pub fn client(&self) -> &Arc<HyperliquidClient> {
        &self.client
    }
}

#[derive(Clone)]
struct TaskProvider {
    client: Arc<HyperliquidClient>,
    #[allow(dead_code)] // Kept for future resubscription functionality
    subscriptions: Arc<SubscriptionManager>,
    candles: Arc<RwLock<HashMap<String, VecDeque<Candle>>>>,
    latest_prices: Arc<RwLock<HashMap<String, f64>>>,
    pending_subscriptions: Arc<RwLock<Vec<(String, String)>>>,
    #[allow(dead_code)] // Used for resubscription
    candle_intervals: Vec<String>,
}

impl TaskProvider {
    async fn handle_messages(&self) {
        loop {
            if let Some(event) = self.client.receive().await {
                match event {
                    ClientEvent::Message(text) => {
                        if let Err(e) = self.process_message(&text).await {
                            eprintln!("Error processing message: {}", e);
                        }
                    }
                    ClientEvent::Connected => {
                        println!("  [DEBUG] TaskProvider: WebSocket connected, resubscribing...");
                        // Wait a moment for connection to stabilize
                        sleep(Duration::from_millis(500)).await;
                        // Resubscribe to all pending subscriptions
                        let pending = self.pending_subscriptions.read().await.clone();
                        println!("  [DEBUG] Resubscribing to {} pending subscriptions", pending.len());
                        for (coin, interval) in pending {
                            if let Err(e) = self.subscribe_candle_internal(&coin, &interval).await {
                                eprintln!("  [DEBUG] Failed to resubscribe to {} {}: {}", coin, interval, e);
                            } else {
                                println!("  [DEBUG] Resubscribed to {} {}", coin, interval);
                            }
                        }
                    }
                    ClientEvent::Disconnected => {
                        eprintln!("  [DEBUG] TaskProvider: WebSocket disconnected");
                    }
                    ClientEvent::Error(e) => {
                        eprintln!("  [DEBUG] TaskProvider: WebSocket error: {}", e);
                    }
                }
            } else {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    async fn subscribe_candle_internal(&self, coin: &str, interval: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use super::subscriptions::SubscriptionKey;
        use super::messages::{RequestMessage, Subscription};
        
        let key = SubscriptionKey::candle(coin, interval);
        
        if self.subscriptions.contains(&key).await {
            return Ok(()); // Already subscribed
        }

        let subscription = Subscription::candle(coin, interval);
        let request = RequestMessage::Subscribe { subscription };

        let json = serde_json::to_string(&request)
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) as Box<dyn std::error::Error + Send + Sync>)?;
        
        println!("  [DEBUG] TaskProvider sending subscription: {}", json);
        
        self.client.send_text(json).await
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("WebSocket send error: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

        self.subscriptions.add(key).await;
        Ok(())
    }

    async fn process_message(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Log all incoming messages for debugging
        println!("  [DEBUG] Raw message received: {}", text);
        
        // Try to parse as our known message types
        let msg: WebSocketMessage = match serde_json::from_str(text) {
            Ok(msg) => msg,
            Err(e) => {
                // If it's not a known format, check if it might be candle data with different structure
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                    if let Some(channel) = value.get("channel").and_then(|c| c.as_str()) {
                        println!("  [DEBUG] Unknown message format with channel: {}", channel);
                        // Try to parse as candle if channel looks like it
                        if channel.contains("candle") || channel == "candle" {
                            if let Ok(candle_data) = serde_json::from_value::<CandleData>(value.clone()) {
                                println!("  [DEBUG] Parsed as candle data (fallback)");
                                if let Err(e) = self.process_candle_update(candle_data.data).await {
                                    eprintln!("Error processing candle update: {}", e);
                                }
                                return Ok(());
                            }
                        }
                    }
                }
                eprintln!("  [DEBUG] Failed to parse message: {} - Raw: {}", e, text);
                return Ok(());
            }
        };

        match msg {
            WebSocketMessage::CandleData(candle_data) => {
                println!("  [DEBUG] Received candle data for channel {}", candle_data.channel);
                if let Err(e) = self.process_candle_update(candle_data.data).await {
                    eprintln!("Error processing candle update: {}", e);
                }
            }
            WebSocketMessage::AllMidsData(mids_data) => {
                println!("  [DEBUG] Received allMids data: {} prices", mids_data.data.len());
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
                let snapshot_info = resp.is_snapshot.map(|s| if s { " (snapshot)" } else { "" }).unwrap_or("");
                println!("  [DEBUG] Subscription response: {} for {}{}", resp.data.method, sub_info, snapshot_info);
            }
            WebSocketMessage::Error(err) => {
                eprintln!("WebSocket error: {}", err.data.error);
            }
        }

        Ok(())
    }

    async fn process_candle_update(&self, update: CandleUpdate) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let coin = &update.coin;
        let interval = &update.interval;
        
        println!("  [DEBUG] Processing candle: {} {} - O:{} H:{} L:{} C:{} V:{}", coin, interval, update.open, update.high, update.low, update.close, update.volume);

        let open: f64 = update.open.parse()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid open price: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
        let high: f64 = update.high.parse()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid high price: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
        let low: f64 = update.low.parse()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid low price: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
        let close: f64 = update.close.parse()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid close price: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
        let volume: f64 = update.volume.parse()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid volume: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

        // Use end_time as the candle timestamp (when the candle closed)
        let timestamp = DateTime::from_timestamp(update.end_time as i64 / 1000, 0)
            .unwrap_or_else(Utc::now);

        let candle = Candle::new(open, high, low, close, volume, timestamp);

        let symbol_key = format!("{}_{}", coin, interval);
        let mut candles_map = self.candles.write().await;
        let candles = candles_map.entry(symbol_key.clone()).or_insert_with(VecDeque::new);

        // Remove any existing candle with the same timestamp (update existing candle)
        candles.retain(|c| c.timestamp != timestamp);
        candles.push_back(candle.clone());
        
        // Keep only last 1000 candles per symbol
        while candles.len() > 1000 {
            candles.pop_front();
        }

        println!("  [DEBUG] Stored candle for {}: total candles = {}", symbol_key, candles.len());

        let mut prices = self.latest_prices.write().await;
        prices.insert(coin.clone(), close);

        Ok(())
    }
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
        
        let candles_map = self.candles.read().await;
        if let Some(candles) = candles_map.get(&symbol_key) {
            let mut result: Vec<Candle> = candles.iter().cloned().collect();
            result.sort_by_key(|c| c.timestamp);
            
            println!("  [DEBUG] get_candles for {}: found {} candles in buffer", symbol_key, result.len());
            
            // Return last `limit` candles
            if result.len() > limit {
                result = result.into_iter().rev().take(limit).collect();
                result.reverse();
            }
            
            Ok(result)
        } else {
            println!("  [DEBUG] get_candles for {}: no candles found, subscribing...", symbol_key);
            // Try to subscribe if we don't have data yet
            drop(candles_map); // Release lock before async call
            if let Err(e) = self.subscribe_candle(symbol, interval).await {
                eprintln!("Failed to subscribe to {}: {}", symbol, e);
            }
            Ok(Vec::new())
        }
    }

    async fn get_latest_price(&self, symbol: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let prices = self.latest_prices.read().await;
        if let Some(&price) = prices.get(symbol) {
            Ok(price)
        } else {
            // Subscribe to get price updates
            if let Err(e) = self.subscribe_candle(symbol, self.get_primary_interval()).await {
                eprintln!("Failed to subscribe to {}: {}", symbol, e);
            }
            // Wait a bit for price to arrive
            tokio::time::sleep(Duration::from_millis(500)).await;
            let prices = self.latest_prices.read().await;
            Ok(prices.get(symbol).copied().unwrap_or(0.0))
        }
    }

    async fn subscribe(&self, symbol: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Subscribe to all intervals for this symbol
        for interval in &self.candle_intervals {
            if let Err(e) = self.subscribe_candle(symbol, interval).await {
                eprintln!("Failed to subscribe to {} {}: {}", symbol, interval, e);
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

