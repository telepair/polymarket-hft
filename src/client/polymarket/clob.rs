//! Polymarket CLOB (Central Limit Order Book) API client.
//!
//! This module provides a client for interacting with the Polymarket CLOB API,
//! which provides access to order book data, pricing information, and spreads.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::client::polymarket::clob::{Client, Side};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!
//!     // Get market price for a token
//!     let price = client.get_market_price("token_id", Side::Buy).await?;
//!     println!("Price: {}", price.price);
//!
//!     Ok(())
//! }
//! ```

mod auth;
mod client;
mod markets;
pub mod order_utils;
pub mod orderbook;
mod pricing;
mod spreads;
mod token_info;
mod trading;
mod types;
pub mod ws;

pub use client::{Client, DEFAULT_BASE_URL};
pub use markets::{
    GetMarketsRequest, Market, MarketToken, MarketTradeEvent, MarketsPaginatedResponse,
    SimplifiedMarket,
};
pub use order_utils::{ExchangeOrderBuilder, OrderData, OrderSide, SignatureType, SignedOrder};
pub use orderbook::{GetOrderBooksRequestItem, OrderBookSummary, PriceLevel};
pub use pricing::{
    GetPriceHistoryRequest, MarketPrice, MarketPriceRequest, MidpointPrice, PriceHistory,
    PriceHistoryInterval, PriceHistoryPoint, Side,
};
pub use spreads::SpreadRequest;
pub use trading::TradingClient;
pub use types::{
    ApiKeyCreds, ApiKeyRaw, ApiKeysResponse, AssetType, BalanceAllowance, BalanceAllowanceParams,
    BanStatus, Chain, CreateOrderOptions, OpenOrder, OpenOrderParams, OrderType, PostOrdersArgs,
    TickSize, Trade, TradeParams, TradesPaginatedResponse, UserLimitOrder, UserMarketOrder,
};
