//! Data API types and utilities.
//!
//! This module contains all the types used by the Data API client.

mod activity;
mod holders;
mod market;
mod positions;
mod trades;
pub(crate) mod validation;

// Re-export all public types
pub use activity::{Activity, ActivitySortBy, ActivityType, GetUserActivityRequest};
pub use holders::{Holder, MarketTopHolders};
pub use market::{EventLiveVolume, MarketLiveVolume, MarketOpenInterest};
pub use positions::{
    ClosedPosition, ClosedPositionSortBy, GetUserClosedPositionsRequest, GetUserPositionsRequest,
    Position, PositionSortBy, UserPositionValue,
};
pub use trades::{GetTradesRequest, Trade, TradeFilterType, UserTradedMarketsCount};

// Re-export validation functions for internal use
pub(crate) use validation::{
    validate_event_id, validate_limit, validate_market_id, validate_min_balance, validate_user,
};

use serde::{Deserialize, Serialize};

/// Sort direction for queries.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum SortDirection {
    /// Ascending order.
    #[serde(rename = "ASC")]
    Asc,
    /// Descending order (default).
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
            _ => Err(format!(
                "Invalid sort direction: '{}'. Valid options: ASC, DESC",
                s
            )),
        }
    }
}

/// Trade side enum (BUY or SELL).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum TradeSide {
    /// Buy side.
    #[serde(rename = "BUY")]
    #[default]
    Buy,
    /// Sell side.
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
            _ => Err(format!(
                "Invalid trade side: '{}'. Valid options: BUY, SELL",
                s
            )),
        }
    }
}

/// Response from the health check endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Health status data, typically "OK".
    pub data: String,
}
