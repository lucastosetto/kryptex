//! Hyperliquid WebSocket client

use crate::config::get_hyperliquid_ws_url;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};
use url::Url;

pub type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug, Clone)]
pub enum ClientEvent {
    Message(String),
    Connected,
    Disconnected,
    Error(String),
}

#[async_trait]
pub trait WebSocketClient: Send + Sync + 'static {
    async fn connect(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn send(&self, message: Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn send_text(
        &self,
        text: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send(Message::Text(text)).await
    }

    async fn receive(&self) -> Option<ClientEvent>;

    async fn is_connected(&self) -> bool;

    async fn wait_for_connection(&self, timeout: Duration) -> bool;
}

pub struct HyperliquidClient {
    url: String,
    sender: Arc<RwLock<Option<mpsc::UnboundedSender<Message>>>>,
    receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ClientEvent>>>>,
    reconnect_delay: Duration,
    max_reconnect_delay: Duration,
}

impl HyperliquidClient {
    pub fn new() -> Self {
        Self::with_url(get_hyperliquid_ws_url())
    }

    pub fn with_url(url: String) -> Self {
        Self {
            url,
            sender: Arc::new(RwLock::new(None)),
            receiver: Arc::new(RwLock::new(None)),
            reconnect_delay: Duration::from_secs(1),
            max_reconnect_delay: Duration::from_secs(60),
        }
    }

    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut current_delay = self.reconnect_delay;

        let mut is_first_connection = true;
        loop {
            match self.try_connect().await {
                Ok(()) => {
                    // Only print once per initial connection, not on every reconnect
                    if is_first_connection {
                        info!("Hyperliquid WebSocket connected");
                        is_first_connection = false;
                    } else {
                        info!(delay = ?current_delay, "Hyperliquid WebSocket reconnected (delay was {:?})", current_delay);
                    }
                    current_delay = self.reconnect_delay;

                    // Spawn ping task to keep connection alive
                    let sender_clone = self.sender.clone();
                    tokio::spawn(async move {
                        let mut ping_interval = tokio::time::interval(Duration::from_secs(30));
                        loop {
                            ping_interval.tick().await;
                            if let Some(sender) = sender_clone.read().await.as_ref() {
                                if sender.send(Message::Ping(vec![])).is_err() {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    });

                    // Wait for disconnection
                    let _ = self.run_connection().await;
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    warn!(error = %e, delay = ?current_delay, "Failed to connect: {}. Retrying in {:?}...", error_msg, current_delay);
                    sleep(current_delay).await;
                    current_delay = std::cmp::min(current_delay * 2, self.max_reconnect_delay);
                }
            }
        }
    }

    async fn try_connect(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(&self.url)?;
        let (ws_stream, _) = connect_async(url).await?;

        let (mut write, mut read) = ws_stream.split();

        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        let (event_tx, event_rx) = mpsc::unbounded_channel::<ClientEvent>();

        // Store sender and receiver
        {
            let mut sender_guard = self.sender.write().await;
            *sender_guard = Some(tx.clone());
        }
        {
            let mut receiver_guard = self.receiver.write().await;
            *receiver_guard = Some(event_rx);
        }

        // Send connection event
        let _ = event_tx.send(ClientEvent::Connected);

        // Spawn writer task
        let event_tx_writer = event_tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    error!(error = %e, "Error sending message");
                    let _ = event_tx_writer.send(ClientEvent::Error(e.to_string()));
                    break;
                }
            }
        });

        // Spawn reader task
        let event_tx_reader = event_tx.clone();
        let sender_for_pong = tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let _ = event_tx_reader.send(ClientEvent::Message(text));
                    }
                    Ok(Message::Close(frame)) => {
                        debug!(frame = ?frame, "WebSocket received Close frame");
                        let _ = event_tx_reader.send(ClientEvent::Disconnected);
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        // Auto-respond to ping with pong
                        let _ = sender_for_pong.send(Message::Pong(data));
                    }
                    Ok(Message::Pong(_)) => {
                        // Pong received, connection is alive
                    }
                    Ok(Message::Binary(data)) => {
                        debug!(
                            bytes = data.len(),
                            "WebSocket received binary message ({} bytes)",
                            data.len()
                        );
                    }
                    Ok(Message::Frame(_)) => {
                        // Raw frame, should be handled by tungstenite
                    }
                    Err(e) => {
                        error!(error = %e, "WebSocket read error");
                        let _ = event_tx_reader.send(ClientEvent::Error(e.to_string()));
                        break;
                    }
                }
            }
            debug!("WebSocket reader task ended");
        });

        Ok(())
    }

    async fn run_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        // This method is called after connection is established
        // The connection will be maintained by the spawned tasks
        // Wait for disconnection event
        while let Some(event) = self.receive().await {
            if matches!(event, ClientEvent::Disconnected | ClientEvent::Error(_)) {
                break;
            }
        }
        Ok(())
    }

    pub async fn send(
        &self,
        message: Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(sender) = self.sender.read().await.as_ref() {
            sender.send(message)?;
            Ok(())
        } else {
            Err("Not connected".into())
        }
    }

    pub async fn receive(&self) -> Option<ClientEvent> {
        // We need to take the receiver out of the lock temporarily
        // This is a bit awkward but necessary for mpsc::UnboundedReceiver
        let mut receiver_guard = self.receiver.write().await;
        if let Some(mut receiver) = receiver_guard.take() {
            let result = receiver.recv().await;
            *receiver_guard = Some(receiver);
            result
        } else {
            None
        }
    }

    pub async fn send_text(
        &self,
        text: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send(Message::Text(text)).await
    }

    pub async fn is_connected(&self) -> bool {
        self.sender.read().await.is_some()
    }

    pub async fn wait_for_connection(&self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self.is_connected().await {
                return true;
            }
            sleep(Duration::from_millis(100)).await;
        }
        false
    }
}

impl Default for HyperliquidClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WebSocketClient for HyperliquidClient {
    async fn connect(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        HyperliquidClient::connect(self).await
    }

    async fn send(&self, message: Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        HyperliquidClient::send(self, message).await
    }

    async fn send_text(
        &self,
        text: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        HyperliquidClient::send_text(self, text).await
    }

    async fn receive(&self) -> Option<ClientEvent> {
        HyperliquidClient::receive(self).await
    }

    async fn is_connected(&self) -> bool {
        HyperliquidClient::is_connected(self).await
    }

    async fn wait_for_connection(&self, timeout: Duration) -> bool {
        HyperliquidClient::wait_for_connection(self, timeout).await
    }
}

/// Simple in-memory WebSocket client mock for integration testing.
pub struct MockWebSocketClient {
    connected: Arc<RwLock<bool>>,
    sent_messages: Arc<RwLock<Vec<Message>>>,
    receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ClientEvent>>>>,
    sender: mpsc::UnboundedSender<ClientEvent>,
}

impl MockWebSocketClient {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            connected: Arc::new(RwLock::new(false)),
            sent_messages: Arc::new(RwLock::new(Vec::new())),
            receiver: Arc::new(RwLock::new(Some(rx))),
            sender: tx,
        }
    }

    /// Seed the mock with an event that will later be consumed.
    pub async fn push_event(&self, event: ClientEvent) {
        let _ = self.sender.send(event);
    }

    /// Retrieve a snapshot of messages that were sent through this client.
    pub async fn sent_messages(&self) -> Vec<Message> {
        self.sent_messages.read().await.clone()
    }

    /// Manually set the connection state.
    pub async fn set_connected(&self, connected: bool) {
        *self.connected.write().await = connected;
    }
}

impl Default for MockWebSocketClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WebSocketClient for MockWebSocketClient {
    async fn connect(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            let mut guard = self.connected.write().await;
            *guard = true;
        }
        let _ = self.sender.send(ClientEvent::Connected);
        Ok(())
    }

    async fn send(&self, message: Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.sent_messages.write().await.push(message);
        Ok(())
    }

    async fn receive(&self) -> Option<ClientEvent> {
        let mut receiver_guard = self.receiver.write().await;
        if let Some(mut receiver) = receiver_guard.take() {
            let result = receiver.recv().await;
            *receiver_guard = Some(receiver);
            result
        } else {
            None
        }
    }

    async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    async fn wait_for_connection(&self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self.is_connected().await {
                return true;
            }
            sleep(Duration::from_millis(50)).await;
        }
        false
    }
}
