//! QuestDB database operations for candles and signals

use crate::config;
use crate::models::indicators::Candle;
use crate::models::signal::{SignalDirection, SignalOutput};
use crate::models::strategy::Strategy;
use chrono::{DateTime, Utc};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::{Client, NoTls};

pub struct QuestDatabase {
    client: Arc<RwLock<Option<Client>>>,
}

impl QuestDatabase {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let questdb_url = config::get_questdb_url();
        let (client, connection) =
            tokio_postgres::connect(&questdb_url, NoTls)
                .await
                .map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::ConnectionRefused,
                        format!("Failed to connect to QuestDB: {}", e),
                    )) as Box<dyn std::error::Error + Send + Sync>
                })?;

        // Spawn connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!(error = %e, "QuestDB connection error");
            }
        });

        let db = Self {
            client: Arc::new(RwLock::new(Some(client))),
        };

        // Initialize schema
        db.init_schema().await?;

        Ok(db)
    }

    async fn init_schema(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            // Create candles table (time-series optimized)
            // QuestDB syntax: TIMESTAMP must be first, PARTITION BY comes after
            c.execute(
                "CREATE TABLE IF NOT EXISTS candles (
                    timestamp TIMESTAMP,
                    symbol SYMBOL,
                    interval SYMBOL,
                    open DOUBLE,
                    high DOUBLE,
                    low DOUBLE,
                    close DOUBLE,
                    volume DOUBLE,
                    open_interest DOUBLE,
                    funding_rate DOUBLE
                ) TIMESTAMP(timestamp) PARTITION BY DAY",
                &[],
            )
            .await
            .map_err(|e| {
                Box::new(std::io::Error::other(format!(
                    "Failed to create candles table: {}",
                    e
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            // Create strategies table
            c.execute(
                "CREATE TABLE IF NOT EXISTS strategies (
                    id LONG,
                    name STRING,
                    symbol SYMBOL,
                    created_at TIMESTAMP,
                    updated_at TIMESTAMP,
                    config_json STRING
                )",
                &[],
            )
            .await
            .map_err(|e| {
                Box::new(std::io::Error::other(format!(
                    "Failed to create strategies table: {}",
                    e
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            // Create signals table
            c.execute(
                "CREATE TABLE IF NOT EXISTS signals (
                    timestamp TIMESTAMP,
                    id LONG,
                    symbol SYMBOL,
                    strategy_id LONG,
                    direction SYMBOL,
                    confidence DOUBLE,
                    sl_pct DOUBLE,
                    tp_pct DOUBLE,
                    price DOUBLE,
                    reasons_json STRING
                ) TIMESTAMP(timestamp) PARTITION BY DAY",
                &[],
            )
            .await
            .map_err(|e| {
                Box::new(std::io::Error::other(format!(
                    "Failed to create signals table: {}",
                    e
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;
        }

        Ok(())
    }

    /// Store a candle in QuestDB
    pub async fn store_candle(
        &self,
        symbol: &str,
        interval: &str,
        candle: &Candle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            // QuestDB expects timestamps - use NaiveDateTime for compatibility
            let timestamp_naive = candle.timestamp.naive_utc();

            c.execute(
                "INSERT INTO candles (timestamp, symbol, interval, open, high, low, close, volume, open_interest, funding_rate)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[
                    &timestamp_naive,
                    &symbol,
                    &interval,
                    &candle.open,
                    &candle.high,
                    &candle.low,
                    &candle.close,
                    &candle.volume,
                    &candle.open_interest.unwrap_or(0.0),
                    &candle.funding_rate.unwrap_or(0.0),
                ],
            )
            .await
            .map_err(|e| {
                Box::new(std::io::Error::other(format!("Failed to store candle: {}", e)))
                    as Box<dyn std::error::Error + Send + Sync>
            })?;
        }

        Ok(())
    }

    /// Store multiple candles in a batch
    pub async fn store_candles_batch(
        &self,
        symbol: &str,
        interval: &str,
        candles: &[Candle],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For now, just store candles one by one
        // TODO: Optimize with batch insert when QuestDB supports it better
        for candle in candles {
            if let Err(e) = self.store_candle(symbol, interval, candle).await {
                tracing::warn!(symbol = %symbol, interval = %interval, error = %e, "Failed to store candle in batch");
            }
        }
        Ok(())
    }

    /// Get candles for a symbol and interval, ordered by timestamp
    pub async fn get_candles(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            let query = if let Some(limit) = limit {
                format!(
                    "SELECT timestamp, open, high, low, close, volume, open_interest, funding_rate
                     FROM candles
                     WHERE symbol = $1 AND interval = $2
                     ORDER BY timestamp DESC
                     LIMIT {}",
                    limit
                )
            } else {
                "SELECT timestamp, open, high, low, close, volume, open_interest, funding_rate
                 FROM candles
                 WHERE symbol = $1 AND interval = $2
                 ORDER BY timestamp DESC"
                    .to_string()
            };

            let rows = c.query(&query, &[&symbol, &interval]).await.map_err(|e| {
                Box::new(std::io::Error::other(format!(
                    "Failed to query candles: {}",
                    e
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            let mut candles = Vec::new();
            for row in rows {
                let timestamp_naive: chrono::NaiveDateTime = row.get(0);
                let timestamp = DateTime::from_naive_utc_and_offset(timestamp_naive, Utc);
                let open: f64 = row.get(1);
                let high: f64 = row.get(2);
                let low: f64 = row.get(3);
                let close: f64 = row.get(4);
                let volume: f64 = row.get(5);
                let open_interest: Option<f64> = row.get(6);
                let funding_rate: Option<f64> = row.get(7);

                let mut candle = Candle::new(open, high, low, close, volume, timestamp);
                if let Some(oi) = open_interest {
                    candle = candle.with_open_interest(oi);
                }
                if let Some(fr) = funding_rate {
                    candle = candle.with_funding_rate(fr);
                }

                candles.push(candle);
            }

            // Reverse to get oldest first
            candles.reverse();

            Ok(candles)
        } else {
            Ok(Vec::new())
        }
    }

    /// Store a signal in QuestDB
    pub async fn store_signal(
        &self,
        signal: &SignalOutput,
        strategy_id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            let direction_str = match signal.direction {
                SignalDirection::Long => "Long",
                SignalDirection::Short => "Short",
                SignalDirection::Neutral => "Neutral",
            };

            let reasons_json = serde_json::to_string(&signal.reasons).map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to serialize reasons: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

            // Generate ID from timestamp (QuestDB doesn't have auto-increment)
            let id = signal.timestamp.timestamp_millis();
            // Convert DateTime<Utc> to NaiveDateTime for QuestDB compatibility
            let timestamp_naive = signal.timestamp.naive_utc();

            c.execute(
                "INSERT INTO signals (timestamp, id, symbol, strategy_id, direction, confidence, sl_pct, tp_pct, price, reasons_json)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[
                    &timestamp_naive,
                    &id,
                    &signal.symbol,
                    &strategy_id,
                    &direction_str,
                    &signal.confidence,
                    &signal.recommended_sl_pct,
                    &signal.recommended_tp_pct,
                    &signal.price,
                    &reasons_json,
                ],
            )
            .await
            .map_err(|e| {
                Box::new(std::io::Error::other(format!("Failed to store signal: {}", e)))
                    as Box<dyn std::error::Error + Send + Sync>
            })?;
        }

        Ok(())
    }

    /// Get signals for a symbol, ordered by timestamp (newest first)
    pub async fn get_signals(
        &self,
        symbol: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<SignalOutput>, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            let query = match (symbol, limit) {
                (Some(_), Some(limit)) => format!(
                    "SELECT symbol, direction, confidence, sl_pct, tp_pct, price, timestamp, reasons_json
                     FROM signals
                     WHERE symbol = $1
                     ORDER BY timestamp DESC
                     LIMIT {}",
                    limit
                ),
                (Some(_), None) => {
                    "SELECT symbol, direction, confidence, sl_pct, tp_pct, price, timestamp, reasons_json
                     FROM signals
                     WHERE symbol = $1
                     ORDER BY timestamp DESC"
                        .to_string()
                }
                (None, Some(limit)) => format!(
                    "SELECT symbol, direction, confidence, sl_pct, tp_pct, price, timestamp, reasons_json
                     FROM signals
                     ORDER BY timestamp DESC
                     LIMIT {}",
                    limit
                ),
                (None, None) => {
                    "SELECT symbol, direction, confidence, sl_pct, tp_pct, price, timestamp, reasons_json
                     FROM signals
                     ORDER BY timestamp DESC"
                        .to_string()
                }
            };

            let rows = if let Some(sym) = symbol {
                c.query(&query, &[&sym]).await
            } else {
                c.query(&query, &[]).await
            }
            .map_err(|e| {
                Box::new(std::io::Error::other(format!(
                    "Failed to query signals: {}",
                    e
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            let mut signals = Vec::new();
            for row in rows {
                let symbol: String = row.get(0);
                let direction_str: String = row.get(1);
                let direction = match direction_str.as_str() {
                    "Long" => SignalDirection::Long,
                    "Short" => SignalDirection::Short,
                    _ => SignalDirection::Neutral,
                };
                let confidence: f64 = row.get(2);
                let sl_pct: f64 = row.get(3);
                let tp_pct: f64 = row.get(4);
                let price: f64 = row.get(5);
                let timestamp_naive: chrono::NaiveDateTime = row.get(6);
                let timestamp = DateTime::from_naive_utc_and_offset(timestamp_naive, Utc);
                let reasons_json: String = row.get(7);

                let reasons: Vec<crate::models::signal::SignalReason> =
                    serde_json::from_str(&reasons_json).map_err(|e| {
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Failed to deserialize reasons: {}", e),
                        )) as Box<dyn std::error::Error + Send + Sync>
                    })?;

                signals.push(SignalOutput {
                    symbol,
                    direction,
                    confidence,
                    recommended_sl_pct: sl_pct,
                    recommended_tp_pct: tp_pct,
                    price,
                    timestamp,
                    reasons,
                });
            }

            Ok(signals)
        } else {
            Ok(Vec::new())
        }
    }

    /// Check if QuestDB connection is available
    pub async fn is_available(&self) -> bool {
        let client = self.client.read().await;
        client.is_some()
    }

    /// Create a new strategy
    pub async fn create_strategy(
        &self,
        strategy: &Strategy,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            let config_json = serde_json::to_string(&strategy.config).map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to serialize strategy config: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

            let id = strategy.created_at.timestamp_millis();
            let created_at_naive = strategy.created_at.naive_utc();
            let updated_at_naive = strategy.updated_at.naive_utc();

            c.execute(
                "INSERT INTO strategies (id, name, symbol, created_at, updated_at, config_json)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                &[
                    &id,
                    &strategy.name,
                    &strategy.symbol,
                    &created_at_naive,
                    &updated_at_naive,
                    &config_json,
                ],
            )
            .await
            .map_err(|e| {
                Box::new(std::io::Error::other(format!(
                    "Failed to create strategy: {}",
                    e
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            Ok(id)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Database connection not available",
            )))
        }
    }

    /// Get a strategy by ID
    pub async fn get_strategy(
        &self,
        id: i64,
    ) -> Result<Strategy, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            let rows = c
                .query(
                    "SELECT id, name, symbol, created_at, updated_at, config_json
                     FROM strategies
                     WHERE id = $1",
                    &[&id],
                )
                .await
                .map_err(|e| {
                    Box::new(std::io::Error::other(format!(
                        "Failed to query strategy: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;

            if rows.is_empty() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Strategy with id {} not found", id),
                )));
            }

            let row = &rows[0];
            let id: i64 = row.get(0);
            let name: String = row.get(1);
            let symbol: String = row.get(2);
            let created_at_naive: chrono::NaiveDateTime = row.get(3);
            let updated_at_naive: chrono::NaiveDateTime = row.get(4);
            let config_json: String = row.get(5);

            let created_at = DateTime::from_naive_utc_and_offset(created_at_naive, Utc);
            let updated_at = DateTime::from_naive_utc_and_offset(updated_at_naive, Utc);

            let config: crate::models::strategy::StrategyConfig =
                serde_json::from_str(&config_json).map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to deserialize strategy config: {}", e),
                    )) as Box<dyn std::error::Error + Send + Sync>
                })?;

            Ok(Strategy {
                id: Some(id),
                name,
                symbol,
                config,
                created_at,
                updated_at,
            })
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Database connection not available",
            )))
        }
    }

    /// Get all strategies, optionally filtered by symbol
    pub async fn get_strategies(
        &self,
        symbol: Option<&str>,
    ) -> Result<Vec<Strategy>, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            let query = if let Some(_sym) = symbol {
                "SELECT id, name, symbol, created_at, updated_at, config_json
                 FROM strategies
                 WHERE symbol = $1
                 ORDER BY created_at DESC"
            } else {
                "SELECT id, name, symbol, created_at, updated_at, config_json
                 FROM strategies
                 ORDER BY created_at DESC"
            };

            let rows = if let Some(sym) = symbol {
                c.query(query, &[&sym]).await
            } else {
                c.query(query, &[]).await
            }
            .map_err(|e| {
                Box::new(std::io::Error::other(format!(
                    "Failed to query strategies: {}",
                    e
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

            let mut strategies = Vec::new();
            for row in rows {
                let id: i64 = row.get(0);
                let name: String = row.get(1);
                let symbol: String = row.get(2);
                let created_at_naive: chrono::NaiveDateTime = row.get(3);
                let updated_at_naive: chrono::NaiveDateTime = row.get(4);
                let config_json: String = row.get(5);

                let created_at = DateTime::from_naive_utc_and_offset(created_at_naive, Utc);
                let updated_at = DateTime::from_naive_utc_and_offset(updated_at_naive, Utc);

                let config: crate::models::strategy::StrategyConfig =
                    serde_json::from_str(&config_json).map_err(|e| {
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Failed to deserialize strategy config: {}", e),
                        )) as Box<dyn std::error::Error + Send + Sync>
                    })?;

                strategies.push(Strategy {
                    id: Some(id),
                    name,
                    symbol,
                    config,
                    created_at,
                    updated_at,
                });
            }

            Ok(strategies)
        } else {
            Ok(Vec::new())
        }
    }

    /// Update a strategy
    pub async fn update_strategy(
        &self,
        id: i64,
        strategy: &Strategy,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            let config_json = serde_json::to_string(&strategy.config).map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to serialize strategy config: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

            let updated_at_naive = strategy.updated_at.naive_utc();

            let rows_affected = c
                .execute(
                    "UPDATE strategies
                     SET name = $1, symbol = $2, updated_at = $3, config_json = $4
                     WHERE id = $5",
                    &[
                        &strategy.name,
                        &strategy.symbol,
                        &updated_at_naive,
                        &config_json,
                        &id,
                    ],
                )
                .await
                .map_err(|e| {
                    Box::new(std::io::Error::other(format!(
                        "Failed to update strategy: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;

            if rows_affected == 0 {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Strategy with id {} not found", id),
                )));
            }

            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Database connection not available",
            )))
        }
    }

    /// Delete a strategy
    pub async fn delete_strategy(
        &self,
        id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.read().await;
        if let Some(ref c) = *client {
            let rows_affected = c
                .execute("DELETE FROM strategies WHERE id = $1", &[&id])
                .await
                .map_err(|e| {
                    Box::new(std::io::Error::other(format!(
                        "Failed to delete strategy: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;

            if rows_affected == 0 {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Strategy with id {} not found", id),
                )));
            }

            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Database connection not available",
            )))
        }
    }
}
