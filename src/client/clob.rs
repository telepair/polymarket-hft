//! Polymarket CLOB (Central Limit Order Book) API client.
//!
//! This module provides a client for interacting with the Polymarket CLOB API,
//! which provides access to order book data, pricing information, and spreads.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::client::clob::{Client, Side};
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

mod client;
mod orderbook;
mod pricing;
mod spreads;
pub mod ws;

pub use client::{Client, DEFAULT_BASE_URL};
pub use orderbook::{GetOrderBooksRequestItem, OrderBookSummary, PriceLevel};
pub use pricing::{
    GetPriceHistoryRequest, MarketPrice, MarketPriceRequest, MidpointPrice, PriceHistory,
    PriceHistoryInterval, PriceHistoryPoint, Side,
};
pub use spreads::SpreadRequest;
