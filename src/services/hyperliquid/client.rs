//! Hyperliquid WebSocket client

use crate::config::get_hyperliquid_ws_url;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

pub type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug, Clone)]
pub enum ClientEvent {
    Message(String),
    Connected,
    Disconnected,
    Error(String),
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

        loop {
            match self.try_connect().await {
                Ok(()) => {
                    // Only print once per connection, not on every reconnect
                    if current_delay == self.reconnect_delay {
                        println!("Hyperliquid WebSocket connected");
                    } else {
                        println!("Hyperliquid WebSocket reconnected");
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
                    eprintln!("Failed to connect: {}. Retrying in {:?}...", error_msg, current_delay);
                    sleep(current_delay).await;
                    current_delay = std::cmp::min(
                        current_delay * 2,
                        self.max_reconnect_delay,
                    );
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
                    eprintln!("Error sending message: {}", e);
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
                    Ok(Message::Close(_)) => {
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
                    Err(e) => {
                        eprintln!("WebSocket read error: {}", e);
                        let _ = event_tx_reader.send(ClientEvent::Error(e.to_string()));
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    async fn run_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        // This method is called after connection is established
        // The connection will be maintained by the spawned tasks
        // Wait for disconnection event
        loop {
            if let Some(event) = self.receive().await {
                match event {
                    ClientEvent::Disconnected | ClientEvent::Error(_) => {
                        break;
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    pub async fn send(&self, message: Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    pub async fn send_text(&self, text: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

