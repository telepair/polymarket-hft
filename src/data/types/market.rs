//! Market data API types.

use serde::{Deserialize, Serialize};

/// Response item from the open interest endpoint.
///
/// Represents the open interest value for a specific market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOpenInterest {
    /// Market ID (0x-prefixed, 64 hex chars).
    pub market: String,
    /// The open interest value for this market.
    pub value: f64,
}

/// Market volume data within a live volume response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketLiveVolume {
    /// Market ID (0x-prefixed, 64 hex chars).
    pub market: String,
    /// The volume value for this market.
    pub value: f64,
}

/// Response from the live volume endpoint.
///
/// Contains the total volume and per-market breakdown for an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLiveVolume {
    /// Total volume across all markets.
    pub total: f64,
    /// Volume breakdown by market (None if no markets have volume).
    pub markets: Option<Vec<MarketLiveVolume>>,
}
