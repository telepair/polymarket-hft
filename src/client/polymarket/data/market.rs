//! Market data types and API methods.
//!
//! This module provides types and methods for querying market open interest and live volume.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

use super::{Client, validate_event_id, validate_market_id};
use crate::error::Result;

// ============================================================================
// Types
// ============================================================================

/// Response item from the open interest endpoint.
///
/// Represents the open interest value for a specific market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOpenInterest {
    /// Market ID (0x-prefixed, 64 hex chars).
    pub market: String,
    /// The open interest value for this market.
    pub value: f64,
}

/// Market volume data within a live volume response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketLiveVolume {
    /// Market ID (0x-prefixed, 64 hex chars).
    pub market: String,
    /// The volume value for this market.
    pub value: f64,
}

/// Response from the live volume endpoint.
///
/// Contains the total volume and per-market breakdown for an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLiveVolume {
    /// Total volume across all markets.
    pub total: f64,
    /// Volume breakdown by market (None if no markets have volume).
    pub markets: Option<Vec<MarketLiveVolume>>,
}

// ============================================================================
// Client Implementation
// ============================================================================

impl Client {
    /// Gets the open interest for the specified markets.
    ///
    /// # Arguments
    ///
    /// * `markets` - A slice of market IDs (0x-prefixed, 64 hex chars each).
    ///
    /// # Returns
    ///
    /// Returns a vector of `MarketOpenInterest` containing the market ID and its open interest value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::data::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let markets = vec![
    ///         "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",
    ///     ];
    ///     let oi = client.get_open_interest(&markets).await?;
    ///     for item in oi {
    ///         println!("Market {} has open interest: {}", item.market, item.value);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self, markets), level = "trace")]
    pub async fn get_open_interest(&self, markets: &[&str]) -> Result<Vec<MarketOpenInterest>> {
        // Validate all market IDs
        for market_id in markets {
            validate_market_id(market_id)?;
        }

        let mut url = self.build_url("oi");

        // Add market query parameter (comma-separated)
        if !markets.is_empty() {
            let market_value = markets.join(",");
            url.query_pairs_mut().append_pair("market", &market_value);
        }

        trace!(url = %url, method = "GET", market_count = markets.len(), "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let open_interest_response: Vec<MarketOpenInterest> = response.json().await?;
        trace!(
            count = open_interest_response.len(),
            "received open interest data"
        );
        Ok(open_interest_response)
    }

    /// Gets the live volume for an event.
    ///
    /// # Arguments
    ///
    /// * `event_id` - The event ID (must be >= 1).
    ///
    /// # Returns
    ///
    /// Returns a `EventLiveVolume` containing the total volume and per-market breakdown.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::data::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let volume = client.get_event_live_volume(123).await?;
    ///     println!("Total volume: {}", volume.total);
    ///     if let Some(markets) = &volume.markets {
    ///         for market in markets {
    ///             println!("Market {} volume: {}", market.market, market.value);
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), fields(event_id = %event_id), level = "trace")]
    pub async fn get_event_live_volume(&self, event_id: i64) -> Result<EventLiveVolume> {
        validate_event_id(event_id)?;

        let mut url = self.build_url("live-volume");
        url.query_pairs_mut()
            .append_pair("id", &event_id.to_string());

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        // API returns an array, we take the first element
        let volume_responses: Vec<EventLiveVolume> = response.json().await?;
        let result = volume_responses.into_iter().next().ok_or_else(|| {
            crate::error::PolymarketError::api("empty live volume response".to_string())
        })?;
        trace!(total = result.total, "received live volume");
        Ok(result)
    }
}
