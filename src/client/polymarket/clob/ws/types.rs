//! WebSocket message types for CLOB channels.

use serde::{Deserialize, Serialize};

// =============================================================================
// Common Types
// =============================================================================

/// WebSocket channel type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    /// Market data channel (order book, prices, trades).
    Market,
    /// User-specific channel (orders, trades) - requires auth.
    User,
}

/// Authentication credentials for user channel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WsAuth {
    /// API key.
    pub api_key: String,
    /// API secret.
    pub secret: String,
    /// API passphrase.
    pub passphrase: String,
}

impl WsAuth {
    /// Creates new authentication credentials.
    pub fn new(
        api_key: impl Into<String>,
        secret: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            secret: secret.into(),
            passphrase: passphrase.into(),
        }
    }

    /// Creates authentication from environment variables.
    ///
    /// Reads from:
    /// - `POLY_API_KEY`
    /// - `POLY_API_SECRET`
    /// - `POLY_PASSPHRASE`
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("POLY_API_KEY").ok()?;
        let secret = std::env::var("POLY_API_SECRET").ok()?;
        let passphrase = std::env::var("POLY_PASSPHRASE").ok()?;
        Some(Self::new(api_key, secret, passphrase))
    }
}

/// Order side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    /// Buy order.
    Buy,
    /// Sell order.
    Sell,
}

/// Order outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Outcome {
    /// Yes outcome.
    Yes,
    /// No outcome.
    No,
}

// =============================================================================
// Subscription Messages
// =============================================================================

/// Market channel subscription request.
#[derive(Debug, Clone, Serialize)]
pub struct MarketSubscription {
    /// Asset IDs (token IDs) to subscribe to.
    pub assets_ids: Vec<String>,
    /// Channel type.
    #[serde(rename = "type")]
    pub channel_type: Channel,
}

impl MarketSubscription {
    /// Creates a new market subscription.
    pub fn new(asset_ids: Vec<String>) -> Self {
        Self {
            assets_ids: asset_ids,
            channel_type: Channel::Market,
        }
    }
}

/// User channel subscription request.
#[derive(Debug, Clone, Serialize)]
pub struct UserSubscription {
    /// Market IDs (condition IDs) to subscribe to.
    pub markets: Vec<String>,
    /// Channel type.
    #[serde(rename = "type")]
    pub channel_type: Channel,
    /// Authentication credentials.
    pub auth: WsAuth,
}

impl UserSubscription {
    /// Creates a new user subscription.
    pub fn new(market_ids: Vec<String>, auth: WsAuth) -> Self {
        Self {
            markets: market_ids,
            channel_type: Channel::User,
            auth,
        }
    }
}

// =============================================================================
// Market Channel Messages
// =============================================================================

/// Order book price level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsPriceLevel {
    /// Price as string.
    pub price: String,
    /// Size as string.
    pub size: String,
}

/// Order book snapshot message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMessage {
    /// Event type (always "book").
    pub event_type: String,
    /// Asset ID (token ID).
    pub asset_id: String,
    /// Market ID (condition ID).
    pub market: String,
    /// Bid price levels.
    pub bids: Vec<WsPriceLevel>,
    /// Ask price levels.
    pub asks: Vec<WsPriceLevel>,
    /// Timestamp.
    pub timestamp: String,
    /// Hash for verification.
    pub hash: String,
}

/// Individual price change entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChange {
    /// Asset ID (token ID).
    pub asset_id: String,
    /// Price as string.
    pub price: String,
    /// Size as string.
    pub size: String,
    /// Order side.
    pub side: Side,
    /// Hash for verification.
    pub hash: String,
    /// Best bid price.
    pub best_bid: String,
    /// Best ask price.
    pub best_ask: String,
}

/// Price change message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChangeMessage {
    /// Event type (always "price_change").
    pub event_type: String,
    /// Market ID (condition ID).
    pub market: String,
    /// List of price changes.
    pub price_changes: Vec<PriceChange>,
    /// Timestamp.
    pub timestamp: String,
}

/// Tick size change message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickSizeChangeMessage {
    /// Event type (always "tick_size_change").
    pub event_type: String,
    /// Asset ID (token ID).
    pub asset_id: String,
    /// Market ID (condition ID).
    pub market: String,
    /// Old tick size.
    pub old_tick_size: String,
    /// New tick size.
    pub new_tick_size: String,
    /// Timestamp.
    pub timestamp: String,
}

/// Last trade price message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTradePriceMessage {
    /// Event type (always "last_trade_price").
    pub event_type: String,
    /// Asset ID (token ID).
    pub asset_id: String,
    /// Market ID (condition ID).
    pub market: String,
    /// Trade price.
    pub price: String,
    /// Trade side.
    pub side: Side,
    /// Trade size.
    pub size: String,
    /// Fee rate in basis points.
    pub fee_rate_bps: String,
    /// Timestamp.
    pub timestamp: String,
}

// =============================================================================
// User Channel Messages
// =============================================================================

/// Trade status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TradeStatus {
    /// Trade matched.
    Matched,
    /// Trade mined on-chain.
    Mined,
    /// Trade confirmed.
    Confirmed,
    /// Trade retrying.
    Retrying,
    /// Trade failed.
    Failed,
}

/// Maker order in a trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakerOrder {
    /// Asset ID (token ID).
    pub asset_id: String,
    /// Amount matched.
    pub matched_amount: String,
    /// Order ID.
    pub order_id: String,
    /// Outcome.
    pub outcome: Outcome,
    /// Owner ID.
    pub owner: String,
    /// Price.
    pub price: String,
}

/// Trade message from user channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeMessage {
    /// Event type (always "trade").
    pub event_type: String,
    /// Trade ID.
    pub id: String,
    /// Asset ID (token ID).
    pub asset_id: String,
    /// Market ID (condition ID).
    pub market: String,
    /// Trade status.
    pub status: TradeStatus,
    /// Trade side.
    pub side: Side,
    /// Trade outcome.
    pub outcome: Outcome,
    /// Trade size.
    pub size: String,
    /// Trade price.
    pub price: String,
    /// Maker orders involved.
    pub maker_orders: Vec<MakerOrder>,
    /// Taker order ID.
    pub taker_order_id: String,
    /// Owner ID.
    pub owner: String,
    /// Trade owner ID.
    pub trade_owner: String,
    /// Match timestamp.
    pub matchtime: String,
    /// Last update timestamp.
    pub last_update: String,
    /// Message timestamp.
    pub timestamp: String,
    /// Trade type.
    #[serde(rename = "type")]
    pub trade_type: String,
}

/// Order event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderEventType {
    /// Order placed.
    Placement,
    /// Order updated (partially filled).
    Update,
    /// Order cancelled.
    Cancellation,
}

/// Order message from user channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderMessage {
    /// Event type (always "order").
    pub event_type: String,
    /// Order ID.
    pub id: String,
    /// Asset ID (token ID).
    pub asset_id: String,
    /// Market ID (condition ID).
    pub market: String,
    /// Order event type.
    #[serde(rename = "type")]
    pub order_type: OrderEventType,
    /// Order side.
    pub side: Side,
    /// Order outcome.
    pub outcome: Outcome,
    /// Original order size.
    pub original_size: String,
    /// Size matched so far.
    pub size_matched: String,
    /// Order price.
    pub price: String,
    /// Owner ID.
    pub owner: String,
    /// Order owner ID.
    pub order_owner: String,
    /// Associated trade IDs.
    pub associate_trades: Option<Vec<String>>,
    /// Timestamp.
    pub timestamp: String,
}

// =============================================================================
// Unified Message Type
// =============================================================================

/// Parsed WebSocket message from CLOB.
#[derive(Debug, Clone)]
pub enum WsMessage {
    /// Order book snapshot.
    Book(BookMessage),
    /// Price level change.
    PriceChange(PriceChangeMessage),
    /// Tick size change.
    TickSizeChange(TickSizeChangeMessage),
    /// Last trade price.
    LastTradePrice(LastTradePriceMessage),
    /// User trade event.
    Trade(TradeMessage),
    /// User order event.
    Order(OrderMessage),
    /// Unknown/unparsed message.
    Unknown(serde_json::Value),
}

impl WsMessage {
    /// Attempts to parse a JSON value into a typed message.
    pub fn from_json(value: serde_json::Value) -> Self {
        let event_type = value
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match event_type {
            "book" => serde_json::from_value(value.clone())
                .map(WsMessage::Book)
                .unwrap_or(WsMessage::Unknown(value)),
            "price_change" => serde_json::from_value(value.clone())
                .map(WsMessage::PriceChange)
                .unwrap_or(WsMessage::Unknown(value)),
            "tick_size_change" => serde_json::from_value(value.clone())
                .map(WsMessage::TickSizeChange)
                .unwrap_or(WsMessage::Unknown(value)),
            "last_trade_price" => serde_json::from_value(value.clone())
                .map(WsMessage::LastTradePrice)
                .unwrap_or(WsMessage::Unknown(value)),
            "trade" => serde_json::from_value(value.clone())
                .map(WsMessage::Trade)
                .unwrap_or(WsMessage::Unknown(value)),
            "order" => serde_json::from_value(value.clone())
                .map(WsMessage::Order)
                .unwrap_or(WsMessage::Unknown(value)),
            _ => WsMessage::Unknown(value),
        }
    }

    /// Returns the event type as a string slice.
    pub fn event_type(&self) -> &str {
        match self {
            WsMessage::Book(_) => "book",
            WsMessage::PriceChange(_) => "price_change",
            WsMessage::TickSizeChange(_) => "tick_size_change",
            WsMessage::LastTradePrice(_) => "last_trade_price",
            WsMessage::Trade(_) => "trade",
            WsMessage::Order(_) => "order",
            WsMessage::Unknown(_) => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_auth_from_env() {
        // This test just verifies the function doesn't panic
        // Actual env var testing would need to be an integration test
        let _auth = WsAuth::from_env();
    }

    #[test]
    fn test_market_subscription_serialization() {
        let sub = MarketSubscription::new(vec!["token123".to_string()]);
        let json = serde_json::to_string(&sub).unwrap();
        assert!(json.contains(r#""assets_ids":["token123"]"#));
        assert!(json.contains(r#""type":"market""#));
    }

    #[test]
    fn test_book_message_deserialization() {
        let json = r#"{
            "event_type": "book",
            "asset_id": "123",
            "market": "0xabc",
            "bids": [{"price": "0.5", "size": "100"}],
            "asks": [{"price": "0.6", "size": "50"}],
            "timestamp": "1234567890",
            "hash": "0x..."
        }"#;

        let msg: BookMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.event_type, "book");
        assert_eq!(msg.asset_id, "123");
        assert_eq!(msg.bids.len(), 1);
        assert_eq!(msg.asks.len(), 1);
    }

    #[test]
    fn test_ws_message_from_json() {
        let json = serde_json::json!({
            "event_type": "book",
            "asset_id": "123",
            "market": "0xabc",
            "bids": [],
            "asks": [],
            "timestamp": "0",
            "hash": "0x"
        });

        let msg = WsMessage::from_json(json);
        assert_eq!(msg.event_type(), "book");
    }

    #[test]
    fn test_price_change_message_deserialization() {
        let json = r#"{
            "event_type": "price_change",
            "market": "0xabc",
            "price_changes": [{
                "asset_id": "123",
                "price": "0.5",
                "size": "100",
                "side": "BUY",
                "hash": "abc123",
                "best_bid": "0.49",
                "best_ask": "0.51"
            }],
            "timestamp": "1234567890"
        }"#;

        let msg: PriceChangeMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.event_type, "price_change");
        assert_eq!(msg.price_changes.len(), 1);
        assert_eq!(msg.price_changes[0].side, Side::Buy);
    }
}
