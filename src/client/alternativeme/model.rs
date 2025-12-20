//! Alternative.me API data models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Alternative.me API error.
#[derive(Debug, Error)]
pub enum AlternativeMeError {
    /// HTTP/network error from reqwest middleware.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest_middleware::Error),

    /// HTTP/network error from reqwest (e.g., JSON parsing).
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error: {0}")]
    Api(String),
}

// =============================================================================
// Common Types
// =============================================================================

/// Response metadata common to most endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub num_cryptocurrencies: Option<i32>,
    #[serde(default)]
    pub error: Option<String>,
}

// =============================================================================
// Ticker (/v2/ticker/)
// =============================================================================

/// Price quote in a specific currency.
/// Note: API returns both percentage_change_* and percent_change_* fields.
/// We use percent_change_* and ignore the duplicates via flatten.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerQuote {
    pub price: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    #[serde(default)]
    pub percent_change_1h: Option<f64>,
    #[serde(default)]
    pub percent_change_24h: Option<f64>,
    #[serde(default)]
    pub percent_change_7d: Option<f64>,
    /// Captures extra/duplicate fields from API (percentage_change_* etc.)
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Single cryptocurrency ticker data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub id: i64,
    pub name: String,
    pub symbol: String,
    pub website_slug: String,
    pub rank: i32,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub quotes: HashMap<String, TickerQuote>,
    pub last_updated: i64,
}

/// Response for /v2/ticker/ endpoint (array structure).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerArrayResponse {
    pub data: Vec<Ticker>,
    pub metadata: Metadata,
}

// =============================================================================
// Global (/v2/global/)
// =============================================================================

/// Global market quote in a specific currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalQuote {
    pub total_market_cap: f64,
    pub total_volume_24h: f64,
}

/// Global market metrics data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalData {
    pub active_cryptocurrencies: i32,
    pub active_markets: i32,
    pub bitcoin_percentage_of_market_cap: f64,
    pub quotes: HashMap<String, GlobalQuote>,
    pub last_updated: i64,
}

/// Response for /v2/global/ endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalResponse {
    pub data: GlobalData,
    pub metadata: Metadata,
}

// =============================================================================
// Fear and Greed Index (/fng/)
// =============================================================================

/// Fear and Greed Index data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearAndGreedData {
    /// Index value (0-100).
    pub value: String,
    /// Classification: "Extreme Fear", "Fear", "Neutral", "Greed", "Extreme Greed".
    pub value_classification: String,
    /// Unix timestamp.
    pub timestamp: String,
    /// Seconds until next update (only present for latest value).
    #[serde(default)]
    pub time_until_update: Option<String>,
}

/// Response for /fng/ endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearAndGreedResponse {
    pub name: String,
    pub data: Vec<FearAndGreedData>,
    pub metadata: Metadata,
}
