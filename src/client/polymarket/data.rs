//! Polymarket Data API client.
//!
//! This module provides a client for interacting with the Polymarket Data API.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::client::polymarket::data::Client;
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

mod activity;
mod client;
mod holders;
mod market;
mod positions;
mod trades;
mod validation;

pub use activity::{Activity, ActivitySortBy, ActivityType, GetUserActivityRequest};
pub use client::{Client, DEFAULT_BASE_URL};
pub use holders::{Holder, MarketTopHolders};
pub use market::{EventLiveVolume, MarketLiveVolume, MarketOpenInterest};
pub use positions::{
    ClosedPosition, ClosedPositionSortBy, GetUserClosedPositionsRequest, GetUserPositionsRequest,
    Position, PositionSortBy, UserPositionValue,
};
pub use trades::{GetTradesRequest, Trade, TradeFilterType, UserTradedMarketsCount};

use serde::{Deserialize, Serialize};

// Re-export validation functions for internal use
pub(crate) use validation::{
    validate_event_id, validate_limit, validate_market_id, validate_min_balance, validate_user,
};

/// Sort direction for queries.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum SortDirection {
    #[serde(rename = "ASC")]
    Asc,
    #[serde(rename = "DESC")]
    #[default]
    Desc,
}

impl std::fmt::Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortDirection::Asc => write!(f, "ASC"),
            SortDirection::Desc => write!(f, "DESC"),
        }
    }
}

impl std::str::FromStr for SortDirection {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ASC" => Ok(SortDirection::Asc),
            "DESC" => Ok(SortDirection::Desc),
            _ => Err(format!("Invalid sort direction: '{}'", s)),
        }
    }
}

/// Trade side enum (BUY or SELL).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum TradeSide {
    #[serde(rename = "BUY")]
    #[default]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

impl std::fmt::Display for TradeSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeSide::Buy => write!(f, "BUY"),
            TradeSide::Sell => write!(f, "SELL"),
        }
    }
}

impl std::str::FromStr for TradeSide {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BUY" => Ok(TradeSide::Buy),
            "SELL" => Ok(TradeSide::Sell),
            _ => Err(format!("Invalid trade side: '{}'", s)),
        }
    }
}

/// Response from the health check endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub data: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_direction_display() {
        assert_eq!(SortDirection::Asc.to_string(), "ASC");
        assert_eq!(SortDirection::Desc.to_string(), "DESC");
    }

    #[test]
    fn test_sort_direction_from_str() {
        assert!(matches!(
            "ASC".parse::<SortDirection>(),
            Ok(SortDirection::Asc)
        ));
        assert!(matches!(
            "DESC".parse::<SortDirection>(),
            Ok(SortDirection::Desc)
        ));
        assert!(matches!(
            "asc".parse::<SortDirection>(),
            Ok(SortDirection::Asc)
        ));
        assert!(matches!(
            "desc".parse::<SortDirection>(),
            Ok(SortDirection::Desc)
        ));
        assert!("invalid".parse::<SortDirection>().is_err());
    }

    #[test]
    fn test_trade_side_display() {
        assert_eq!(TradeSide::Buy.to_string(), "BUY");
        assert_eq!(TradeSide::Sell.to_string(), "SELL");
    }

    #[test]
    fn test_trade_side_from_str() {
        assert!(matches!("BUY".parse::<TradeSide>(), Ok(TradeSide::Buy)));
        assert!(matches!("SELL".parse::<TradeSide>(), Ok(TradeSide::Sell)));
        assert!(matches!("buy".parse::<TradeSide>(), Ok(TradeSide::Buy)));
        assert!(matches!("sell".parse::<TradeSide>(), Ok(TradeSide::Sell)));
        assert!("invalid".parse::<TradeSide>().is_err());
    }
}
