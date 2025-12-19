//! Trade-related types and API methods.
//!
//! This module provides types and methods for querying trades.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};
use url::Url;

use super::{Client, TradeSide, validate_event_id, validate_market_id, validate_user};
use crate::error::{PolymarketError, Result};

// ============================================================================
// Types
// ============================================================================

/// Filter type for trades query (CASH or TOKENS).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum TradeFilterType {
    #[serde(rename = "CASH")]
    #[default]
    Cash,
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
            _ => Err(format!("Invalid filter type: '{}'", s)),
        }
    }
}

/// A trade record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    pub side: TradeSide,
    pub asset: String,
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    pub size: f64,
    pub price: f64,
    pub timestamp: i64,
    pub title: String,
    pub slug: String,
    pub icon: String,
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    pub outcome: String,
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    pub name: String,
    pub pseudonym: String,
    pub bio: String,
    #[serde(rename = "profileImage")]
    pub profile_image: String,
    #[serde(rename = "profileImageOptimized")]
    pub profile_image_optimized: String,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
}

/// Response from the traded endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTradedMarketsCount {
    pub user: String,
    pub traded: i64,
}

/// Request parameters for [`Client::get_trades`].
#[derive(Debug, Clone)]
pub struct GetTradesRequest<'a> {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub taker_only: Option<bool>,
    pub filter_type: Option<TradeFilterType>,
    pub filter_amount: Option<f64>,
    pub markets: Option<&'a [&'a str]>,
    pub event_ids: Option<&'a [i64]>,
    pub user: Option<&'a str>,
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
    pub fn validate(&self) -> Result<()> {
        if let Some(l) = self.limit
            && !(0..=10000).contains(&l)
        {
            return Err(PolymarketError::bad_request(
                "limit must be between 0 and 10000".to_string(),
            ));
        }
        if let Some(o) = self.offset
            && !(0..=10000).contains(&o)
        {
            return Err(PolymarketError::bad_request(
                "offset must be between 0 and 10000".to_string(),
            ));
        }
        if self.filter_type.is_some() != self.filter_amount.is_some() {
            return Err(PolymarketError::bad_request(
                "filterType and filterAmount must be provided together".to_string(),
            ));
        }
        if let Some(amount) = self.filter_amount
            && amount < 0.0
        {
            return Err(PolymarketError::bad_request(
                "filterAmount must be >= 0".to_string(),
            ));
        }
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

    pub fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("trades");
        if let Some(l) = self.limit {
            url.query_pairs_mut().append_pair("limit", &l.to_string());
        }
        if let Some(o) = self.offset {
            url.query_pairs_mut().append_pair("offset", &o.to_string());
        }
        if let Some(t) = self.taker_only {
            url.query_pairs_mut()
                .append_pair("takerOnly", &t.to_string());
        }
        if let Some(ft) = self.filter_type {
            url.query_pairs_mut()
                .append_pair("filterType", &ft.to_string());
        }
        if let Some(fa) = self.filter_amount {
            url.query_pairs_mut()
                .append_pair("filterAmount", &fa.to_string());
        }
        if let Some(market_ids) = self.markets.filter(|ids| !ids.is_empty()) {
            url.query_pairs_mut()
                .append_pair("market", &market_ids.join(","));
        }
        if let Some(ids) = self.event_ids.filter(|ids| !ids.is_empty()) {
            let v = ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            url.query_pairs_mut().append_pair("eventId", &v);
        }
        if let Some(u) = self.user {
            url.query_pairs_mut().append_pair("user", u);
        }
        if let Some(s) = self.side {
            url.query_pairs_mut().append_pair("side", &s.to_string());
        }
        url
    }
}

// ============================================================================
// Client Implementation
// ============================================================================

impl Client {
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_trades(&self, request: GetTradesRequest<'_>) -> Result<Vec<Trade>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let trades: Vec<Trade> = response.json().await?;
        trace!(count = trades.len(), "received trades");
        Ok(trades)
    }

    #[instrument(skip(self), fields(user = %user), level = "trace")]
    pub async fn get_user_traded_markets(&self, user: &str) -> Result<UserTradedMarketsCount> {
        validate_user(user)?;
        let mut url = self.build_url("traded");
        url.query_pairs_mut().append_pair("user", user);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let traded_response: UserTradedMarketsCount = response.json().await?;
        trace!(
            traded = traded_response.traded,
            "received traded markets count"
        );
        Ok(traded_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_filter_type_display() {
        assert_eq!(TradeFilterType::Cash.to_string(), "CASH");
        assert_eq!(TradeFilterType::Tokens.to_string(), "TOKENS");
    }

    #[test]
    fn test_trade_filter_type_from_str() {
        assert!(matches!(
            "CASH".parse::<TradeFilterType>(),
            Ok(TradeFilterType::Cash)
        ));
        assert!(matches!(
            "TOKENS".parse::<TradeFilterType>(),
            Ok(TradeFilterType::Tokens)
        ));
        // Case insensitive
        assert!(matches!(
            "cash".parse::<TradeFilterType>(),
            Ok(TradeFilterType::Cash)
        ));
        assert!(matches!(
            "tokens".parse::<TradeFilterType>(),
            Ok(TradeFilterType::Tokens)
        ));
        // Invalid
        assert!("invalid".parse::<TradeFilterType>().is_err());
    }
}
