//! Core types for CLOB trading.

use std::collections::HashMap;

use alloy_primitives::Address;
use serde::{Deserialize, Serialize};

// =============================================================================
// Fundamental Enums
// =============================================================================

/// Blockchain network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Chain {
    /// Polygon mainnet (137).
    #[default]
    #[serde(rename = "137")]
    Polygon = 137,
    /// Amoy testnet (80002).
    #[serde(rename = "80002")]
    Amoy = 80002,
}

impl Chain {
    /// Returns the chain ID as a u64.
    pub fn chain_id(self) -> u64 {
        self as u64
    }
}

/// Order type for trading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Good Till Cancel - standard limit order.
    #[default]
    Gtc,
    /// Fill or Kill - must execute completely or not at all.
    Fok,
    /// Good Till Date - limit order with expiration.
    Gtd,
    /// Fill and Kill - partial fills allowed, cancel remainder.
    Fak,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gtc => write!(f, "GTC"),
            Self::Fok => write!(f, "FOK"),
            Self::Gtd => write!(f, "GTD"),
            Self::Fak => write!(f, "FAK"),
        }
    }
}

/// Asset type for balance/allowance operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetType {
    #[default]
    Collateral,
    Conditional,
}

/// Tick size for price precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TickSize {
    #[serde(rename = "0.1")]
    PointOne,
    #[default]
    #[serde(rename = "0.01")]
    PointZeroOne,
    #[serde(rename = "0.001")]
    PointZeroZeroOne,
    #[serde(rename = "0.0001")]
    PointZeroZeroZeroOne,
}

impl TickSize {
    /// Returns the tick size as an f64.
    pub fn as_f64(self) -> f64 {
        match self {
            Self::PointOne => 0.1,
            Self::PointZeroOne => 0.01,
            Self::PointZeroZeroOne => 0.001,
            Self::PointZeroZeroZeroOne => 0.0001,
        }
    }

    /// Returns the tick size as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PointOne => "0.1",
            Self::PointZeroOne => "0.01",
            Self::PointZeroZeroOne => "0.001",
            Self::PointZeroZeroZeroOne => "0.0001",
        }
    }
}

impl std::fmt::Display for TickSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =============================================================================
// API Key Credentials
// =============================================================================

/// API key credentials for L2 authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreds {
    /// API key.
    pub key: String,
    /// API secret.
    pub secret: String,
    /// API passphrase.
    pub passphrase: String,
}

/// Raw API key response from server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyRaw {
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

impl From<ApiKeyRaw> for ApiKeyCreds {
    fn from(raw: ApiKeyRaw) -> Self {
        Self {
            key: raw.api_key,
            secret: raw.secret,
            passphrase: raw.passphrase,
        }
    }
}

/// Response containing multiple API keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeysResponse {
    pub api_keys: Vec<String>,
}

// =============================================================================
// Authentication Headers
// =============================================================================

/// L1 authentication headers (EIP-712 signature based).
/// Used for API key management operations.
#[derive(Debug, Clone)]
pub struct L1PolyHeader {
    pub poly_address: String,
    pub poly_signature: String,
    pub poly_timestamp: String,
    pub poly_nonce: String,
}

impl L1PolyHeader {
    /// Converts the struct to a HashMap for HTTP client usage.
    #[allow(dead_code)]
    pub fn to_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("POLY_ADDRESS".to_string(), self.poly_address.clone());
        headers.insert("POLY_SIGNATURE".to_string(), self.poly_signature.clone());
        headers.insert("POLY_TIMESTAMP".to_string(), self.poly_timestamp.clone());
        headers.insert("POLY_NONCE".to_string(), self.poly_nonce.clone());
        headers
    }
}

/// L2 authentication headers (HMAC signature based).
/// Used for trading operations with API credentials.
#[derive(Debug, Clone)]
pub struct L2PolyHeader {
    pub poly_address: String,
    pub poly_signature: String,
    pub poly_timestamp: String,
    pub poly_api_key: String,
    pub poly_passphrase: String,
}

impl L2PolyHeader {
    /// Converts the struct to a HashMap for HTTP client usage.
    pub fn to_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("POLY_ADDRESS".to_string(), self.poly_address.clone());
        headers.insert("POLY_SIGNATURE".to_string(), self.poly_signature.clone());
        headers.insert("POLY_TIMESTAMP".to_string(), self.poly_timestamp.clone());
        headers.insert("POLY_API_KEY".to_string(), self.poly_api_key.clone());
        headers.insert("POLY_PASSPHRASE".to_string(), self.poly_passphrase.clone());
        headers
    }
}

// =============================================================================
// Order Types
// =============================================================================

/// Simplified user order for creating limit orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLimitOrder {
    /// Token ID of the conditional token asset being traded.
    #[serde(rename = "tokenID")]
    pub token_id: String,

    /// Price used to create the order.
    pub price: f64,

    /// Size in terms of the ConditionalToken.
    pub size: f64,

    /// Side of the order.
    pub side: super::pricing::Side,

    /// Fee rate, in basis points, charged to the order maker.
    #[serde(rename = "feeRateBps", skip_serializing_if = "Option::is_none")]
    pub fee_rate_bps: Option<u32>,

    /// Nonce used for onchain cancellations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,

    /// Timestamp after which the order is expired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u64>,

    /// Address of the order taker (zero address = public order).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taker: Option<Address>,
}

/// Simplified market order for users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMarketOrder {
    /// Token ID of the conditional token asset being traded.
    #[serde(rename = "tokenID")]
    pub token_id: String,

    /// Price (if not present, market price will be calculated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,

    /// BUY orders: $$$ Amount to buy. SELL orders: Shares to sell.
    pub amount: f64,

    /// Side of the order.
    pub side: super::pricing::Side,

    /// Fee rate, in basis points.
    #[serde(rename = "feeRateBps", skip_serializing_if = "Option::is_none")]
    pub fee_rate_bps: Option<u32>,

    /// Nonce used for onchain cancellations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,

    /// Address of the order taker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taker: Option<Address>,

    /// Order type (FOK or FAK).
    #[serde(rename = "orderType", skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
}

/// Order payload for cancellation.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderPayload {
    pub order_id: String,
}

/// Order market cancel parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderMarketCancelParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
}

/// Arguments for posting multiple orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostOrdersArgs {
    pub order: serde_json::Value,
    pub order_type: OrderType,
}

/// Create order options.
#[derive(Debug, Clone)]
pub struct CreateOrderOptions {
    pub tick_size: TickSize,
    pub neg_risk: Option<bool>,
}

// =============================================================================
// Open Order
// =============================================================================

/// Open order information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOrder {
    pub id: String,
    pub status: String,
    pub owner: String,
    pub maker_address: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub original_size: String,
    pub size_matched: String,
    pub price: String,
    pub associate_trades: Vec<String>,
    pub outcome: String,
    pub created_at: u64,
    pub expiration: String,
    pub order_type: String,
}

/// Open orders response.
pub type OpenOrdersResponse = Vec<OpenOrder>;

/// Open order parameters for filtering.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenOrderParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
}

// =============================================================================
// Trade Types
// =============================================================================

/// Trade history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub taker_order_id: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub size: String,
    pub fee_rate_bps: String,
    pub price: String,
    pub status: String,
    pub match_time: String,
    pub last_update: String,
    pub outcome: String,
    pub maker_address: String,
    pub owner: String,
    pub transaction_hash: Option<String>,
    pub bucket_index: Option<String>,
    pub maker_orders: Vec<MakerOrder>,
    #[serde(rename = "type")]
    pub trade_type: Option<String>,
}

/// Maker order information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakerOrder {
    pub order_id: String,
    pub owner: String,
    pub maker_address: String,
    pub matched_amount: String,
    pub price: String,
    pub fee_rate_bps: String,
    pub asset_id: String,
    pub outcome: String,
    pub side: super::pricing::Side,
}

/// Trade query parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradeParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maker_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<u64>,
}

/// Paginated trades response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesPaginatedResponse {
    pub data: Vec<Trade>,
    pub next_cursor: String,
}

// =============================================================================
// Balance & Allowance
// =============================================================================

/// Balance allowance parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceAllowanceParams {
    pub asset_type: AssetType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
}

/// Balance allowance response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceAllowance {
    pub balance: String,
    pub allowance: String,
}

// =============================================================================
// Ban Status
// =============================================================================

/// Ban (closed-only) status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanStatus {
    #[serde(rename = "closedOnlyMode")]
    pub closed_only_mode: bool,
}

// =============================================================================
// Pagination Constants
// =============================================================================

/// Initial cursor for pagination.
pub const INITIAL_CURSOR: &str = "MA==";

/// End cursor indicating no more results.
pub const END_CURSOR: &str = "LTE=";
