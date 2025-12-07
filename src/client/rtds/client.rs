//! RTDS WebSocket client implementation.

use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::interval;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{debug, error, info, trace, warn};

use super::model::{ConnectionStatus, Message, Subscription, SubscriptionRequest};
use crate::error::{PolymarketError, Result};

/// Default WebSocket server host.
pub const DEFAULT_HOST: &str = "wss://ws-live-data.polymarket.com";

/// Default ping interval (5 seconds).
pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(5);

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>;
type WsReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Builder for configuring `RtdsClient`.
#[derive(Debug, Clone)]
pub struct RtdsClientBuilder {
    host: String,
    ping_interval: Duration,
    auto_reconnect: bool,
}

impl Default for RtdsClientBuilder {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            ping_interval: DEFAULT_PING_INTERVAL,
            auto_reconnect: true,
        }
    }
}

impl RtdsClientBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the WebSocket host URL.
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Sets the ping interval for keepalive.
    pub fn ping_interval(mut self, interval: Duration) -> Self {
        self.ping_interval = interval;
        self
    }

    /// Enables or disables auto-reconnect on disconnect.
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }

    /// Builds the `RtdsClient`.
    pub fn build(self) -> RtdsClient {
        RtdsClient {
            host: self.host,
            ping_interval: self.ping_interval,
            auto_reconnect: self.auto_reconnect,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            writer: Arc::new(Mutex::new(None)),
            reader: Arc::new(Mutex::new(None)),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Polymarket Real-Time Data Service WebSocket client.
///
/// Provides real-time streaming of market data, trades, comments, prices, and CLOB updates.
#[derive(Debug)]
pub struct RtdsClient {
    host: String,
    ping_interval: Duration,
    auto_reconnect: bool,
    status: Arc<Mutex<ConnectionStatus>>,
    writer: Arc<Mutex<Option<WsWriter>>>,
    reader: Arc<Mutex<Option<WsReader>>>,
    subscriptions: Arc<Mutex<Vec<Subscription>>>,
}

impl Clone for RtdsClient {
    fn clone(&self) -> Self {
        Self {
            host: self.host.clone(),
            ping_interval: self.ping_interval,
            auto_reconnect: self.auto_reconnect,
            status: Arc::clone(&self.status),
            writer: Arc::clone(&self.writer),
            reader: Arc::clone(&self.reader),
            subscriptions: Arc::clone(&self.subscriptions),
        }
    }
}

impl Default for RtdsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RtdsClient {
    /// Creates a new `RtdsClient` with default settings.
    pub fn new() -> Self {
        RtdsClientBuilder::default().build()
    }

    /// Returns a builder for configuring the client.
    pub fn builder() -> RtdsClientBuilder {
        RtdsClientBuilder::new()
    }

    /// Returns the current connection status.
    pub async fn status(&self) -> ConnectionStatus {
        *self.status.lock().await
    }

    /// Connects to the RTDS WebSocket server.
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to RTDS at {}", self.host);
        self.set_status(ConnectionStatus::Connecting).await;

        let (ws_stream, response) = connect_async(&self.host)
            .await
            .map_err(|e| PolymarketError::websocket(format!("Connection failed: {e}")))?;

        debug!(
            "WebSocket handshake completed with status: {}",
            response.status()
        );

        let (writer, reader) = ws_stream.split();
        *self.writer.lock().await = Some(writer);
        *self.reader.lock().await = Some(reader);

        self.set_status(ConnectionStatus::Connected).await;
        info!("Connected to RTDS");

        // Resubscribe to previous subscriptions after reconnect
        let subs = self.subscriptions.lock().await.clone();
        if !subs.is_empty() {
            debug!("Resubscribing to {} topics", subs.len());
            self.subscribe_internal(subs).await?;
        }

        // Start ping loop
        self.start_ping_loop();

        Ok(())
    }

    /// Disconnects from the RTDS WebSocket server.
    pub async fn disconnect(&mut self) {
        info!("Disconnecting from RTDS");

        if let Some(mut writer) = self.writer.lock().await.take() {
            let _ = writer.close().await;
        }
        *self.reader.lock().await = None;

        self.set_status(ConnectionStatus::Disconnected).await;
        info!("Disconnected from RTDS");
    }

    /// Subscribes to the specified topics.
    pub async fn subscribe(&mut self, subscriptions: Vec<Subscription>) -> Result<()> {
        // Store subscriptions for reconnect
        {
            let mut stored = self.subscriptions.lock().await;
            for sub in &subscriptions {
                if !stored
                    .iter()
                    .any(|s| s.topic == sub.topic && s.message_type == sub.message_type)
                {
                    stored.push(sub.clone());
                }
            }
        }

        self.subscribe_internal(subscriptions).await
    }

    /// Internal subscribe without storing.
    async fn subscribe_internal(&self, subscriptions: Vec<Subscription>) -> Result<()> {
        let request = SubscriptionRequest::subscribe(subscriptions);
        let json = serde_json::to_string(&request)?;

        trace!("Sending subscribe: {}", json);
        self.send_text(json).await
    }

    /// Unsubscribes from the specified topics.
    pub async fn unsubscribe(&mut self, subscriptions: Vec<Subscription>) -> Result<()> {
        // Remove from stored subscriptions
        {
            let mut stored = self.subscriptions.lock().await;
            stored.retain(|s| {
                !subscriptions
                    .iter()
                    .any(|u| u.topic == s.topic && u.message_type == s.message_type)
            });
        }

        let request = SubscriptionRequest::unsubscribe(subscriptions);
        let json = serde_json::to_string(&request)?;

        trace!("Sending unsubscribe: {}", json);
        self.send_text(json).await
    }

    /// Receives the next message from the WebSocket.
    ///
    /// Returns `None` if the connection is closed.
    pub async fn next_message(&mut self) -> Option<Message> {
        loop {
            let msg = {
                let mut reader_guard = self.reader.lock().await;
                let reader = reader_guard.as_mut()?;
                reader.next().await
            };

            match msg {
                Some(Ok(WsMessage::Text(text))) => {
                    // Skip pong responses
                    if text == "pong" || text.is_empty() {
                        continue;
                    }

                    // Try to parse as Message
                    if text.contains("payload") {
                        match serde_json::from_str::<Message>(&text) {
                            Ok(message) => {
                                trace!(
                                    "Received message: topic={}, type={}",
                                    message.topic, message.message_type
                                );
                                return Some(message);
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {e}");
                                debug!("Raw message: {}", text);
                            }
                        }
                    } else {
                        trace!("Received non-payload message: {}", text);
                    }
                }
                Some(Ok(WsMessage::Ping(data))) => {
                    trace!("Received ping, sending pong");
                    if let Err(e) = self.send_pong(data.to_vec()).await {
                        error!("Failed to send pong: {e}");
                    }
                }
                Some(Ok(WsMessage::Pong(_))) => {
                    trace!("Received pong");
                }
                Some(Ok(WsMessage::Close(frame))) => {
                    info!("Connection closed: {:?}", frame);
                    self.set_status(ConnectionStatus::Disconnected).await;

                    if self.auto_reconnect {
                        info!("Attempting to reconnect...");
                        if let Err(e) = self.reconnect().await {
                            error!("Reconnect failed: {e}");
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                Some(Err(e)) => {
                    error!("WebSocket error: {e}");
                    self.set_status(ConnectionStatus::Disconnected).await;

                    if self.auto_reconnect {
                        info!("Attempting to reconnect...");
                        if let Err(e) = self.reconnect().await {
                            error!("Reconnect failed: {e}");
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                None => {
                    debug!("WebSocket stream ended");
                    self.set_status(ConnectionStatus::Disconnected).await;

                    if self.auto_reconnect {
                        info!("Attempting to reconnect...");
                        if let Err(e) = self.reconnect().await {
                            error!("Reconnect failed: {e}");
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                _ => {
                    // Ignore binary and other message types
                }
            }
        }
    }

    /// Sends a text message to the WebSocket.
    async fn send_text(&self, text: String) -> Result<()> {
        let mut writer_guard = self.writer.lock().await;
        let writer = writer_guard
            .as_mut()
            .ok_or_else(|| PolymarketError::websocket("Not connected"))?;

        writer
            .send(WsMessage::Text(text.into()))
            .await
            .map_err(|e| PolymarketError::websocket(format!("Send failed: {e}")))?;

        Ok(())
    }

    /// Sends a pong response.
    async fn send_pong(&self, data: Vec<u8>) -> Result<()> {
        let mut writer_guard = self.writer.lock().await;
        if let Some(writer) = writer_guard.as_mut() {
            writer
                .send(WsMessage::Pong(data.into()))
                .await
                .map_err(|e| PolymarketError::websocket(format!("Pong failed: {e}")))?;
        }
        Ok(())
    }

    /// Sends a ping message.
    async fn send_ping(&self) -> Result<()> {
        self.send_text("ping".to_string()).await
    }

    /// Sets the connection status.
    async fn set_status(&self, status: ConnectionStatus) {
        *self.status.lock().await = status;
    }

    /// Starts the ping keepalive loop.
    fn start_ping_loop(&self) {
        let client = self.clone();
        let ping_interval = self.ping_interval;

        tokio::spawn(async move {
            let mut ticker = interval(ping_interval);

            loop {
                ticker.tick().await;

                let status = client.status().await;
                if status != ConnectionStatus::Connected {
                    debug!("Ping loop stopping: not connected");
                    break;
                }

                if let Err(e) = client.send_ping().await {
                    warn!("Ping failed: {e}");
                    break;
                }
                trace!("Ping sent");
            }
        });
    }

    /// Attempts to reconnect to the server.
    async fn reconnect(&mut self) -> Result<()> {
        // Clear existing connection
        *self.writer.lock().await = None;
        *self.reader.lock().await = None;

        // Exponential backoff: try up to 5 times
        for attempt in 1..=5 {
            let delay = Duration::from_millis(500 * 2u64.pow(attempt - 1));
            info!("Reconnect attempt {} in {:?}", attempt, delay);
            tokio::time::sleep(delay).await;

            match self.connect().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    warn!("Reconnect attempt {} failed: {e}", attempt);
                }
            }
        }

        Err(PolymarketError::websocket(
            "Max reconnect attempts exceeded",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let client = RtdsClient::builder().build();
        assert_eq!(client.host, DEFAULT_HOST);
        assert_eq!(client.ping_interval, DEFAULT_PING_INTERVAL);
        assert!(client.auto_reconnect);
    }

    #[test]
    fn test_builder_custom() {
        let client = RtdsClient::builder()
            .host("wss://custom.example.com")
            .ping_interval(Duration::from_secs(10))
            .auto_reconnect(false)
            .build();

        assert_eq!(client.host, "wss://custom.example.com");
        assert_eq!(client.ping_interval, Duration::from_secs(10));
        assert!(!client.auto_reconnect);
    }

    #[tokio::test]
    async fn test_initial_status() {
        let client = RtdsClient::new();
        assert_eq!(client.status().await, ConnectionStatus::Disconnected);
    }
}
