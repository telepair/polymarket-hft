//! Polymarket Data API client.
//!
//! This module provides a client for interacting with the Polymarket Data API.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::data::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!     
//!     // Check API health
//!     let health = client.health().await?;
//!     println!("API status: {}", health.data);
//!     
//!     Ok(())
//! }
//! ```

mod client;
mod types;

pub use client::{Client, DEFAULT_BASE_URL};
pub use types::{
    Activity, ActivitySortBy, ActivityType, ClosedPosition, ClosedPositionSortBy, EventLiveVolume,
    GetTradesRequest, GetUserActivityRequest, GetUserClosedPositionsRequest,
    GetUserPositionsRequest, HealthStatus, Holder, MarketLiveVolume, MarketOpenInterest,
    MarketTopHolders, Position, PositionSortBy, SortDirection, Trade, TradeFilterType, TradeSide,
    UserPositionValue, UserTradedMarketsCount,
};
