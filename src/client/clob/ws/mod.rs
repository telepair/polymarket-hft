//! CLOB WebSocket client module.
//!
//! Provides real-time streaming of order book data, price changes, and user events
//! from the Polymarket CLOB WebSocket API.

mod client;
mod types;

pub use client::{
    ClobWsClient, ClobWsClientBuilder, ConnectionStatus, DEFAULT_PING_INTERVAL, DEFAULT_WS_URL,
};
pub use types::{
    BookMessage, Channel, LastTradePriceMessage, MakerOrder, MarketSubscription, OrderEventType,
    OrderMessage, Outcome, PriceChange, PriceChangeMessage, Side, TickSizeChangeMessage,
    TradeMessage, TradeStatus, UserSubscription, WsAuth, WsMessage, WsPriceLevel,
};
