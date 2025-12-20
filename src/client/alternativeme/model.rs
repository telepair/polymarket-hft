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
    pub timestamp: i64,
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

/// Request parameters for /v2/ticker/ endpoint.
#[derive(Debug, Clone, Default)]
pub struct GetTickerRequest {
    /// Limit the number of returned results. Default is 100, use 0 for all.
    pub limit: Option<i32>,
    /// Starting position for pagination.
    pub start: Option<i32>,
    /// Currency conversion target (USD, EUR, BTC, etc.).
    pub convert: Option<String>,
    /// Response structure: "dictionary" or "array".
    pub structure: Option<String>,
    /// Sort field: id, rank, volume_24h, percent_change_24h, price, etc.
    pub sort: Option<String>,
}

/// Response for /v2/ticker/ endpoint (array structure).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerArrayResponse {
    pub data: Vec<Ticker>,
    pub metadata: Metadata,
}

/// Response for /v2/ticker/ endpoint (dictionary structure).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerDictResponse {
    pub data: HashMap<String, Ticker>,
    pub metadata: Metadata,
}

/// Request parameters for /v2/ticker/{id}/ endpoint.
#[derive(Debug, Clone, Default)]
pub struct GetTickerByIdRequest {
    /// Currency conversion target (USD, EUR, BTC, etc.).
    pub convert: Option<String>,
    /// Response structure: "dictionary" or "array".
    pub structure: Option<String>,
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

/// Request parameters for /v2/global/ endpoint.
#[derive(Debug, Clone, Default)]
pub struct GetGlobalRequest {
    /// Currency conversion target (USD, EUR, BTC, etc.).
    pub convert: Option<String>,
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

/// Metadata for Fear and Greed response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearAndGreedMetadata {
    #[serde(default)]
    pub error: Option<String>,
}

/// Response for /fng/ endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearAndGreedResponse {
    pub name: String,
    pub data: Vec<FearAndGreedData>,
    pub metadata: FearAndGreedMetadata,
}

/// Request parameters for /fng/ endpoint.
#[derive(Debug, Clone, Default)]
pub struct GetFearAndGreedRequest {
    /// Limit the number of returned results. Default is 1, use 0 for all.
    pub limit: Option<i32>,
    /// Response format: "json" or "csv".
    pub format: Option<String>,
    /// Date format: "us", "cn", "kr", "world".
    pub date_format: Option<String>,
}

// =============================================================================
// ToMetrics / ToState Implementations
// =============================================================================

use crate::engine::{Metric, StateEntry, ToMetrics, ToState};
use chrono::Utc;

impl ToMetrics for FearAndGreedResponse {
    fn to_metrics(&self, source: &str) -> Vec<Metric> {
        self.data
            .iter()
            .filter_map(|d| {
                d.value.parse::<f64>().ok().map(|value| {
                    Metric::new(source, "fear_and_greed_index", value)
                        .with_label("classification", &d.value_classification)
                })
            })
            .collect()
    }
}

impl ToState for FearAndGreedResponse {
    fn to_state(&self, source: &str) -> Vec<StateEntry> {
        self.data
            .first()
            .map(|d| {
                StateEntry::new(
                    format!("state:{}:fear_and_greed", source),
                    serde_json::json!({
                        "value": d.value,
                        "classification": d.value_classification,
                        "timestamp": d.timestamp,
                        "updated_at": Utc::now().to_rfc3339(),
                    }),
                )
            })
            .into_iter()
            .collect()
    }
}

impl ToMetrics for GlobalResponse {
    fn to_metrics(&self, source: &str) -> Vec<Metric> {
        let mut metrics = vec![
            Metric::new(
                source,
                "active_cryptocurrencies",
                self.data.active_cryptocurrencies as f64,
            ),
            Metric::new(source, "active_markets", self.data.active_markets as f64),
            Metric::new(
                source,
                "bitcoin_dominance",
                self.data.bitcoin_percentage_of_market_cap,
            ),
        ];

        // Add market cap and volume for each currency
        for (currency, quote) in &self.data.quotes {
            metrics.push(
                Metric::new(source, "total_market_cap", quote.total_market_cap)
                    .with_label("currency", currency),
            );
            metrics.push(
                Metric::new(source, "total_volume_24h", quote.total_volume_24h)
                    .with_label("currency", currency),
            );
        }

        metrics
    }
}

impl ToState for GlobalResponse {
    fn to_state(&self, source: &str) -> Vec<StateEntry> {
        vec![StateEntry::new(
            format!("state:{}:global", source),
            serde_json::json!({
                "active_cryptocurrencies": self.data.active_cryptocurrencies,
                "active_markets": self.data.active_markets,
                "bitcoin_dominance": self.data.bitcoin_percentage_of_market_cap,
                "quotes": self.data.quotes,
                "updated_at": Utc::now().to_rfc3339(),
            }),
        )]
    }
}

impl ToMetrics for TickerArrayResponse {
    fn to_metrics(&self, source: &str) -> Vec<Metric> {
        self.data
            .iter()
            .flat_map(|ticker| {
                ticker.quotes.iter().flat_map(|(currency, quote)| {
                    vec![
                        Metric::new(source, "price", quote.price)
                            .with_label("symbol", &ticker.symbol)
                            .with_label("currency", currency),
                        Metric::new(source, "market_cap", quote.market_cap)
                            .with_label("symbol", &ticker.symbol)
                            .with_label("currency", currency),
                        Metric::new(source, "volume_24h", quote.volume_24h)
                            .with_label("symbol", &ticker.symbol)
                            .with_label("currency", currency),
                    ]
                })
            })
            .collect()
    }
}

impl ToState for TickerArrayResponse {
    fn to_state(&self, source: &str) -> Vec<StateEntry> {
        self.data
            .iter()
            .map(|ticker| {
                StateEntry::new(
                    format!("state:{}:ticker:{}", source, ticker.symbol.to_lowercase()),
                    serde_json::json!({
                        "id": ticker.id,
                        "name": ticker.name,
                        "symbol": ticker.symbol,
                        "rank": ticker.rank,
                        "quotes": ticker.quotes,
                        "updated_at": Utc::now().to_rfc3339(),
                    }),
                )
            })
            .collect()
    }
}
