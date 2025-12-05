//! Trade-related types and request builders.

use serde::{Deserialize, Serialize};
use url::Url;

use super::{TradeSide, validate_event_id, validate_market_id, validate_user};
use crate::error::{PolymarketError, Result};

/// Filter type for trades query (CASH or TOKENS).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum TradeFilterType {
    /// Filter by cash amount.
    #[serde(rename = "CASH")]
    #[default]
    Cash,
    /// Filter by token amount.
    #[serde(rename = "TOKENS")]
    Tokens,
}

impl std::fmt::Display for TradeFilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeFilterType::Cash => write!(f, "CASH"),
            TradeFilterType::Tokens => write!(f, "TOKENS"),
        }
    }
}

impl std::str::FromStr for TradeFilterType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "CASH" => Ok(TradeFilterType::Cash),
            "TOKENS" => Ok(TradeFilterType::Tokens),
            _ => Err(format!(
                "Invalid filter type: '{}'. Valid options: CASH, TOKENS",
                s
            )),
        }
    }
}

/// A trade record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Proxy wallet address (0x-prefixed, 40 hex chars).
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    /// Trade side (BUY or SELL).
    pub side: TradeSide,
    /// Asset identifier.
    pub asset: String,
    /// Condition ID (0x-prefixed, 64 hex string).
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    /// Trade size.
    pub size: f64,
    /// Trade price.
    pub price: f64,
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
    /// Trade outcome.
    pub outcome: String,
    /// Outcome index.
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    /// User name.
    pub name: String,
    /// User pseudonym.
    pub pseudonym: String,
    /// User bio.
    pub bio: String,
    /// Profile image URL.
    #[serde(rename = "profileImage")]
    pub profile_image: String,
    /// Optimized profile image URL.
    #[serde(rename = "profileImageOptimized")]
    pub profile_image_optimized: String,
    /// Transaction hash.
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
}

/// Response from the traded endpoint.
///
/// Contains the total number of markets a user has traded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTradedMarketsCount {
    /// User Profile Address (0x-prefixed, 40 hex chars).
    pub user: String,
    /// Total number of markets the user has traded.
    pub traded: i64,
}

/// Request parameters for [`Client::get_trades`](crate::data::Client::get_trades).
///
/// # Example
///
/// ```no_run
/// use polymarket_hft::data::{Client, GetTradesRequest, TradeSide};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::new();
///     let trades = client.get_trades(GetTradesRequest {
///         user: Some("0x56687bf447db6ffa42ffe2204a05edaa20f55839"),
///         limit: Some(50),
///         side: Some(TradeSide::Buy),
///         ..Default::default()
///     }).await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GetTradesRequest<'a> {
    /// Limit for results (0-10000, default: 100).
    pub limit: Option<i32>,
    /// Offset for pagination (0-10000, default: 0).
    pub offset: Option<i32>,
    /// Filter for taker-only trades (default: true).
    pub taker_only: Option<bool>,
    /// Filter type (CASH or TOKENS). Must be provided with filter_amount.
    pub filter_type: Option<TradeFilterType>,
    /// Filter amount (>= 0). Must be provided with filter_type.
    pub filter_amount: Option<f64>,
    /// Market condition IDs to filter by. Mutually exclusive with event_ids.
    pub markets: Option<&'a [&'a str]>,
    /// Event IDs to filter by. Mutually exclusive with markets.
    pub event_ids: Option<&'a [i64]>,
    /// User address to filter by (0x-prefixed, 40 hex chars).
    pub user: Option<&'a str>,
    /// Trade side filter.
    pub side: Option<TradeSide>,
}

impl Default for GetTradesRequest<'_> {
    fn default() -> Self {
        Self {
            limit: None,
            offset: None,
            taker_only: Some(true),
            filter_type: None,
            filter_amount: None,
            markets: None,
            event_ids: None,
            user: None,
            side: None,
        }
    }
}

impl GetTradesRequest<'_> {
    /// Validates the request parameters.
    pub fn validate(&self) -> Result<()> {
        // Validate limit (0-10000)
        if let Some(l) = self.limit
            && !(0..=10000).contains(&l)
        {
            return Err(PolymarketError::bad_request(
                "limit must be between 0 and 10000".to_string(),
            ));
        }

        // Validate offset (0-10000)
        if let Some(o) = self.offset
            && !(0..=10000).contains(&o)
        {
            return Err(PolymarketError::bad_request(
                "offset must be between 0 and 10000".to_string(),
            ));
        }

        // filter_type and filter_amount must be provided together
        if self.filter_type.is_some() != self.filter_amount.is_some() {
            return Err(PolymarketError::bad_request(
                "filterType and filterAmount must be provided together".to_string(),
            ));
        }

        // Validate filter_amount (>= 0)
        if let Some(amount) = self.filter_amount
            && amount < 0.0
        {
            return Err(PolymarketError::bad_request(
                "filterAmount must be >= 0".to_string(),
            ));
        }

        // markets and event_ids are mutually exclusive
        if self.markets.map(|m| !m.is_empty()).unwrap_or(false)
            && self.event_ids.map(|e| !e.is_empty()).unwrap_or(false)
        {
            return Err(PolymarketError::bad_request(
                "market and eventId are mutually exclusive".to_string(),
            ));
        }

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

        if let Some(u) = self.user {
            validate_user(u)?;
        }

        Ok(())
    }

    /// Builds the URL with query parameters for this request.
    pub fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("trades");

        // Optional: limit
        if let Some(l) = self.limit {
            url.query_pairs_mut().append_pair("limit", &l.to_string());
        }

        // Optional: offset
        if let Some(o) = self.offset {
            url.query_pairs_mut().append_pair("offset", &o.to_string());
        }

        // Optional: takerOnly
        if let Some(t) = self.taker_only {
            url.query_pairs_mut()
                .append_pair("takerOnly", &t.to_string());
        }

        // Optional: filterType
        if let Some(ft) = self.filter_type {
            url.query_pairs_mut()
                .append_pair("filterType", &ft.to_string());
        }

        // Optional: filterAmount
        if let Some(fa) = self.filter_amount {
            url.query_pairs_mut()
                .append_pair("filterAmount", &fa.to_string());
        }

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

        // Optional: user
        if let Some(u) = self.user {
            url.query_pairs_mut().append_pair("user", u);
        }

        // Optional: side
        if let Some(s) = self.side {
            url.query_pairs_mut().append_pair("side", &s.to_string());
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_USER: &str = "0x0123456789012345678901234567890123456789";
    const VALID_MARKET: &str = "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917";

    #[test]
    fn default_sets_taker_only_true() {
        let req = GetTradesRequest::default();
        assert_eq!(req.taker_only, Some(true));
    }

    #[test]
    fn build_url_includes_taker_only_by_default() {
        let base = Url::parse("https://example.com").unwrap();
        let url = GetTradesRequest::default().build_url(&base);
        let query = url.query().unwrap_or_default();
        assert!(
            query.contains("takerOnly=true"),
            "expected takerOnly in query, got {query}"
        );
    }

    #[test]
    fn validate_requires_filter_amount_with_filter_type() {
        let req = GetTradesRequest {
            filter_type: Some(TradeFilterType::Cash),
            ..Default::default()
        };

        let err = req.validate().unwrap_err();
        assert!(
            err.to_string().contains("filterAmount"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_rejects_negative_filter_amount() {
        let req = GetTradesRequest {
            filter_type: Some(TradeFilterType::Cash),
            filter_amount: Some(-1.0),
            ..Default::default()
        };

        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains(">= 0"), "unexpected error: {err}");
    }

    #[test]
    fn validate_rejects_markets_and_event_ids_together() {
        let markets = &[VALID_MARKET];
        let event_ids = &[1_i64];
        let req = GetTradesRequest {
            markets: Some(markets),
            event_ids: Some(event_ids),
            ..Default::default()
        };

        let err = req.validate().unwrap_err();
        assert!(
            err.to_string().contains("mutually exclusive"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn build_url_includes_filters() {
        let base = Url::parse("https://example.com").unwrap();
        let markets = &[VALID_MARKET];
        let url = GetTradesRequest {
            limit: Some(5),
            offset: Some(3),
            taker_only: Some(true),
            filter_type: Some(TradeFilterType::Cash),
            filter_amount: Some(10.5),
            markets: Some(markets),
            user: Some(VALID_USER),
            side: Some(TradeSide::Sell),
            ..Default::default()
        }
        .build_url(&base);

        let query = url.query().unwrap_or_default();
        for expected in [
            "limit=5",
            "offset=3",
            "takerOnly=true",
            "filterType=CASH",
            "filterAmount=10.5",
            &format!("market={VALID_MARKET}"),
            &format!("user={VALID_USER}"),
            "side=SELL",
        ] {
            assert!(
                query.contains(expected),
                "missing '{expected}' in query: {query}"
            );
        }
    }
}
