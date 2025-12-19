//! Position-related types and API methods.
//!
//! This module provides types and methods for querying user positions.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};
use url::Url;

use super::{
    Client, SortDirection, validate_event_id, validate_limit, validate_market_id, validate_user,
};
use crate::error::{PolymarketError, Result};

// ============================================================================
// Types
// ============================================================================

/// Sort by options for positions query.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum PositionSortBy {
    #[serde(rename = "CURRENT")]
    Current,
    #[serde(rename = "INITIAL")]
    Initial,
    #[serde(rename = "TOKENS")]
    #[default]
    Tokens,
    #[serde(rename = "CASHPNL")]
    CashPnl,
    #[serde(rename = "PERCENTPNL")]
    PercentPnl,
    #[serde(rename = "TITLE")]
    Title,
    #[serde(rename = "RESOLVING")]
    Resolving,
    #[serde(rename = "PRICE")]
    Price,
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
            _ => Err(format!("Invalid sort by: '{}'", s)),
        }
    }
}

/// A user's position in a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    pub asset: String,
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    pub size: f64,
    #[serde(rename = "avgPrice")]
    pub avg_price: f64,
    #[serde(rename = "initialValue")]
    pub initial_value: f64,
    #[serde(rename = "currentValue")]
    pub current_value: f64,
    #[serde(rename = "cashPnl")]
    pub cash_pnl: f64,
    #[serde(rename = "percentPnl")]
    pub percent_pnl: f64,
    #[serde(rename = "totalBought")]
    pub total_bought: f64,
    #[serde(rename = "realizedPnl")]
    pub realized_pnl: f64,
    #[serde(rename = "percentRealizedPnl")]
    pub percent_realized_pnl: f64,
    #[serde(rename = "curPrice")]
    pub cur_price: f64,
    pub redeemable: bool,
    pub mergeable: bool,
    pub title: String,
    pub slug: String,
    pub icon: String,
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    pub outcome: String,
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    #[serde(rename = "oppositeOutcome")]
    pub opposite_outcome: String,
    #[serde(rename = "oppositeAsset")]
    pub opposite_asset: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
    #[serde(rename = "negativeRisk")]
    pub negative_risk: bool,
}

/// Sort by options for closed positions query.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum ClosedPositionSortBy {
    #[serde(rename = "REALIZEDPNL")]
    #[default]
    RealizedPnl,
    #[serde(rename = "TITLE")]
    Title,
    #[serde(rename = "PRICE")]
    Price,
    #[serde(rename = "AVGPRICE")]
    AvgPrice,
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
            _ => Err(format!("Invalid sort by: '{}'", s)),
        }
    }
}

/// A user's closed position in a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedPosition {
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    pub asset: String,
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    #[serde(rename = "avgPrice")]
    pub avg_price: f64,
    #[serde(rename = "totalBought")]
    pub total_bought: f64,
    #[serde(rename = "realizedPnl")]
    pub realized_pnl: f64,
    #[serde(rename = "curPrice")]
    pub cur_price: f64,
    pub timestamp: i64,
    pub title: String,
    pub slug: String,
    pub icon: String,
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    pub outcome: String,
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    #[serde(rename = "oppositeOutcome")]
    pub opposite_outcome: String,
    #[serde(rename = "oppositeAsset")]
    pub opposite_asset: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
}

/// Response from the value endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPositionValue {
    pub user: String,
    pub value: f64,
}

/// Request parameters for [`Client::get_user_positions`].
#[derive(Debug, Clone, Default)]
pub struct GetUserPositionsRequest<'a> {
    pub user: &'a str,
    pub markets: Option<&'a [&'a str]>,
    pub event_ids: Option<&'a [i64]>,
    pub size_threshold: Option<f64>,
    pub redeemable: Option<bool>,
    pub mergeable: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub sort_by: Option<PositionSortBy>,
    pub sort_direction: Option<SortDirection>,
    pub title: Option<&'a str>,
}

impl GetUserPositionsRequest<'_> {
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

    pub fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("positions");
        url.query_pairs_mut().append_pair("user", self.user);
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
        if let Some(t) = self.size_threshold {
            url.query_pairs_mut()
                .append_pair("sizeThreshold", &t.to_string());
        }
        if let Some(r) = self.redeemable {
            url.query_pairs_mut()
                .append_pair("redeemable", &r.to_string());
        }
        if let Some(m) = self.mergeable {
            url.query_pairs_mut()
                .append_pair("mergeable", &m.to_string());
        }
        if let Some(l) = self.limit {
            url.query_pairs_mut().append_pair("limit", &l.to_string());
        }
        if let Some(o) = self.offset {
            url.query_pairs_mut().append_pair("offset", &o.to_string());
        }
        if let Some(sort) = self.sort_by {
            url.query_pairs_mut()
                .append_pair("sortBy", &sort.to_string());
        }
        if let Some(dir) = self.sort_direction {
            url.query_pairs_mut()
                .append_pair("sortDirection", &dir.to_string());
        }
        if let Some(t) = self.title {
            url.query_pairs_mut().append_pair("title", t);
        }
        url
    }
}

/// Request parameters for [`Client::get_user_closed_positions`].
#[derive(Debug, Clone, Default)]
pub struct GetUserClosedPositionsRequest<'a> {
    pub user: &'a str,
    pub markets: Option<&'a [&'a str]>,
    pub title: Option<&'a str>,
    pub event_ids: Option<&'a [i64]>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub sort_by: Option<ClosedPositionSortBy>,
    pub sort_direction: Option<SortDirection>,
}

impl GetUserClosedPositionsRequest<'_> {
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

    pub fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("closed-positions");
        url.query_pairs_mut().append_pair("user", self.user);
        if let Some(market_ids) = self.markets.filter(|ids| !ids.is_empty()) {
            url.query_pairs_mut()
                .append_pair("market", &market_ids.join(","));
        }
        if let Some(t) = self.title {
            url.query_pairs_mut().append_pair("title", t);
        }
        if let Some(ids) = self.event_ids.filter(|ids| !ids.is_empty()) {
            let v = ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            url.query_pairs_mut().append_pair("eventId", &v);
        }
        if let Some(l) = self.limit {
            url.query_pairs_mut().append_pair("limit", &l.to_string());
        }
        if let Some(o) = self.offset {
            url.query_pairs_mut().append_pair("offset", &o.to_string());
        }
        if let Some(sort) = self.sort_by {
            url.query_pairs_mut()
                .append_pair("sortBy", &sort.to_string());
        }
        if let Some(dir) = self.sort_direction {
            url.query_pairs_mut()
                .append_pair("sortDirection", &dir.to_string());
        }
        url
    }
}

// ============================================================================
// Client Implementation
// ============================================================================

impl Client {
    #[instrument(skip(self, request), fields(user = %request.user), level = "trace")]
    pub async fn get_user_positions(
        &self,
        request: GetUserPositionsRequest<'_>,
    ) -> Result<Vec<Position>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let positions: Vec<Position> = response.json().await?;
        trace!(count = positions.len(), "received positions");
        Ok(positions)
    }

    #[instrument(skip(self, request), fields(user = %request.user), level = "trace")]
    pub async fn get_user_closed_positions(
        &self,
        request: GetUserClosedPositionsRequest<'_>,
    ) -> Result<Vec<ClosedPosition>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let positions: Vec<ClosedPosition> = response.json().await?;
        trace!(count = positions.len(), "received closed positions");
        Ok(positions)
    }

    #[instrument(skip(self, markets), fields(user = %user), level = "trace")]
    pub async fn get_user_portfolio_value(
        &self,
        user: &str,
        markets: Option<&[&str]>,
    ) -> Result<Vec<UserPositionValue>> {
        validate_user(user)?;
        if let Some(market_ids) = markets {
            for market_id in market_ids {
                validate_market_id(market_id)?;
            }
        }
        let mut url = self.build_url("value");
        url.query_pairs_mut().append_pair("user", user);
        if let Some(market_ids) = markets.filter(|ids| !ids.is_empty()) {
            url.query_pairs_mut()
                .append_pair("market", &market_ids.join(","));
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let value_response: Vec<UserPositionValue> = response.json().await?;
        trace!(count = value_response.len(), "received portfolio values");
        Ok(value_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_sort_by_display() {
        assert_eq!(PositionSortBy::Current.to_string(), "CURRENT");
        assert_eq!(PositionSortBy::Tokens.to_string(), "TOKENS");
        assert_eq!(PositionSortBy::CashPnl.to_string(), "CASHPNL");
    }

    #[test]
    fn test_position_sort_by_from_str() {
        assert!(matches!(
            "CURRENT".parse::<PositionSortBy>(),
            Ok(PositionSortBy::Current)
        ));
        assert!(matches!(
            "INITIAL".parse::<PositionSortBy>(),
            Ok(PositionSortBy::Initial)
        ));
        assert!(matches!(
            "TOKENS".parse::<PositionSortBy>(),
            Ok(PositionSortBy::Tokens)
        ));
        assert!(matches!(
            "CASHPNL".parse::<PositionSortBy>(),
            Ok(PositionSortBy::CashPnl)
        ));
        assert!(matches!(
            "PERCENTPNL".parse::<PositionSortBy>(),
            Ok(PositionSortBy::PercentPnl)
        ));
        assert!(matches!(
            "TITLE".parse::<PositionSortBy>(),
            Ok(PositionSortBy::Title)
        ));
        assert!(matches!(
            "RESOLVING".parse::<PositionSortBy>(),
            Ok(PositionSortBy::Resolving)
        ));
        assert!(matches!(
            "PRICE".parse::<PositionSortBy>(),
            Ok(PositionSortBy::Price)
        ));
        assert!(matches!(
            "AVGPRICE".parse::<PositionSortBy>(),
            Ok(PositionSortBy::AvgPrice)
        ));
        // Case insensitive
        assert!(matches!(
            "current".parse::<PositionSortBy>(),
            Ok(PositionSortBy::Current)
        ));
        // Invalid
        assert!("invalid".parse::<PositionSortBy>().is_err());
    }

    #[test]
    fn test_closed_position_sort_by_display() {
        assert_eq!(ClosedPositionSortBy::RealizedPnl.to_string(), "REALIZEDPNL");
        assert_eq!(ClosedPositionSortBy::Timestamp.to_string(), "TIMESTAMP");
    }

    #[test]
    fn test_closed_position_sort_by_from_str() {
        assert!(matches!(
            "REALIZEDPNL".parse::<ClosedPositionSortBy>(),
            Ok(ClosedPositionSortBy::RealizedPnl)
        ));
        assert!(matches!(
            "TITLE".parse::<ClosedPositionSortBy>(),
            Ok(ClosedPositionSortBy::Title)
        ));
        assert!(matches!(
            "PRICE".parse::<ClosedPositionSortBy>(),
            Ok(ClosedPositionSortBy::Price)
        ));
        assert!(matches!(
            "AVGPRICE".parse::<ClosedPositionSortBy>(),
            Ok(ClosedPositionSortBy::AvgPrice)
        ));
        assert!(matches!(
            "TIMESTAMP".parse::<ClosedPositionSortBy>(),
            Ok(ClosedPositionSortBy::Timestamp)
        ));
        // Case insensitive
        assert!(matches!(
            "realizedpnl".parse::<ClosedPositionSortBy>(),
            Ok(ClosedPositionSortBy::RealizedPnl)
        ));
        // Invalid
        assert!("invalid".parse::<ClosedPositionSortBy>().is_err());
    }
}
