//! Position-related types and request builders.

use serde::{Deserialize, Serialize};
use url::Url;

use super::{SortDirection, validate_event_id, validate_limit, validate_market_id, validate_user};
use crate::error::{PolymarketError, Result};

/// Sort by options for positions query.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum PositionSortBy {
    /// Sort by current value.
    #[serde(rename = "CURRENT")]
    Current,
    /// Sort by initial value.
    #[serde(rename = "INITIAL")]
    Initial,
    /// Sort by token count (default).
    #[serde(rename = "TOKENS")]
    #[default]
    Tokens,
    /// Sort by cash PnL.
    #[serde(rename = "CASHPNL")]
    CashPnl,
    /// Sort by percent PnL.
    #[serde(rename = "PERCENTPNL")]
    PercentPnl,
    /// Sort by title.
    #[serde(rename = "TITLE")]
    Title,
    /// Sort by resolving status.
    #[serde(rename = "RESOLVING")]
    Resolving,
    /// Sort by price.
    #[serde(rename = "PRICE")]
    Price,
    /// Sort by average price.
    #[serde(rename = "AVGPRICE")]
    AvgPrice,
}

impl std::fmt::Display for PositionSortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionSortBy::Current => write!(f, "CURRENT"),
            PositionSortBy::Initial => write!(f, "INITIAL"),
            PositionSortBy::Tokens => write!(f, "TOKENS"),
            PositionSortBy::CashPnl => write!(f, "CASHPNL"),
            PositionSortBy::PercentPnl => write!(f, "PERCENTPNL"),
            PositionSortBy::Title => write!(f, "TITLE"),
            PositionSortBy::Resolving => write!(f, "RESOLVING"),
            PositionSortBy::Price => write!(f, "PRICE"),
            PositionSortBy::AvgPrice => write!(f, "AVGPRICE"),
        }
    }
}

impl std::str::FromStr for PositionSortBy {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "CURRENT" => Ok(PositionSortBy::Current),
            "INITIAL" => Ok(PositionSortBy::Initial),
            "TOKENS" => Ok(PositionSortBy::Tokens),
            "CASHPNL" => Ok(PositionSortBy::CashPnl),
            "PERCENTPNL" => Ok(PositionSortBy::PercentPnl),
            "TITLE" => Ok(PositionSortBy::Title),
            "RESOLVING" => Ok(PositionSortBy::Resolving),
            "PRICE" => Ok(PositionSortBy::Price),
            "AVGPRICE" => Ok(PositionSortBy::AvgPrice),
            _ => Err(format!(
                "Invalid sort by: '{}'. Valid options: CURRENT, INITIAL, TOKENS, CASHPNL, PERCENTPNL, TITLE, RESOLVING, PRICE, AVGPRICE",
                s
            )),
        }
    }
}

/// A user's position in a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Proxy wallet address (0x-prefixed, 40 hex chars).
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    /// Asset identifier.
    pub asset: String,
    /// Condition ID (0x-prefixed, 64 hex string).
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    /// Position size.
    pub size: f64,
    /// Average price of the position.
    #[serde(rename = "avgPrice")]
    pub avg_price: f64,
    /// Initial value of the position.
    #[serde(rename = "initialValue")]
    pub initial_value: f64,
    /// Current value of the position.
    #[serde(rename = "currentValue")]
    pub current_value: f64,
    /// Cash profit and loss.
    #[serde(rename = "cashPnl")]
    pub cash_pnl: f64,
    /// Percent profit and loss.
    #[serde(rename = "percentPnl")]
    pub percent_pnl: f64,
    /// Total bought amount.
    #[serde(rename = "totalBought")]
    pub total_bought: f64,
    /// Realized profit and loss.
    #[serde(rename = "realizedPnl")]
    pub realized_pnl: f64,
    /// Percent realized profit and loss.
    #[serde(rename = "percentRealizedPnl")]
    pub percent_realized_pnl: f64,
    /// Current price.
    #[serde(rename = "curPrice")]
    pub cur_price: f64,
    /// Whether the position is redeemable.
    pub redeemable: bool,
    /// Whether the position is mergeable.
    pub mergeable: bool,
    /// Market title.
    pub title: String,
    /// Market slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Event slug.
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    /// Position outcome.
    pub outcome: String,
    /// Outcome index.
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    /// Opposite outcome.
    #[serde(rename = "oppositeOutcome")]
    pub opposite_outcome: String,
    /// Opposite asset ID.
    #[serde(rename = "oppositeAsset")]
    pub opposite_asset: String,
    /// End date of the market.
    #[serde(rename = "endDate")]
    pub end_date: String,
    /// Whether this is a negative risk market.
    #[serde(rename = "negativeRisk")]
    pub negative_risk: bool,
}

/// Sort by options for closed positions query.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum ClosedPositionSortBy {
    /// Sort by realized PnL (default).
    #[serde(rename = "REALIZEDPNL")]
    #[default]
    RealizedPnl,
    /// Sort by title.
    #[serde(rename = "TITLE")]
    Title,
    /// Sort by price.
    #[serde(rename = "PRICE")]
    Price,
    /// Sort by average price.
    #[serde(rename = "AVGPRICE")]
    AvgPrice,
    /// Sort by timestamp.
    #[serde(rename = "TIMESTAMP")]
    Timestamp,
}

impl std::fmt::Display for ClosedPositionSortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClosedPositionSortBy::RealizedPnl => write!(f, "REALIZEDPNL"),
            ClosedPositionSortBy::Title => write!(f, "TITLE"),
            ClosedPositionSortBy::Price => write!(f, "PRICE"),
            ClosedPositionSortBy::AvgPrice => write!(f, "AVGPRICE"),
            ClosedPositionSortBy::Timestamp => write!(f, "TIMESTAMP"),
        }
    }
}

impl std::str::FromStr for ClosedPositionSortBy {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "REALIZEDPNL" => Ok(ClosedPositionSortBy::RealizedPnl),
            "TITLE" => Ok(ClosedPositionSortBy::Title),
            "PRICE" => Ok(ClosedPositionSortBy::Price),
            "AVGPRICE" => Ok(ClosedPositionSortBy::AvgPrice),
            "TIMESTAMP" => Ok(ClosedPositionSortBy::Timestamp),
            _ => Err(format!(
                "Invalid sort by: '{}'. Valid options: REALIZEDPNL, TITLE, PRICE, AVGPRICE, TIMESTAMP",
                s
            )),
        }
    }
}

/// A user's closed position in a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedPosition {
    /// Proxy wallet address (0x-prefixed, 40 hex chars).
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    /// Asset identifier.
    pub asset: String,
    /// Condition ID (0x-prefixed, 64 hex string).
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    /// Average price of the position.
    #[serde(rename = "avgPrice")]
    pub avg_price: f64,
    /// Total bought amount.
    #[serde(rename = "totalBought")]
    pub total_bought: f64,
    /// Realized profit and loss.
    #[serde(rename = "realizedPnl")]
    pub realized_pnl: f64,
    /// Current price.
    #[serde(rename = "curPrice")]
    pub cur_price: f64,
    /// Unix timestamp.
    pub timestamp: i64,
    /// Market title.
    pub title: String,
    /// Market slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Event slug.
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    /// Position outcome.
    pub outcome: String,
    /// Outcome index.
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    /// Opposite outcome.
    #[serde(rename = "oppositeOutcome")]
    pub opposite_outcome: String,
    /// Opposite asset ID.
    #[serde(rename = "oppositeAsset")]
    pub opposite_asset: String,
    /// End date of the market.
    #[serde(rename = "endDate")]
    pub end_date: String,
}

/// Response from the value endpoint.
///
/// Contains the total value of a user's positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPositionValue {
    /// User Profile Address (0x-prefixed, 40 hex chars).
    pub user: String,
    /// The total value of user's positions.
    pub value: f64,
}

/// Request parameters for [`Client::get_user_positions`](crate::data::Client::get_user_positions).
///
/// # Example
///
/// ```no_run
/// use polymarket_hft::data::{Client, GetUserPositionsRequest, PositionSortBy, SortDirection};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::new();
///     let positions = client.get_user_positions(GetUserPositionsRequest {
///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
///         limit: Some(10),
///         sort_by: Some(PositionSortBy::CashPnl),
///         sort_direction: Some(SortDirection::Desc),
///         ..Default::default()
///     }).await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetUserPositionsRequest<'a> {
    /// User Profile Address (0x-prefixed, 40 hex chars). Required.
    pub user: &'a str,
    /// Market condition IDs to filter by (0x-prefixed, 64 hex chars each).
    pub markets: Option<&'a [&'a str]>,
    /// Event IDs to filter by.
    pub event_ids: Option<&'a [i64]>,
    /// Minimum position size (must be >= 0).
    pub size_threshold: Option<f64>,
    /// Filter for redeemable positions.
    pub redeemable: Option<bool>,
    /// Filter for mergeable positions.
    pub mergeable: Option<bool>,
    /// Limit for results (0-500, default: 100).
    pub limit: Option<i32>,
    /// Offset for pagination (0-10000, default: 0).
    pub offset: Option<i32>,
    /// Sort field (default: TOKENS).
    pub sort_by: Option<PositionSortBy>,
    /// Sort direction (default: DESC).
    pub sort_direction: Option<SortDirection>,
    /// Title filter (max 160 chars).
    pub title: Option<&'a str>,
}

impl GetUserPositionsRequest<'_> {
    /// Validates the request parameters.
    pub fn validate(&self) -> Result<()> {
        validate_user(self.user)?;

        if let Some(market_ids) = self.markets {
            for market_id in market_ids {
                validate_market_id(market_id)?;
            }
        }

        if let Some(ids) = self.event_ids {
            for id in ids {
                validate_event_id(*id)?;
            }
        }

        if let Some(threshold) = self.size_threshold
            && threshold < 0.0
        {
            return Err(PolymarketError::bad_request(
                "sizeThreshold must be >= 0".to_string(),
            ));
        }

        validate_limit(self.limit)?;

        if let Some(o) = self.offset
            && !(0..=10000).contains(&o)
        {
            return Err(PolymarketError::bad_request(
                "offset must be between 0 and 10000".to_string(),
            ));
        }

        if let Some(t) = self.title
            && t.len() > 160
        {
            return Err(PolymarketError::bad_request(
                "title must be at most 160 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Builds the URL with query parameters for this request.
    pub fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("positions");

        // Required: user parameter
        url.query_pairs_mut().append_pair("user", self.user);

        // Optional: market filter (comma-separated)
        if let Some(market_ids) = self.markets.filter(|ids| !ids.is_empty()) {
            let market_value = market_ids.join(",");
            url.query_pairs_mut().append_pair("market", &market_value);
        }

        // Optional: eventId filter (comma-separated)
        if let Some(ids) = self.event_ids.filter(|ids| !ids.is_empty()) {
            let event_value = ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            url.query_pairs_mut().append_pair("eventId", &event_value);
        }

        // Optional: sizeThreshold
        if let Some(threshold) = self.size_threshold {
            url.query_pairs_mut()
                .append_pair("sizeThreshold", &threshold.to_string());
        }

        // Optional: redeemable
        if let Some(r) = self.redeemable {
            url.query_pairs_mut()
                .append_pair("redeemable", &r.to_string());
        }

        // Optional: mergeable
        if let Some(m) = self.mergeable {
            url.query_pairs_mut()
                .append_pair("mergeable", &m.to_string());
        }

        // Optional: limit
        if let Some(l) = self.limit {
            url.query_pairs_mut().append_pair("limit", &l.to_string());
        }

        // Optional: offset
        if let Some(o) = self.offset {
            url.query_pairs_mut().append_pair("offset", &o.to_string());
        }

        // Optional: sortBy
        if let Some(sort) = self.sort_by {
            url.query_pairs_mut()
                .append_pair("sortBy", &sort.to_string());
        }

        // Optional: sortDirection
        if let Some(dir) = self.sort_direction {
            url.query_pairs_mut()
                .append_pair("sortDirection", &dir.to_string());
        }

        // Optional: title
        if let Some(t) = self.title {
            url.query_pairs_mut().append_pair("title", t);
        }

        url
    }
}

/// Request parameters for [`Client::get_user_closed_positions`](crate::data::Client::get_user_closed_positions).
///
/// # Example
///
/// ```no_run
/// use polymarket_hft::data::{Client, GetUserClosedPositionsRequest, ClosedPositionSortBy, SortDirection};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::new();
///     let positions = client.get_user_closed_positions(GetUserClosedPositionsRequest {
///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
///         limit: Some(10),
///         ..Default::default()
///     }).await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetUserClosedPositionsRequest<'a> {
    /// User Profile Address (0x-prefixed, 40 hex chars). Required.
    pub user: &'a str,
    /// Market condition IDs to filter by (0x-prefixed, 64 hex chars each).
    pub markets: Option<&'a [&'a str]>,
    /// Title filter (max 100 chars).
    pub title: Option<&'a str>,
    /// Event IDs to filter by.
    pub event_ids: Option<&'a [i64]>,
    /// Limit for results (0-50, default: 10).
    pub limit: Option<i32>,
    /// Offset for pagination (0-100000, default: 0).
    pub offset: Option<i32>,
    /// Sort field (default: REALIZEDPNL).
    pub sort_by: Option<ClosedPositionSortBy>,
    /// Sort direction (default: DESC).
    pub sort_direction: Option<SortDirection>,
}

impl GetUserClosedPositionsRequest<'_> {
    /// Validates the request parameters.
    pub fn validate(&self) -> Result<()> {
        validate_user(self.user)?;

        if let Some(market_ids) = self.markets {
            for market_id in market_ids {
                validate_market_id(market_id)?;
            }
        }

        if let Some(t) = self.title
            && t.len() > 100
        {
            return Err(PolymarketError::bad_request(
                "title must be at most 100 characters".to_string(),
            ));
        }

        if let Some(ids) = self.event_ids {
            for id in ids {
                validate_event_id(*id)?;
            }
        }

        if let Some(l) = self.limit
            && !(0..=50).contains(&l)
        {
            return Err(PolymarketError::bad_request(
                "limit must be between 0 and 50".to_string(),
            ));
        }

        if let Some(o) = self.offset
            && !(0..=100000).contains(&o)
        {
            return Err(PolymarketError::bad_request(
                "offset must be between 0 and 100000".to_string(),
            ));
        }

        Ok(())
    }

    /// Builds the URL with query parameters for this request.
    pub fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("closed-positions");

        // Required: user parameter
        url.query_pairs_mut().append_pair("user", self.user);

        // Optional: market filter (comma-separated)
        if let Some(market_ids) = self.markets.filter(|ids| !ids.is_empty()) {
            let market_value = market_ids.join(",");
            url.query_pairs_mut().append_pair("market", &market_value);
        }

        // Optional: title
        if let Some(t) = self.title {
            url.query_pairs_mut().append_pair("title", t);
        }

        // Optional: eventId filter (comma-separated)
        if let Some(ids) = self.event_ids.filter(|ids| !ids.is_empty()) {
            let event_value = ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            url.query_pairs_mut().append_pair("eventId", &event_value);
        }

        // Optional: limit
        if let Some(l) = self.limit {
            url.query_pairs_mut().append_pair("limit", &l.to_string());
        }

        // Optional: offset
        if let Some(o) = self.offset {
            url.query_pairs_mut().append_pair("offset", &o.to_string());
        }

        // Optional: sortBy
        if let Some(sort) = self.sort_by {
            url.query_pairs_mut()
                .append_pair("sortBy", &sort.to_string());
        }

        // Optional: sortDirection
        if let Some(dir) = self.sort_direction {
            url.query_pairs_mut()
                .append_pair("sortDirection", &dir.to_string());
        }

        url
    }
}
