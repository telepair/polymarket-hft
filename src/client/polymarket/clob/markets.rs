//! CLOB Markets endpoints.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

use super::Client;
use crate::error::Result;

// =============================================================================
// Types
// =============================================================================

/// Token within a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketToken {
    /// Token ID.
    pub token_id: String,
    /// Outcome label.
    pub outcome: String,
    /// Current price (may be int or string or float).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<serde_json::Value>,
    /// Whether this is the winning outcome.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<bool>,
}

/// CLOB Market response.
/// Uses serde_json::Value for flexible fields that may vary in type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    /// Condition ID (market identifier).
    pub condition_id: String,
    /// Question text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// Tokens (outcomes) for this market.
    pub tokens: Vec<MarketToken>,
    /// Rewards configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewards: Option<serde_json::Value>,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Icon URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Whether this is an active market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Whether this market is closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    /// Whether this uses negative risk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neg_risk: Option<bool>,
    /// Market slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_slug: Option<String>,
    /// Catch all other fields for API compatibility.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Request for getting markets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetMarketsRequest {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Paginated markets response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsPaginatedResponse {
    /// List of markets.
    pub data: Vec<Market>,
    /// Next pagination cursor.
    pub next_cursor: String,
    /// Total count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
}

/// Simplified market for sampling endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedMarket {
    /// Condition ID.
    pub condition_id: String,
    /// Tokens.
    pub tokens: Vec<MarketToken>,
}

/// Market trade event from live activity endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTradeEvent {
    /// Event ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Condition ID (market).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_id: Option<String>,
    /// Asset ID (token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    /// Side of the trade.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    /// Trade size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Trade price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// Timestamp of the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Catch all other fields for API compatibility.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

// =============================================================================
// Client Implementation
// =============================================================================

impl Client {
    /// Gets markets with pagination.
    ///
    /// # Arguments
    ///
    /// * `request` - Request with optional pagination cursor.
    ///
    /// # Returns
    ///
    /// Returns a paginated response with markets.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::{Client, GetMarketsRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let response = client.get_markets(GetMarketsRequest::default()).await?;
    ///     println!("Got {} markets", response.data.len());
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), level = "trace")]
    pub async fn get_markets(
        &self,
        request: GetMarketsRequest,
    ) -> Result<MarketsPaginatedResponse> {
        let mut url = self.build_url("markets");

        if let Some(cursor) = &request.next_cursor {
            url.query_pairs_mut().append_pair("next_cursor", cursor);
        }

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let markets: MarketsPaginatedResponse = response.json().await?;
        trace!(count = markets.data.len(), "received markets");
        Ok(markets)
    }

    /// Gets a single market by condition ID.
    ///
    /// # Arguments
    ///
    /// * `condition_id` - The condition ID of the market.
    ///
    /// # Returns
    ///
    /// Returns the market details.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_market(&self, condition_id: &str) -> Result<Market> {
        let url = self.build_url(&format!("markets/{}", condition_id));

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let market: Market = response.json().await?;
        trace!(condition_id = %market.condition_id, "received market");
        Ok(market)
    }

    /// Gets sampling markets (for market making).
    ///
    /// # Arguments
    ///
    /// * `next_cursor` - Optional pagination cursor.
    ///
    /// # Returns
    ///
    /// Returns a paginated response with sampling markets.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_sampling_markets(
        &self,
        next_cursor: Option<&str>,
    ) -> Result<MarketsPaginatedResponse> {
        let mut url = self.build_url("sampling-markets");

        if let Some(cursor) = next_cursor {
            url.query_pairs_mut().append_pair("next_cursor", cursor);
        }

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let markets: MarketsPaginatedResponse = response.json().await?;
        trace!(count = markets.data.len(), "received sampling markets");
        Ok(markets)
    }

    /// Gets simplified sampling markets.
    ///
    /// # Arguments
    ///
    /// * `next_cursor` - Optional pagination cursor.
    ///
    /// # Returns
    ///
    /// Returns a paginated response with simplified markets.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_sampling_simplified_markets(
        &self,
        next_cursor: Option<&str>,
    ) -> Result<MarketsPaginatedResponse> {
        let mut url = self.build_url("sampling-simplified-markets");

        if let Some(cursor) = next_cursor {
            url.query_pairs_mut().append_pair("next_cursor", cursor);
        }

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let markets: MarketsPaginatedResponse = response.json().await?;
        trace!(count = markets.data.len(), "received simplified markets");
        Ok(markets)
    }

    /// Gets simplified markets with pagination.
    ///
    /// Unlike `get_markets()`, this returns a lighter response without full market details.
    ///
    /// # Arguments
    ///
    /// * `next_cursor` - Optional pagination cursor.
    ///
    /// # Returns
    ///
    /// Returns a paginated response with simplified markets.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let response = client.get_simplified_markets(None).await?;
    ///     println!("Got {} simplified markets", response.data.len());
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), level = "trace")]
    pub async fn get_simplified_markets(
        &self,
        next_cursor: Option<&str>,
    ) -> Result<MarketsPaginatedResponse> {
        let mut url = self.build_url("simplified-markets");

        if let Some(cursor) = next_cursor {
            url.query_pairs_mut().append_pair("next_cursor", cursor);
        }

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let markets: MarketsPaginatedResponse = response.json().await?;
        trace!(count = markets.data.len(), "received simplified markets");
        Ok(markets)
    }

    /// Gets market trade events (live activity) for a specific market.
    ///
    /// # Arguments
    ///
    /// * `condition_id` - The condition ID of the market.
    ///
    /// # Returns
    ///
    /// Returns a list of trade events for the market.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let events = client.get_market_trades_events("0x123...").await?;
    ///     println!("Got {} trade events", events.len());
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), level = "trace")]
    pub async fn get_market_trades_events(
        &self,
        condition_id: &str,
    ) -> Result<Vec<MarketTradeEvent>> {
        let url = self.build_url(&format!("live-activity/events/{}", condition_id));

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let events: Vec<MarketTradeEvent> = response.json().await?;
        trace!(count = events.len(), "received market trade events");
        Ok(events)
    }
}
