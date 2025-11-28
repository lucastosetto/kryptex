//! Redis cache for candles

use crate::config;
use crate::models::indicators::Candle;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::RwLock;

const CANDLE_CACHE_TTL: i64 = 3600; // 1 hour in seconds
const CACHE_KEY_PREFIX: &str = "candles";

pub struct RedisCache {
    client: Arc<RwLock<Option<redis::aio::ConnectionManager>>>,
}

impl RedisCache {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let redis_url = config::get_redis_url();
        let client = redis::Client::open(redis_url.as_str()).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Failed to create Redis client: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        let connection = client
            .get_connection_manager()
            .await
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!("Failed to connect to Redis: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

        Ok(Self {
            client: Arc::new(RwLock::new(Some(connection))),
        })
    }

    /// Cache candles for a symbol and interval
    pub async fn cache_candles(
        &self,
        symbol: &str,
        interval: &str,
        candles: &[Candle],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.write().await;
        if let Some(ref mut c) = *conn {
            let key = format!("{}:{}:{}", CACHE_KEY_PREFIX, symbol, interval);
            let json = serde_json::to_string(candles).map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to serialize candles: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

            c.set::<_, _, ()>(&key, &json)
                .await
                .map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to set cache: {}", e),
                    )) as Box<dyn std::error::Error + Send + Sync>
                })?;

            c.expire::<_, ()>(&key, CANDLE_CACHE_TTL)
                .await
                .map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to set cache TTL: {}", e),
                    )) as Box<dyn std::error::Error + Send + Sync>
                })?;
        }

        Ok(())
    }

    /// Get cached candles for a symbol and interval
    pub async fn get_cached_candles(
        &self,
        symbol: &str,
        interval: &str,
    ) -> Result<Option<Vec<Candle>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.write().await;
        if let Some(ref mut c) = *conn {
            let key = format!("{}:{}:{}", CACHE_KEY_PREFIX, symbol, interval);
            let json: Option<String> = c.get(&key).await.map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to get cache: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

            if let Some(json_str) = json {
                let candles: Vec<Candle> = serde_json::from_str(&json_str).map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to deserialize candles: {}", e),
                    )) as Box<dyn std::error::Error + Send + Sync>
                })?;
                return Ok(Some(candles));
            }
        }

        Ok(None)
    }

    /// Invalidate cache for a symbol and interval
    pub async fn invalidate_candles(
        &self,
        symbol: &str,
        interval: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.client.write().await;
        if let Some(ref mut c) = *conn {
            let key = format!("{}:{}:{}", CACHE_KEY_PREFIX, symbol, interval);
            c.del::<_, ()>(&key)
                .await
                .map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to delete cache: {}", e),
                    )) as Box<dyn std::error::Error + Send + Sync>
                })?;
        }

        Ok(())
    }

    /// Check if Redis connection is available
    pub async fn is_available(&self) -> bool {
        let conn = self.client.read().await;
        conn.is_some()
    }
}

