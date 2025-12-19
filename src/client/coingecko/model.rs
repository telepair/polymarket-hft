//! CoinGecko API data models.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// CoinGecko API error.
#[derive(Debug, Error)]
pub enum CgError {
    /// HTTP/network error from reqwest middleware.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest_middleware::Error),

    /// Request error from reqwest.
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error: {0}")]
    Api(String),
}

/// Helper to deserialize a value that may be either a string or integer.
fn deserialize_string_or_int<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(serde_json::Value::Number(n)) => n
            .as_i64()
            .map(Some)
            .ok_or_else(|| D::Error::custom("invalid number")),
        Some(serde_json::Value::String(s)) => s
            .parse::<i64>()
            .map(Some)
            .map_err(|_| D::Error::custom("invalid string number")),
        Some(_) => Err(D::Error::custom("expected string or number")),
    }
}

// === Simple Price ===

/// Request parameters for /simple/price endpoint.
#[derive(Debug, Clone, Default)]
pub struct SimplePriceRequest {
    /// Comma-separated list of coin IDs (e.g., "bitcoin,ethereum").
    pub ids: String,
    /// Comma-separated list of target currencies (e.g., "usd,eur").
    pub vs_currencies: String,
    /// Include market cap in response.
    pub include_market_cap: Option<bool>,
    /// Include 24h volume in response.
    pub include_24hr_vol: Option<bool>,
    /// Include 24h change in response.
    pub include_24hr_change: Option<bool>,
    /// Include last updated timestamp.
    pub include_last_updated_at: Option<bool>,
}

/// Price data for a single coin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinPrice {
    /// Price in target currency (dynamic key based on vs_currency).
    #[serde(flatten)]
    pub prices: HashMap<String, Option<f64>>,
}

/// Response from /simple/price endpoint.
/// Map of coin ID to its price data.
pub type SimplePriceResponse = HashMap<String, HashMap<String, Option<f64>>>;

// === Coins List ===

/// Request parameters for /coins/list endpoint.
#[derive(Debug, Clone, Default)]
pub struct CoinsListRequest {
    /// Include platform contract addresses.
    pub include_platform: Option<bool>,
}

/// A coin entry from /coins/list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinListEntry {
    /// Coin ID (e.g., "bitcoin").
    pub id: String,
    /// Coin symbol (e.g., "btc").
    pub symbol: String,
    /// Coin name (e.g., "Bitcoin").
    pub name: String,
    /// Platform information (when include_platform=true).
    #[serde(default)]
    pub platforms: Option<HashMap<String, Option<String>>>,
}

/// Response from /coins/list endpoint.
pub type CoinsListResponse = Vec<CoinListEntry>;

// === Coins Markets ===

/// Request parameters for /coins/markets endpoint.
#[derive(Debug, Clone, Default)]
pub struct CoinsMarketsRequest {
    /// Target currency for market data (required, e.g., "usd").
    pub vs_currency: String,
    /// Comma-separated list of coin IDs to filter.
    pub ids: Option<String>,
    /// Filter by category.
    pub category: Option<String>,
    /// Sort order (e.g., "market_cap_desc", "volume_desc").
    pub order: Option<String>,
    /// Number of results per page (max 250).
    pub per_page: Option<u32>,
    /// Page number for pagination.
    pub page: Option<u32>,
    /// Include 7-day sparkline data.
    pub sparkline: Option<bool>,
    /// Include price change percentage for time frames (e.g., "1h,24h,7d").
    pub price_change_percentage: Option<String>,
    /// Localization (e.g., "en").
    pub locale: Option<String>,
    /// Decimal precision for price.
    pub precision: Option<String>,
}

/// ROI (Return on Investment) data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roi {
    pub times: Option<f64>,
    pub currency: Option<String>,
    pub percentage: Option<f64>,
}

/// Market data for a single coin from /coins/markets endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinMarket {
    /// Coin ID.
    pub id: String,
    /// Coin symbol.
    pub symbol: String,
    /// Coin name.
    pub name: String,
    /// Coin image URL.
    pub image: Option<String>,
    /// Current price.
    pub current_price: Option<f64>,
    /// Market capitalization.
    pub market_cap: Option<f64>,
    /// Market cap rank.
    pub market_cap_rank: Option<u32>,
    /// Fully diluted valuation.
    pub fully_diluted_valuation: Option<f64>,
    /// Total trading volume.
    pub total_volume: Option<f64>,
    /// 24h high.
    pub high_24h: Option<f64>,
    /// 24h low.
    pub low_24h: Option<f64>,
    /// Price change in 24h.
    pub price_change_24h: Option<f64>,
    /// Price change percentage in 24h.
    pub price_change_percentage_24h: Option<f64>,
    /// Market cap change in 24h.
    pub market_cap_change_24h: Option<f64>,
    /// Market cap change percentage in 24h.
    pub market_cap_change_percentage_24h: Option<f64>,
    /// Circulating supply.
    pub circulating_supply: Option<f64>,
    /// Total supply.
    pub total_supply: Option<f64>,
    /// Maximum supply.
    pub max_supply: Option<f64>,
    /// All-time high price.
    pub ath: Option<f64>,
    /// ATH change percentage.
    pub ath_change_percentage: Option<f64>,
    /// ATH date.
    pub ath_date: Option<String>,
    /// All-time low price.
    pub atl: Option<f64>,
    /// ATL change percentage.
    pub atl_change_percentage: Option<f64>,
    /// ATL date.
    pub atl_date: Option<String>,
    /// ROI data.
    pub roi: Option<Roi>,
    /// Last updated timestamp.
    pub last_updated: Option<String>,
    /// 7-day sparkline data.
    #[serde(default)]
    pub sparkline_in_7d: Option<SparklineData>,
    /// Price change percentage in 1h.
    pub price_change_percentage_1h_in_currency: Option<f64>,
    /// Price change percentage in 24h (in currency).
    pub price_change_percentage_24h_in_currency: Option<f64>,
    /// Price change percentage in 7d.
    pub price_change_percentage_7d_in_currency: Option<f64>,
}

/// Sparkline data for 7 days.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparklineData {
    pub price: Vec<f64>,
}

/// Response from /coins/markets endpoint.
pub type CoinsMarketsResponse = Vec<CoinMarket>;

// === Trending ===

/// Trending coin item data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingCoinItem {
    pub id: String,
    #[serde(default, deserialize_with = "deserialize_string_or_int")]
    pub coin_id: Option<i64>,
    pub name: String,
    pub symbol: String,
    pub market_cap_rank: Option<u32>,
    pub thumb: Option<String>,
    pub small: Option<String>,
    pub large: Option<String>,
    pub slug: Option<String>,
    pub price_btc: Option<f64>,
    pub score: Option<i32>,
    #[serde(default)]
    pub data: Option<TrendingCoinData>,
}

/// Additional data for trending coin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingCoinData {
    pub price: Option<f64>,
    pub price_btc: Option<String>,
    pub price_change_percentage_24h: Option<HashMap<String, f64>>,
    pub market_cap: Option<String>,
    pub market_cap_btc: Option<String>,
    pub total_volume: Option<String>,
    pub total_volume_btc: Option<String>,
    pub sparkline: Option<String>,
    pub content: Option<serde_json::Value>,
}

/// Trending coin wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingCoin {
    pub item: TrendingCoinItem,
}

/// Trending NFT data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingNft {
    pub id: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub thumb: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_int")]
    pub nft_contract_id: Option<i64>,
    pub native_currency_symbol: Option<String>,
    pub floor_price_in_native_currency: Option<f64>,
    #[serde(rename = "floor_price_24h_percentage_change")]
    pub floor_price_24h_percentage_change: Option<f64>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// Trending category data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingCategory {
    #[serde(default, deserialize_with = "deserialize_string_or_int")]
    pub id: Option<i64>,
    pub name: Option<String>,
    pub market_cap_1h_change: Option<f64>,
    pub slug: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_int")]
    pub coins_count: Option<i64>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// Response from /search/trending endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingResponse {
    pub coins: Vec<TrendingCoin>,
    #[serde(default)]
    pub nfts: Vec<TrendingNft>,
    #[serde(default)]
    pub categories: Vec<TrendingCategory>,
}

// === Global ===

/// Market cap percentage by coin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketCapPercentage {
    #[serde(flatten)]
    pub percentages: HashMap<String, f64>,
}

/// Global market data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalData {
    /// Number of active cryptocurrencies.
    pub active_cryptocurrencies: Option<i64>,
    /// Number of upcoming ICOs.
    pub upcoming_icos: Option<i64>,
    /// Number of ongoing ICOs.
    pub ongoing_icos: Option<i64>,
    /// Number of ended ICOs.
    pub ended_icos: Option<i64>,
    /// Number of markets.
    pub markets: Option<i64>,
    /// Total market cap by currency.
    #[serde(default)]
    pub total_market_cap: HashMap<String, f64>,
    /// Total 24h volume by currency.
    #[serde(default)]
    pub total_volume: HashMap<String, f64>,
    /// Market cap percentage by coin.
    #[serde(default)]
    pub market_cap_percentage: HashMap<String, f64>,
    /// Market cap change percentage in 24h (USD).
    pub market_cap_change_percentage_24h_usd: Option<f64>,
    /// Last updated timestamp (Unix).
    pub updated_at: Option<i64>,
}

/// Response from /global endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalResponse {
    pub data: GlobalData,
}

// === Exchanges ===

/// Request parameters for /exchanges endpoint.
#[derive(Debug, Clone, Default)]
pub struct ExchangesRequest {
    /// Number of results per page (max 250).
    pub per_page: Option<u32>,
    /// Page number for pagination.
    pub page: Option<u32>,
}

/// Exchange data from /exchanges endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exchange {
    /// Exchange ID (e.g., "binance", "coinbase").
    pub id: String,
    /// Exchange name.
    pub name: String,
    /// Year the exchange was established.
    pub year_established: Option<i32>,
    /// Country where exchange is based.
    pub country: Option<String>,
    /// Description of the exchange.
    pub description: Option<String>,
    /// Exchange website URL.
    pub url: Option<String>,
    /// Exchange logo image URL.
    pub image: Option<String>,
    /// Whether exchange has trading incentive.
    pub has_trading_incentive: Option<bool>,
    /// Trust score (1-10).
    pub trust_score: Option<i32>,
    /// Trust score rank.
    pub trust_score_rank: Option<i32>,
    /// 24h trading volume in BTC.
    pub trade_volume_24h_btc: Option<f64>,
    /// 24h trading volume in BTC (normalized).
    pub trade_volume_24h_btc_normalized: Option<f64>,
}

/// Response from /exchanges endpoint.
pub type ExchangesResponse = Vec<Exchange>;

// === Simple Supported VS Currencies ===

/// Response from /simple/supported_vs_currencies endpoint.
/// Returns a list of supported vs currencies (e.g., ["usd", "eur", "btc"]).
pub type SupportedVsCurrenciesResponse = Vec<String>;

// === Coin Detail (/coins/{id}) ===

/// Request parameters for /coins/{id} endpoint.
#[derive(Debug, Clone, Default)]
pub struct CoinDetailRequest {
    /// Coin ID (e.g., "bitcoin").
    pub id: String,
    /// Include localization data.
    pub localization: Option<bool>,
    /// Include ticker data.
    pub tickers: Option<bool>,
    /// Include market data.
    pub market_data: Option<bool>,
    /// Include community data.
    pub community_data: Option<bool>,
    /// Include developer data.
    pub developer_data: Option<bool>,
    /// Include 7-day sparkline.
    pub sparkline: Option<bool>,
}

/// Coin detail response from /coins/{id} endpoint.
/// Using Value for flexibility as response is very complex.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinDetailResponse {
    pub id: String,
    pub symbol: String,
    pub name: String,
    #[serde(default)]
    pub web_slug: Option<String>,
    #[serde(default)]
    pub asset_platform_id: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub description: Option<HashMap<String, String>>,
    #[serde(default)]
    pub links: Option<serde_json::Value>,
    #[serde(default)]
    pub image: Option<CoinImage>,
    #[serde(default)]
    pub genesis_date: Option<String>,
    #[serde(default)]
    pub market_cap_rank: Option<i32>,
    #[serde(default)]
    pub market_data: Option<serde_json::Value>,
    #[serde(default)]
    pub community_data: Option<serde_json::Value>,
    #[serde(default)]
    pub developer_data: Option<serde_json::Value>,
    #[serde(default)]
    pub last_updated: Option<String>,
}

/// Coin image URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinImage {
    pub thumb: Option<String>,
    pub small: Option<String>,
    pub large: Option<String>,
}

// === Market Chart (/coins/{id}/market_chart) ===

/// Request parameters for /coins/{id}/market_chart endpoint.
#[derive(Debug, Clone, Default)]
pub struct MarketChartRequest {
    /// Coin ID (e.g., "bitcoin").
    pub id: String,
    /// Target currency (e.g., "usd").
    pub vs_currency: String,
    /// Number of days (e.g., "1", "7", "30", "max").
    pub days: String,
    /// Data interval (optional, auto-determined by days if not set).
    pub interval: Option<String>,
}

/// Response from /coins/{id}/market_chart endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketChartResponse {
    /// Array of [timestamp, price] pairs.
    pub prices: Vec<Vec<f64>>,
    /// Array of [timestamp, market_cap] pairs.
    pub market_caps: Vec<Vec<f64>>,
    /// Array of [timestamp, total_volume] pairs.
    pub total_volumes: Vec<Vec<f64>>,
}

// === Coin History (/coins/{id}/history) ===

/// Request parameters for /coins/{id}/history endpoint.
#[derive(Debug, Clone, Default)]
pub struct CoinHistoryRequest {
    /// Coin ID (e.g., "bitcoin").
    pub id: String,
    /// Date in dd-mm-yyyy format (e.g., "30-12-2022").
    pub date: String,
    /// Include localization data.
    pub localization: Option<bool>,
}

/// Response from /coins/{id}/history endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinHistoryResponse {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub image: Option<CoinImage>,
    #[serde(default)]
    pub market_data: Option<HistoricalMarketData>,
    #[serde(default)]
    pub community_data: Option<serde_json::Value>,
    #[serde(default)]
    pub developer_data: Option<serde_json::Value>,
}

/// Historical market data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMarketData {
    #[serde(default)]
    pub current_price: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub market_cap: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub total_volume: Option<HashMap<String, f64>>,
}

// === OHLC (/coins/{id}/ohlc) ===

/// Request parameters for /coins/{id}/ohlc endpoint.
#[derive(Debug, Clone, Default)]
pub struct OhlcRequest {
    /// Coin ID (e.g., "bitcoin").
    pub id: String,
    /// Target currency (e.g., "usd").
    pub vs_currency: String,
    /// Number of days (1, 7, 14, 30, 90, 180, 365, max).
    pub days: String,
}

/// OHLC candle data: [timestamp, open, high, low, close].
pub type OhlcCandle = Vec<f64>;

/// Response from /coins/{id}/ohlc endpoint.
/// Array of OHLC candles.
pub type OhlcResponse = Vec<OhlcCandle>;
