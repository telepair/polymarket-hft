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
    pub price: f64,
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
