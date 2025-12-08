//! CLOB WebSocket client implementation.

use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::interval;
use tokio_tungstenite::tungstenite::protocol::Message as TungsteniteMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{debug, error, info, trace, warn};

use super::types::{Channel, MarketSubscription, UserSubscription, WsAuth, WsMessage};
use crate::error::{PolymarketError, Result};

/// Default WebSocket server URL.
pub const DEFAULT_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com";

/// Default ping interval (10 seconds per docs).
pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(10);

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, TungsteniteMessage>;
type WsReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Connection status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Attempting to connect.
    Connecting,
    /// Successfully connected.
    Connected,
    /// Disconnected from server.
    Disconnected,
}

/// Builder for configuring `ClobWsClient`.
#[derive(Debug, Clone)]
pub struct ClobWsClientBuilder {
    base_url: String,
    ping_interval: Duration,
    auto_reconnect: bool,
}

impl Default for ClobWsClientBuilder {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_WS_URL.to_string(),
            ping_interval: DEFAULT_PING_INTERVAL,
            auto_reconnect: true,
        }
    }
}

impl ClobWsClientBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the WebSocket base URL.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
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

    /// Builds the `ClobWsClient`.
    pub fn build(self) -> ClobWsClient {
        ClobWsClient {
            base_url: self.base_url,
            ping_interval: self.ping_interval,
            auto_reconnect: self.auto_reconnect,
            channel: None,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            writer: Arc::new(Mutex::new(None)),
            reader: Arc::new(Mutex::new(None)),
            market_subscription: Arc::new(Mutex::new(None)),
            user_subscription: Arc::new(Mutex::new(None)),
        }
    }
}

/// Polymarket CLOB WebSocket client.
///
/// Provides real-time streaming of order book data, price changes, and user events.
#[derive(Debug)]
pub struct ClobWsClient {
    base_url: String,
    ping_interval: Duration,
    auto_reconnect: bool,
    channel: Option<Channel>,
    status: Arc<Mutex<ConnectionStatus>>,
    writer: Arc<Mutex<Option<WsWriter>>>,
    reader: Arc<Mutex<Option<WsReader>>>,
    market_subscription: Arc<Mutex<Option<MarketSubscription>>>,
    user_subscription: Arc<Mutex<Option<UserSubscription>>>,
}

impl Clone for ClobWsClient {
    fn clone(&self) -> Self {
        Self {
            base_url: self.base_url.clone(),
            ping_interval: self.ping_interval,
            auto_reconnect: self.auto_reconnect,
            channel: self.channel,
            status: Arc::clone(&self.status),
            writer: Arc::clone(&self.writer),
            reader: Arc::clone(&self.reader),
            market_subscription: Arc::clone(&self.market_subscription),
            user_subscription: Arc::clone(&self.user_subscription),
        }
    }
}

impl Default for ClobWsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ClobWsClient {
    /// Creates a new `ClobWsClient` with default settings.
    pub fn new() -> Self {
        ClobWsClientBuilder::default().build()
    }

    /// Returns a builder for configuring the client.
    pub fn builder() -> ClobWsClientBuilder {
        ClobWsClientBuilder::new()
    }

    /// Returns the current connection status.
    pub async fn status(&self) -> ConnectionStatus {
        *self.status.lock().await
    }

    /// Returns the current channel, if connected.
    pub fn channel(&self) -> Option<Channel> {
        self.channel
    }

    /// Connects and subscribes to the market channel.
    ///
    /// # Arguments
    /// * `asset_ids` - Token IDs to subscribe to
    pub async fn subscribe_market(&mut self, asset_ids: Vec<String>) -> Result<()> {
        let subscription = MarketSubscription::new(asset_ids);
        *self.market_subscription.lock().await = Some(subscription.clone());

        self.connect_to_channel(Channel::Market).await?;
        self.send_subscription(&subscription).await
    }

    /// Connects and subscribes to the user channel.
    ///
    /// # Arguments
    /// * `market_ids` - Market IDs (condition IDs) to subscribe to
    /// * `auth` - Authentication credentials
    pub async fn subscribe_user(&mut self, market_ids: Vec<String>, auth: WsAuth) -> Result<()> {
        let subscription = UserSubscription::new(market_ids, auth);
        *self.user_subscription.lock().await = Some(subscription.clone());

        self.connect_to_channel(Channel::User).await?;
        self.send_subscription(&subscription).await
    }

    /// Connects to a specific channel.
    async fn connect_to_channel(&mut self, channel: Channel) -> Result<()> {
        let channel_path = match channel {
            Channel::Market => "market",
            Channel::User => "user",
        };

        let url = format!("{}/ws/{}", self.base_url, channel_path);
        info!("Connecting to CLOB WebSocket at {}", url);
        self.set_status(ConnectionStatus::Connecting).await;

        let (ws_stream, response) = connect_async(&url)
            .await
            .map_err(|e| PolymarketError::websocket(format!("Connection failed: {e}")))?;

        debug!(
            "WebSocket handshake completed with status: {}",
            response.status()
        );

        let (writer, reader) = ws_stream.split();
        *self.writer.lock().await = Some(writer);
        *self.reader.lock().await = Some(reader);
        self.channel = Some(channel);

        self.set_status(ConnectionStatus::Connected).await;
        info!("Connected to CLOB WebSocket ({})", channel_path);

        // Start ping loop
        self.start_ping_loop();

        Ok(())
    }

    /// Sends a subscription message.
    async fn send_subscription<T: serde::Serialize>(&self, subscription: &T) -> Result<()> {
        let json = serde_json::to_string(subscription)?;
        trace!("Sending subscription: {}", json);
        self.send_text(json).await
    }

    /// Disconnects from the WebSocket server.
    pub async fn disconnect(&mut self) {
        info!("Disconnecting from CLOB WebSocket");

        if let Some(mut writer) = self.writer.lock().await.take() {
            let _ = writer.close().await;
        }
        *self.reader.lock().await = None;
        self.channel = None;

        self.set_status(ConnectionStatus::Disconnected).await;
        info!("Disconnected from CLOB WebSocket");
    }

    /// Receives the next message from the WebSocket.
    ///
    /// Returns `None` if the connection is closed.
    pub async fn next_message(&mut self) -> Option<WsMessage> {
        loop {
            let msg = {
                let mut reader_guard = self.reader.lock().await;
                let reader = reader_guard.as_mut()?;
                reader.next().await
            };

            match msg {
                Some(Ok(TungsteniteMessage::Text(text))) => {
                    // Skip pong responses
                    if text == "PONG" || text.is_empty() {
                        trace!("Received PONG");
                        continue;
                    }

                    // Parse JSON
                    match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(value) => {
                            let ws_msg = WsMessage::from_json(value);
                            trace!("Received message: type={}", ws_msg.event_type());
                            return Some(ws_msg);
                        }
                        Err(e) => {
                            warn!("Failed to parse message: {e}");
                            debug!("Raw message: {}", text);
                        }
                    }
                }
                Some(Ok(TungsteniteMessage::Ping(data))) => {
                    trace!("Received ping, sending pong");
                    if let Err(e) = self.send_pong(data.to_vec()).await {
                        error!("Failed to send pong: {e}");
                    }
                }
                Some(Ok(TungsteniteMessage::Pong(_))) => {
                    trace!("Received pong");
                }
                Some(Ok(TungsteniteMessage::Close(frame))) => {
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
            .send(TungsteniteMessage::Text(text.into()))
            .await
            .map_err(|e| PolymarketError::websocket(format!("Send failed: {e}")))?;

        Ok(())
    }

    /// Sends a pong response.
    async fn send_pong(&self, data: Vec<u8>) -> Result<()> {
        let mut writer_guard = self.writer.lock().await;
        if let Some(writer) = writer_guard.as_mut() {
            writer
                .send(TungsteniteMessage::Pong(data.into()))
                .await
                .map_err(|e| PolymarketError::websocket(format!("Pong failed: {e}")))?;
        }
        Ok(())
    }

    /// Sends a ping message (text "PING" per Polymarket docs).
    async fn send_ping(&self) -> Result<()> {
        self.send_text("PING".to_string()).await
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

        let channel = self
            .channel
            .ok_or_else(|| PolymarketError::websocket("Cannot reconnect: no previous channel"))?;

        // Exponential backoff: try up to 5 times
        for attempt in 1..=5 {
            let delay = Duration::from_millis(500 * 2u64.pow(attempt - 1));
            info!("Reconnect attempt {} in {:?}", attempt, delay);
            tokio::time::sleep(delay).await;

            // Reconnect to channel
            if let Err(e) = self.connect_to_channel(channel).await {
                warn!("Reconnect attempt {} failed: {e}", attempt);
                continue;
            }

            // Resend subscription
            match channel {
                Channel::Market => {
                    if let Some(sub) = self.market_subscription.lock().await.clone()
                        && let Err(e) = self.send_subscription(&sub).await
                    {
                        warn!("Resubscribe failed: {e}");
                        continue;
                    }
                }
                Channel::User => {
                    if let Some(sub) = self.user_subscription.lock().await.clone()
                        && let Err(e) = self.send_subscription(&sub).await
                    {
                        warn!("Resubscribe failed: {e}");
                        continue;
                    }
                }
            }

            return Ok(());
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
        let client = ClobWsClient::builder().build();
        assert_eq!(client.base_url, DEFAULT_WS_URL);
        assert_eq!(client.ping_interval, DEFAULT_PING_INTERVAL);
        assert!(client.auto_reconnect);
    }

    #[test]
    fn test_builder_custom() {
        let client = ClobWsClient::builder()
            .base_url("wss://custom.example.com")
            .ping_interval(Duration::from_secs(5))
            .auto_reconnect(false)
            .build();

        assert_eq!(client.base_url, "wss://custom.example.com");
        assert_eq!(client.ping_interval, Duration::from_secs(5));
        assert!(!client.auto_reconnect);
    }

    #[tokio::test]
    async fn test_initial_status() {
        let client = ClobWsClient::new();
        assert_eq!(client.status().await, ConnectionStatus::Disconnected);
        assert!(client.channel().is_none());
    }
}
