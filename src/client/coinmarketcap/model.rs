use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// CoinMarketCap API error.
#[derive(Debug, Error)]
pub enum CmcError {
    /// HTTP/network error from reqwest middleware.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest_middleware::Error),

    /// HTTP/network error from reqwest (e.g., JSON parsing).
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    /// API returned an error response (error_code != 0).
    #[error("API error {code}: {message}")]
    Api { code: i32, message: String },
}

/// Helper to deserialize error_code that may be either string or integer.
fn deserialize_error_code<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i32),
    }

    match StringOrInt::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse().map_err(de::Error::custom),
        StringOrInt::Int(i) => Ok(i),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub timestamp: String,
    #[serde(deserialize_with = "deserialize_error_code")]
    pub error_code: i32,
    #[serde(default)]
    pub error_message: Option<String>,
    pub elapsed: i32,
    pub credit_count: i32,
    #[serde(default)]
    pub notice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub id: i32,
    pub name: String,
    pub symbol: String,
    pub slug: String,
    pub token_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub price: Option<f64>,
    pub volume_24h: Option<f64>,
    pub volume_change_24h: Option<f64>,
    pub percent_change_1h: Option<f64>,
    pub percent_change_24h: Option<f64>,
    pub percent_change_7d: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_dominance: Option<f64>,
    pub fully_diluted_market_cap: Option<f64>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cryptocurrency {
    pub id: i32,
    pub name: String,
    pub symbol: String,
    pub slug: String,
    pub num_market_pairs: Option<i32>,
    pub date_added: Option<String>,
    pub tags: Option<Vec<String>>,
    pub max_supply: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub infinite_supply: Option<bool>,
    pub platform: Option<Platform>,
    pub cmc_rank: Option<i32>,
    pub self_reported_circulating_supply: Option<f64>,
    pub self_reported_market_cap: Option<f64>,
    pub tvl_ratio: Option<f64>,
    pub last_updated: String,
    pub quote: HashMap<String, Quote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListingsLatestResponse {
    pub status: Status,
    pub data: Vec<Cryptocurrency>,
}

#[derive(Debug, Clone, Default)]
pub struct GetListingsLatestRequest {
    pub start: Option<i32>,
    pub limit: Option<i32>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub market_cap_min: Option<f64>,
    pub market_cap_max: Option<f64>,
    pub volume_24h_min: Option<f64>,
    pub volume_24h_max: Option<f64>,
    pub circulating_supply_min: Option<f64>,
    pub circulating_supply_max: Option<f64>,
    pub percent_change_24h_min: Option<f64>,
    pub percent_change_24h_max: Option<f64>,
    pub convert: Option<String>,
    pub convert_id: Option<String>,
    pub sort: Option<String>,
    pub sort_dir: Option<String>,
    pub cryptocurrency_type: Option<String>,
    /// Tag filter: "all", "defi", "filesharing", etc.
    pub tag: Option<String>,
    /// Auxiliary fields to include in response.
    pub aux: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalQuote {
    pub total_market_cap: f64,
    pub total_volume_24h: f64,
    pub total_volume_24h_reported: f64,
    pub altcoin_volume_24h: f64,
    pub altcoin_market_cap: f64,
    pub defi_volume_24h: Option<f64>,
    pub defi_market_cap: Option<f64>,
    pub defi_24h_percentage_change: Option<f64>,
    pub stablecoin_volume_24h: Option<f64>,
    pub stablecoin_market_cap: Option<f64>,
    pub stablecoin_24h_percentage_change: Option<f64>,
    pub der_volume_24h: Option<f64>,
    pub der_24h_percentage_change: Option<f64>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetricsQuotesLatestData {
    pub active_cryptocurrencies: i32,
    pub total_cryptocurrencies: i32,
    pub active_market_pairs: i32,
    pub active_exchanges: i32,
    pub total_exchanges: i32,
    pub eth_dominance: f64,
    pub btc_dominance: f64,
    pub eth_dominance_yesterday: Option<f64>,
    pub btc_dominance_yesterday: Option<f64>,
    pub defi_volume_24h_reported: Option<f64>,
    pub stablecoin_volume_24h_reported: Option<f64>,
    pub der_volume_24h_reported: Option<f64>,
    pub quote: HashMap<String, GlobalQuote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetricsQuotesLatestResponse {
    pub status: Status,
    pub data: GlobalMetricsQuotesLatestData,
}

#[derive(Debug, Clone, Default)]
pub struct GetGlobalMetricsQuotesLatestRequest {
    pub convert: Option<String>,
    pub convert_id: Option<String>,
}

// === Fear and Greed Index ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearAndGreed {
    pub value: f64,
    pub value_classification: String,
    #[serde(alias = "timestamp", alias = "update_time")]
    pub update_time: String,
    #[serde(default)]
    pub time_until_update: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearAndGreedResponse {
    pub status: Status,
    pub data: FearAndGreed,
}

/// Request parameters for Fear and Greed Index (no parameters required).
#[derive(Debug, Clone, Default)]
pub struct GetFearAndGreedLatestRequest {}

// === API Key Info ===

/// API plan information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanInfo {
    /// Daily credit limit (may not be present in some plans).
    #[serde(default)]
    pub credit_limit_daily: Option<i64>,
    /// Timestamp when daily credits reset.
    #[serde(default)]
    pub credit_limit_daily_reset: Option<String>,
    /// Monthly credit limit.
    #[serde(default)]
    pub credit_limit_monthly: Option<i64>,
    /// Timestamp when monthly credits reset (human-readable).
    #[serde(default)]
    pub credit_limit_monthly_reset: Option<String>,
    /// Timestamp when monthly credits reset (ISO format).
    #[serde(default)]
    pub credit_limit_monthly_reset_timestamp: Option<String>,
    /// Rate limit per minute.
    #[serde(default)]
    pub rate_limit_minute: Option<i32>,
}

/// API usage details for a specific period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageDetails {
    /// Credits used in this period.
    #[serde(default)]
    pub credits_used: Option<i64>,
    /// Credits remaining in this period.
    #[serde(default)]
    pub credits_left: Option<i64>,
    /// Requests made (for minute-level tracking).
    #[serde(default)]
    pub requests_made: Option<i32>,
    /// Requests left (for minute-level tracking).
    #[serde(default)]
    pub requests_left: Option<i32>,
}

/// API usage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    /// Usage for the current minute.
    #[serde(default)]
    pub current_minute: Option<UsageDetails>,
    /// Usage for the current day.
    #[serde(default)]
    pub current_day: Option<UsageDetails>,
    /// Usage for the current month.
    #[serde(default)]
    pub current_month: Option<UsageDetails>,
}

/// API key information data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfoData {
    /// Plan details.
    pub plan: PlanInfo,
    /// Current usage.
    pub usage: UsageInfo,
}

/// Response for /v1/key/info endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfoResponse {
    pub status: Status,
    pub data: KeyInfoData,
}

// =============================================================================
// Cryptocurrency Map
// =============================================================================

/// A single cryptocurrency in the map response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptocurrencyMapItem {
    pub id: i32,
    pub rank: Option<i32>,
    pub name: String,
    pub symbol: String,
    pub slug: String,
    pub is_active: i32,
    #[serde(default)]
    pub first_historical_data: Option<String>,
    #[serde(default)]
    pub last_historical_data: Option<String>,
    #[serde(default)]
    pub platform: Option<Platform>,
}

/// Response for /v1/cryptocurrency/map endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptocurrencyMapResponse {
    pub status: Status,
    pub data: Vec<CryptocurrencyMapItem>,
}

/// Request parameters for /v1/cryptocurrency/map endpoint.
#[derive(Debug, Clone, Default)]
pub struct GetCryptocurrencyMapRequest {
    /// Filter by listing status: "active", "inactive", or "untracked".
    pub listing_status: Option<String>,
    /// Offset for pagination (1-based).
    pub start: Option<i32>,
    /// Number of results to return.
    pub limit: Option<i32>,
    /// Sort field: "id" or "cmc_rank".
    pub sort: Option<String>,
    /// Filter by symbol (comma-separated).
    pub symbol: Option<String>,
    /// Auxiliary fields to include: "platform", "first_historical_data", "last_historical_data", "is_active".
    pub aux: Option<String>,
}

// =============================================================================
// Cryptocurrency Info (Metadata)
// =============================================================================

/// URLs associated with a cryptocurrency.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CryptocurrencyUrls {
    #[serde(default)]
    pub website: Vec<String>,
    #[serde(default)]
    pub twitter: Vec<String>,
    #[serde(default)]
    pub message_board: Vec<String>,
    #[serde(default)]
    pub chat: Vec<String>,
    #[serde(default)]
    pub facebook: Vec<String>,
    #[serde(default)]
    pub explorer: Vec<String>,
    #[serde(default)]
    pub reddit: Vec<String>,
    #[serde(default)]
    pub technical_doc: Vec<String>,
    #[serde(default)]
    pub source_code: Vec<String>,
    #[serde(default)]
    pub announcement: Vec<String>,
}

/// Cryptocurrency metadata/info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptocurrencyInfo {
    pub id: i32,
    pub name: String,
    pub symbol: String,
    pub slug: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub subreddit: Option<String>,
    #[serde(default)]
    pub notice: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub platform: Option<Platform>,
    #[serde(default)]
    pub date_added: Option<String>,
    #[serde(default)]
    pub date_launched: Option<String>,
    #[serde(default)]
    pub urls: Option<CryptocurrencyUrls>,
    #[serde(default)]
    pub is_hidden: Option<i32>,
    #[serde(default)]
    pub infinite_supply: Option<bool>,
    #[serde(default)]
    pub self_reported_circulating_supply: Option<f64>,
    #[serde(default)]
    pub self_reported_market_cap: Option<f64>,
    #[serde(default)]
    pub self_reported_tags: Option<Vec<String>>,
}

/// Response for /v1/cryptocurrency/info endpoint.
/// Data is keyed by the requested identifier (id, slug, or symbol).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptocurrencyInfoResponse {
    pub status: Status,
    pub data: HashMap<String, CryptocurrencyInfo>,
}

/// Request parameters for /v1/cryptocurrency/info endpoint.
#[derive(Debug, Clone, Default)]
pub struct GetCryptocurrencyInfoRequest {
    /// CoinMarketCap cryptocurrency ID (comma-separated for multiple).
    pub id: Option<String>,
    /// Cryptocurrency slug (comma-separated for multiple).
    pub slug: Option<String>,
    /// Cryptocurrency symbol (comma-separated for multiple).
    pub symbol: Option<String>,
    /// Contract address (for tokens).
    pub address: Option<String>,
    /// Auxiliary fields: "urls", "logo", "description", "tags", "platform", "date_added", "notice".
    pub aux: Option<String>,
    /// Skip invalid lookups instead of erroring.
    pub skip_invalid: Option<bool>,
}

// =============================================================================
// Cryptocurrency Quotes Latest
// =============================================================================

/// Tag structured as an object (used in /v2/cryptocurrency/quotes/latest).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotesTag {
    pub slug: String,
    pub name: String,
    pub category: String,
}

/// Cryptocurrency data returned by /v2/cryptocurrency/quotes/latest.
/// Different from Cryptocurrency in listings: tags are objects, not strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotesCryptocurrency {
    pub id: i32,
    pub name: String,
    pub symbol: String,
    pub slug: String,
    pub num_market_pairs: Option<i32>,
    pub date_added: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<QuotesTag>>,
    pub max_supply: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub infinite_supply: Option<bool>,
    pub platform: Option<Platform>,
    pub cmc_rank: Option<i32>,
    pub self_reported_circulating_supply: Option<f64>,
    pub self_reported_market_cap: Option<f64>,
    pub tvl_ratio: Option<f64>,
    pub last_updated: String,
    pub quote: HashMap<String, Quote>,
    #[serde(default)]
    pub is_active: Option<i32>,
    #[serde(default)]
    pub is_fiat: Option<i32>,
}

/// Response for /v2/cryptocurrency/quotes/latest endpoint.
/// Data is keyed by the requested identifier (id, slug, or symbol).
/// Each key maps to an array of cryptocurrencies (symbol can match multiple coins).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotesLatestResponse {
    pub status: Status,
    pub data: HashMap<String, Vec<QuotesCryptocurrency>>,
}

/// Request parameters for /v2/cryptocurrency/quotes/latest endpoint.
#[derive(Debug, Clone, Default)]
pub struct GetQuotesLatestRequest {
    /// CoinMarketCap cryptocurrency ID (comma-separated for multiple).
    pub id: Option<String>,
    /// Cryptocurrency slug (comma-separated for multiple).
    pub slug: Option<String>,
    /// Cryptocurrency symbol (comma-separated for multiple).
    pub symbol: Option<String>,
    /// Currency for price conversion (e.g., "USD", "EUR").
    pub convert: Option<String>,
    /// CoinMarketCap ID for conversion currency.
    pub convert_id: Option<String>,
    /// Auxiliary fields to include.
    pub aux: Option<String>,
    /// Skip invalid lookups instead of erroring.
    pub skip_invalid: Option<bool>,
}

// =============================================================================
// Fiat Currency Map
// =============================================================================

/// A single fiat currency in the map response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatMapItem {
    pub id: i32,
    pub name: String,
    pub sign: String,
    pub symbol: String,
}

/// Response for /v1/fiat/map endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatMapResponse {
    pub status: Status,
    pub data: Vec<FiatMapItem>,
}

/// Request parameters for /v1/fiat/map endpoint.
#[derive(Debug, Clone, Default)]
pub struct GetFiatMapRequest {
    /// Offset for pagination (1-based).
    pub start: Option<i32>,
    /// Number of results to return.
    pub limit: Option<i32>,
    /// Sort field: "id" or "name".
    pub sort: Option<String>,
    /// Include precious metals (gold, silver, etc.).
    pub include_metals: Option<bool>,
}

// =============================================================================
// Price Conversion
// =============================================================================

/// Conversion quote with price and timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionQuote {
    pub price: f64,
    pub last_updated: String,
}

/// Price conversion result data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceConversionData {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub amount: f64,
    pub last_updated: String,
    pub quote: HashMap<String, ConversionQuote>,
}

/// Response for /v1/tools/price-conversion endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceConversionResponse {
    pub status: Status,
    pub data: PriceConversionData,
}

/// Request parameters for /v1/tools/price-conversion endpoint.
#[derive(Debug, Clone, Default)]
pub struct PriceConversionRequest {
    /// Amount of the source currency to convert.
    pub amount: f64,
    /// CoinMarketCap ID of the source currency.
    pub id: Option<i32>,
    /// Symbol of the source currency.
    pub symbol: Option<String>,
    /// Target currency symbol(s) for conversion (comma-separated).
    pub convert: Option<String>,
    /// Target CoinMarketCap ID(s) for conversion (comma-separated).
    pub convert_id: Option<String>,
    /// Historical time for conversion (ISO 8601 timestamp).
    pub time: Option<String>,
}

// =============================================================================
// ToMetrics / ToState Implementations
// =============================================================================

use crate::engine::{Metric, StateEntry, ToMetrics, ToState};
use chrono::Utc;

impl ToMetrics for FearAndGreedResponse {
    fn to_metrics(&self, source: &str) -> Vec<Metric> {
        vec![
            Metric::new(source, "fear_and_greed_index", self.data.value)
                .with_label("classification", &self.data.value_classification),
        ]
    }
}

impl ToState for FearAndGreedResponse {
    fn to_state(&self, source: &str) -> Vec<StateEntry> {
        vec![StateEntry::new(
            format!("state:{}:fear_and_greed", source),
            serde_json::json!({
                "value": self.data.value,
                "classification": self.data.value_classification,
                "update_time": self.data.update_time,
                "updated_at": Utc::now().to_rfc3339(),
            }),
        )]
    }
}

impl ToMetrics for GlobalMetricsQuotesLatestResponse {
    fn to_metrics(&self, source: &str) -> Vec<Metric> {
        let mut metrics = vec![
            Metric::new(
                source,
                "active_cryptocurrencies",
                self.data.active_cryptocurrencies as f64,
            ),
            Metric::new(
                source,
                "total_cryptocurrencies",
                self.data.total_cryptocurrencies as f64,
            ),
            Metric::new(
                source,
                "active_exchanges",
                self.data.active_exchanges as f64,
            ),
            Metric::new(source, "btc_dominance", self.data.btc_dominance),
            Metric::new(source, "eth_dominance", self.data.eth_dominance),
        ];

        for (currency, quote) in &self.data.quote {
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

impl ToState for GlobalMetricsQuotesLatestResponse {
    fn to_state(&self, source: &str) -> Vec<StateEntry> {
        vec![StateEntry::new(
            format!("state:{}:global_metrics", source),
            serde_json::json!({
                "active_cryptocurrencies": self.data.active_cryptocurrencies,
                "total_cryptocurrencies": self.data.total_cryptocurrencies,
                "btc_dominance": self.data.btc_dominance,
                "eth_dominance": self.data.eth_dominance,
                "quote": self.data.quote,
                "updated_at": Utc::now().to_rfc3339(),
            }),
        )]
    }
}

impl ToMetrics for ListingsLatestResponse {
    fn to_metrics(&self, source: &str) -> Vec<Metric> {
        self.data
            .iter()
            .flat_map(|crypto| {
                crypto.quote.iter().flat_map(|(currency, quote)| {
                    let mut metrics = Vec::new();
                    if let Some(price) = quote.price {
                        metrics.push(
                            Metric::new(source, "price", price)
                                .with_label("symbol", &crypto.symbol)
                                .with_label("currency", currency),
                        );
                    }
                    if let Some(market_cap) = quote.market_cap {
                        metrics.push(
                            Metric::new(source, "market_cap", market_cap)
                                .with_label("symbol", &crypto.symbol)
                                .with_label("currency", currency),
                        );
                    }
                    if let Some(volume) = quote.volume_24h {
                        metrics.push(
                            Metric::new(source, "volume_24h", volume)
                                .with_label("symbol", &crypto.symbol)
                                .with_label("currency", currency),
                        );
                    }
                    metrics
                })
            })
            .collect()
    }
}

impl ToState for ListingsLatestResponse {
    fn to_state(&self, source: &str) -> Vec<StateEntry> {
        self.data
            .iter()
            .map(|crypto| {
                StateEntry::new(
                    format!("state:{}:crypto:{}", source, crypto.symbol.to_lowercase()),
                    serde_json::json!({
                        "id": crypto.id,
                        "name": crypto.name,
                        "symbol": crypto.symbol,
                        "cmc_rank": crypto.cmc_rank,
                        "quote": crypto.quote,
                        "updated_at": Utc::now().to_rfc3339(),
                    }),
                )
            })
            .collect()
    }
}
