//! WebSocket service for maintaining long-lived connection to market data provider

use crate::services::hyperliquid::HyperliquidMarketDataProvider;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{info, warn};

/// WebSocket service that maintains a persistent connection to the market data provider
/// 
/// This service runs independently and maintains the WebSocket connection.
/// It receives real-time updates and stores them in Redis/QuestDB.
/// Jobs read from the stored data and never create new connections.
pub struct WebSocketService {
    provider: Arc<HyperliquidMarketDataProvider>,
    handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl WebSocketService {
    /// Create a new WebSocket service with a provider
    /// 
    /// The provider should already have database and cache configured.
    /// The service will start background tasks to maintain the connection.
    pub fn new(provider: HyperliquidMarketDataProvider) -> Self {
        // The provider's spawn_background_tasks() is called in with_clients(),
        // so the connection is already being maintained
        Self {
            provider: Arc::new(provider),
            handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the WebSocket service monitoring
    /// 
    /// This monitors the connection health. The actual connection
    /// is maintained by the provider's background tasks.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider.clone();
        let handle_arc = self.handle.clone();

        let handle = tokio::spawn(async move {
            // Wait for initial connection
            let client = provider.client();
            if client.wait_for_connection(Duration::from_secs(10)).await {
                info!("WebSocket service: connection established");
            } else {
                warn!("WebSocket service: connection timeout, background tasks will retry");
            }

            // Monitor connection health periodically
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let is_connected = client.is_connected().await;
                if !is_connected {
                    warn!("WebSocket service: connection lost, background tasks will reconnect");
                }
            }
        });

        {
            let mut h = handle_arc.write().await;
            *h = Some(handle);
        }

        Ok(())
    }

    /// Stop the WebSocket service
    pub async fn stop(&self) {
        let mut handle = self.handle.write().await;
        if let Some(h) = handle.take() {
            h.abort();
            info!("WebSocket service stopped");
        }
    }

        /// Get the provider (for subscribing to symbols)
        pub fn get_provider(&self) -> Arc<HyperliquidMarketDataProvider> {
            self.provider.clone()
        }

        /// Subscribe to a symbol
        pub async fn subscribe(&self, symbol: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            // Arc<T> implements Deref<Target = T>, so we can call methods directly
            self.provider.subscribe(symbol).await
        }

    /// Check if the service is running
    pub async fn is_running(&self) -> bool {
        let handle = self.handle.read().await;
        handle.is_some()
    }
}

