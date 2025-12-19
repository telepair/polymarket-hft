//! Activity-related types and API methods.
//!
//! This module provides types and methods for querying user on-chain activity.

use serde::{Deserialize, Deserializer, Serialize};
use tracing::{instrument, trace};
use url::Url;

use super::{
    Client, SortDirection, TradeSide, validate_event_id, validate_limit, validate_market_id,
    validate_user,
};
use crate::error::{PolymarketError, Result};

// ============================================================================
// Types
// ============================================================================

/// Deserialize TradeSide, treating empty strings as None.
fn deserialize_optional_trade_side<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<TradeSide>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(ref val) if val.is_empty() => Ok(None),
        Some(val) => match val.as_str() {
            "BUY" => Ok(Some(TradeSide::Buy)),
            "SELL" => Ok(Some(TradeSide::Sell)),
            _ => Ok(None),
        },
        None => Ok(None),
    }
}

/// Activity type enum for user activity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActivityType {
    /// Trade activity.
    #[serde(rename = "TRADE")]
    Trade,
    /// Split activity.
    #[serde(rename = "SPLIT")]
    Split,
    /// Merge activity.
    #[serde(rename = "MERGE")]
    Merge,
    /// Redeem activity.
    #[serde(rename = "REDEEM")]
    Redeem,
    /// Reward activity.
    #[serde(rename = "REWARD")]
    Reward,
    /// Conversion activity.
    #[serde(rename = "CONVERSION")]
    Conversion,
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityType::Trade => write!(f, "TRADE"),
            ActivityType::Split => write!(f, "SPLIT"),
            ActivityType::Merge => write!(f, "MERGE"),
            ActivityType::Redeem => write!(f, "REDEEM"),
            ActivityType::Reward => write!(f, "REWARD"),
            ActivityType::Conversion => write!(f, "CONVERSION"),
        }
    }
}

impl std::str::FromStr for ActivityType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TRADE" => Ok(ActivityType::Trade),
            "SPLIT" => Ok(ActivityType::Split),
            "MERGE" => Ok(ActivityType::Merge),
            "REDEEM" => Ok(ActivityType::Redeem),
            "REWARD" => Ok(ActivityType::Reward),
            "CONVERSION" => Ok(ActivityType::Conversion),
            _ => Err(format!(
                "Invalid activity type: '{}'. Valid options: TRADE, SPLIT, MERGE, REDEEM, REWARD, CONVERSION",
                s
            )),
        }
    }
}

/// Sort by options for activity query.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum ActivitySortBy {
    /// Sort by timestamp (default).
    #[serde(rename = "TIMESTAMP")]
    #[default]
    Timestamp,
    /// Sort by token count.
    #[serde(rename = "TOKENS")]
    Tokens,
    /// Sort by cash amount.
    #[serde(rename = "CASH")]
    Cash,
}

impl std::fmt::Display for ActivitySortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivitySortBy::Timestamp => write!(f, "TIMESTAMP"),
            ActivitySortBy::Tokens => write!(f, "TOKENS"),
            ActivitySortBy::Cash => write!(f, "CASH"),
        }
    }
}

impl std::str::FromStr for ActivitySortBy {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TIMESTAMP" => Ok(ActivitySortBy::Timestamp),
            "TOKENS" => Ok(ActivitySortBy::Tokens),
            "CASH" => Ok(ActivitySortBy::Cash),
            _ => Err(format!(
                "Invalid sort by: '{}'. Valid options: TIMESTAMP, TOKENS, CASH",
                s
            )),
        }
    }
}

/// A user activity record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// Proxy wallet address (0x-prefixed, 40 hex chars).
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    /// Unix timestamp.
    pub timestamp: i64,
    /// Condition ID (0x-prefixed, 64 hex string).
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    /// Activity type.
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    /// Activity size.
    pub size: f64,
    /// USDC size.
    #[serde(rename = "usdcSize")]
    pub usdc_size: f64,
    /// Transaction hash.
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    /// Price.
    pub price: f64,
    /// Asset identifier.
    pub asset: String,
    /// Trade side (BUY or SELL), None for non-trade activities.
    #[serde(default, deserialize_with = "deserialize_optional_trade_side")]
    pub side: Option<TradeSide>,
    /// Outcome index.
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    /// Market title.
    pub title: String,
    /// Market slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Event slug.
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    /// Activity outcome.
    pub outcome: String,
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
}

/// Request parameters for [`Client::get_user_activity`].
///
/// # Example
///
/// ```no_run
/// use polymarket_hft::client::polymarket::data::{Client, GetUserActivityRequest, ActivityType, ActivitySortBy, SortDirection};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::new();
///     let activity_types = vec![ActivityType::Trade];
///     let activity = client.get_user_activity(GetUserActivityRequest {
///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
///         limit: Some(50),
///         activity_types: Some(&activity_types),
///         ..Default::default()
///     }).await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetUserActivityRequest<'a> {
    /// User Profile Address (0x-prefixed, 40 hex chars). Required.
    pub user: &'a str,
    /// Limit for results (0-500, default: 100).
    pub limit: Option<i32>,
    /// Offset for pagination (0-10000, default: 0).
    pub offset: Option<i32>,
    /// Market condition IDs to filter by. Mutually exclusive with event_ids.
    pub markets: Option<&'a [&'a str]>,
    /// Event IDs to filter by. Mutually exclusive with markets.
    pub event_ids: Option<&'a [i64]>,
    /// Activity types to filter by.
    pub activity_types: Option<&'a [ActivityType]>,
    /// Start timestamp (>= 0).
    pub start: Option<i64>,
    /// End timestamp (>= 0).
    pub end: Option<i64>,
    /// Sort field (default: TIMESTAMP).
    pub sort_by: Option<ActivitySortBy>,
    /// Sort direction (default: DESC).
    pub sort_direction: Option<SortDirection>,
    /// Trade side filter.
    pub side: Option<TradeSide>,
}

impl GetUserActivityRequest<'_> {
    /// Validates the request parameters.
    pub fn validate(&self) -> Result<()> {
        validate_user(self.user)?;
        validate_limit(self.limit)?;

        if let Some(o) = self.offset
            && !(0..=10000).contains(&o)
        {
            return Err(PolymarketError::bad_request(
                "offset must be between 0 and 10000".to_string(),
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

        if let Some(s) = self.start
            && s < 0
        {
            return Err(PolymarketError::bad_request(
                "start must be >= 0".to_string(),
            ));
        }

        if let Some(e) = self.end
            && e < 0
        {
            return Err(PolymarketError::bad_request("end must be >= 0".to_string()));
        }

        if let (Some(s), Some(e)) = (self.start, self.end)
            && s > e
        {
            return Err(PolymarketError::bad_request(
                "start must be <= end".to_string(),
            ));
        }

        Ok(())
    }

    /// Builds the URL with query parameters for this request.
    pub fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("activity");

        // Required: user parameter
        url.query_pairs_mut().append_pair("user", self.user);

        // Optional: limit
        if let Some(l) = self.limit {
            url.query_pairs_mut().append_pair("limit", &l.to_string());
        }

        // Optional: offset
        if let Some(o) = self.offset {
            url.query_pairs_mut().append_pair("offset", &o.to_string());
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

        // Optional: type filter (comma-separated)
        if let Some(types) = self.activity_types.filter(|t| !t.is_empty()) {
            let type_value = types
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(",");
            url.query_pairs_mut().append_pair("type", &type_value);
        }

        // Optional: start
        if let Some(s) = self.start {
            url.query_pairs_mut().append_pair("start", &s.to_string());
        }

        // Optional: end
        if let Some(e) = self.end {
            url.query_pairs_mut().append_pair("end", &e.to_string());
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

        // Optional: side
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
    /// Gets the on-chain activity for a user.
    ///
    /// # Arguments
    ///
    /// * `request` - Request parameters. See [`GetUserActivityRequest`] for details.
    ///
    /// # Returns
    ///
    /// Returns a vector of `Activity` containing the user's on-chain activity.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::data::{Client, GetUserActivityRequest, ActivityType, ActivitySortBy, SortDirection};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     
    ///     // Get all activity for a user
    ///     let activity = client.get_user_activity(GetUserActivityRequest {
    ///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ///         ..Default::default()
    ///     }).await?;
    ///     
    ///     for act in &activity {
    ///         println!("Activity: {:?} - Size: {} - Price: {}", act.activity_type, act.size, act.price);
    ///     }
    ///     
    ///     // Get activity with filters
    ///     let activity_types = vec![ActivityType::Trade];
    ///     let activity = client.get_user_activity(GetUserActivityRequest {
    ///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ///         limit: Some(50),
    ///         activity_types: Some(&activity_types),
    ///         sort_by: Some(ActivitySortBy::Timestamp),
    ///         sort_direction: Some(SortDirection::Desc),
    ///         ..Default::default()
    ///     }).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self, request), fields(user = %request.user), level = "trace")]
    pub async fn get_user_activity(
        &self,
        request: GetUserActivityRequest<'_>,
    ) -> Result<Vec<Activity>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let activity: Vec<Activity> = response.json().await?;
        trace!(count = activity.len(), "received activity records");
        Ok(activity)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_USER: &str = "0x0123456789012345678901234567890123456789";
    const VALID_MARKET: &str = "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917";

    #[test]
    fn validate_rejects_start_greater_than_end() {
        let req = GetUserActivityRequest {
            user: "0x0123456789012345678901234567890123456789",
            start: Some(11),
            end: Some(10),
            ..Default::default()
        };

        let err = req.validate().unwrap_err();
        assert!(
            err.to_string().contains("start must be <="),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_rejects_offset_over_limit() {
        let req = GetUserActivityRequest {
            user: VALID_USER,
            offset: Some(10001),
            ..Default::default()
        };

        let err = req.validate().unwrap_err();
        assert!(
            err.to_string()
                .contains("offset must be between 0 and 10000"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_rejects_market_and_event_together() {
        let markets = &[VALID_MARKET];
        let event_ids = &[1_i64];
        let req = GetUserActivityRequest {
            user: VALID_USER,
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
    fn validate_rejects_negative_start() {
        let req = GetUserActivityRequest {
            user: VALID_USER,
            start: Some(-5),
            ..Default::default()
        };

        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("start must be >="));
    }

    #[test]
    fn build_url_serializes_filters() {
        let base = Url::parse("https://example.com").unwrap();
        let markets = &[VALID_MARKET];
        let activity_types = &[ActivityType::Trade];
        let url = GetUserActivityRequest {
            user: VALID_USER,
            limit: Some(50),
            offset: Some(5),
            markets: Some(markets),
            activity_types: Some(activity_types),
            start: Some(100),
            end: Some(200),
            sort_by: Some(ActivitySortBy::Cash),
            sort_direction: Some(SortDirection::Asc),
            side: Some(TradeSide::Sell),
            ..Default::default()
        }
        .build_url(&base);

        let query = url.query().unwrap_or_default();
        for expected in [
            &format!("user={VALID_USER}"),
            "limit=50",
            "offset=5",
            &format!("market={VALID_MARKET}"),
            "type=TRADE",
            "start=100",
            "end=200",
            "sortBy=CASH",
            "sortDirection=ASC",
            "side=SELL",
        ] {
            assert!(
                query.contains(expected),
                "missing '{expected}' in query: {query}"
            );
        }
    }

    #[test]
    fn test_activity_type_display() {
        assert_eq!(ActivityType::Trade.to_string(), "TRADE");
        assert_eq!(ActivityType::Split.to_string(), "SPLIT");
        assert_eq!(ActivityType::Merge.to_string(), "MERGE");
        assert_eq!(ActivityType::Redeem.to_string(), "REDEEM");
        assert_eq!(ActivityType::Reward.to_string(), "REWARD");
        assert_eq!(ActivityType::Conversion.to_string(), "CONVERSION");
    }

    #[test]
    fn test_activity_type_from_str() {
        assert!(matches!(
            "TRADE".parse::<ActivityType>(),
            Ok(ActivityType::Trade)
        ));
        assert!(matches!(
            "SPLIT".parse::<ActivityType>(),
            Ok(ActivityType::Split)
        ));
        assert!(matches!(
            "MERGE".parse::<ActivityType>(),
            Ok(ActivityType::Merge)
        ));
        assert!(matches!(
            "REDEEM".parse::<ActivityType>(),
            Ok(ActivityType::Redeem)
        ));
        assert!(matches!(
            "REWARD".parse::<ActivityType>(),
            Ok(ActivityType::Reward)
        ));
        assert!(matches!(
            "CONVERSION".parse::<ActivityType>(),
            Ok(ActivityType::Conversion)
        ));
        // Case insensitive
        assert!(matches!(
            "trade".parse::<ActivityType>(),
            Ok(ActivityType::Trade)
        ));
        // Invalid
        assert!("invalid".parse::<ActivityType>().is_err());
    }

    #[test]
    fn test_activity_sort_by_display() {
        assert_eq!(ActivitySortBy::Timestamp.to_string(), "TIMESTAMP");
        assert_eq!(ActivitySortBy::Tokens.to_string(), "TOKENS");
        assert_eq!(ActivitySortBy::Cash.to_string(), "CASH");
    }

    #[test]
    fn test_activity_sort_by_from_str() {
        assert!(matches!(
            "TIMESTAMP".parse::<ActivitySortBy>(),
            Ok(ActivitySortBy::Timestamp)
        ));
        assert!(matches!(
            "TOKENS".parse::<ActivitySortBy>(),
            Ok(ActivitySortBy::Tokens)
        ));
        assert!(matches!(
            "CASH".parse::<ActivitySortBy>(),
            Ok(ActivitySortBy::Cash)
        ));
        // Case insensitive
        assert!(matches!(
            "timestamp".parse::<ActivitySortBy>(),
            Ok(ActivitySortBy::Timestamp)
        ));
        // Invalid
        assert!("invalid".parse::<ActivitySortBy>().is_err());
    }
}
