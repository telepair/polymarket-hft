//! Holders-related types and API methods.
//!
//! This module provides types and methods for querying market top holders.

use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};
use tracing::{instrument, trace};

use super::{Client, validate_limit, validate_market_id, validate_min_balance};
use crate::error::Result;

// ============================================================================
// Types
// ============================================================================

/// A holder's position information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holder {
    /// Proxy wallet address.
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    /// User bio.
    pub bio: String,
    /// Asset identifier.
    pub asset: String,
    /// User pseudonym.
    pub pseudonym: String,
    /// Position amount.
    pub amount: f64,
    /// Whether username is public.
    #[serde(rename = "displayUsernamePublic")]
    pub display_username_public: bool,
    /// Outcome index.
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    /// User name.
    pub name: String,
    /// Profile image URL.
    #[serde(rename = "profileImage")]
    pub profile_image: String,
    /// Optimized profile image URL.
    #[serde(rename = "profileImageOptimized")]
    pub profile_image_optimized: String,
}

/// Response from the holders endpoint.
///
/// Contains the token identifier and list of holders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTopHolders {
    /// Token identifier.
    pub token: String,
    /// List of holders for this token.
    pub holders: Vec<Holder>,
}

// ============================================================================
// Client Implementation
// ============================================================================

const MAX_RETRIES: usize = 2;
const RETRY_DELAY_MS: u64 = 300;

fn is_retryable_status(status: u16) -> bool {
    status == 408 || status == 429 || (500..=504).contains(&status)
}

impl Client {
    /// Gets the top holders for the specified markets.
    ///
    /// # Arguments
    ///
    /// * `markets` - A slice of market IDs (0x-prefixed, 64 hex chars each).
    /// * `limit` - Optional limit for results (0-500, default: 100).
    /// * `min_balance` - Optional minimum balance filter (0-999999, default: 1).
    ///
    /// # Returns
    ///
    /// Returns a vector of `MarketTopHolders` containing token and holder information.
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
    ///     let holders = client.get_market_top_holders(&markets, Some(10), None).await?;
    ///     for item in holders {
    ///         println!("Token {} has {} holders", item.token, item.holders.len());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self, markets), level = "trace")]
    pub async fn get_market_top_holders(
        &self,
        markets: &[&str],
        limit: Option<i32>,
        min_balance: Option<i32>,
    ) -> Result<Vec<MarketTopHolders>> {
        // Validate all market IDs
        for market_id in markets {
            validate_market_id(market_id)?;
        }

        // Validate optional parameters
        validate_limit(limit)?;
        validate_min_balance(min_balance)?;

        let mut url = self.build_url("holders");

        // Add market query parameter (comma-separated)
        if !markets.is_empty() {
            let market_value = markets.join(",");
            url.query_pairs_mut().append_pair("market", &market_value);
        }

        // Add optional limit parameter
        if let Some(l) = limit {
            url.query_pairs_mut().append_pair("limit", &l.to_string());
        }

        // Add optional minBalance parameter
        if let Some(mb) = min_balance {
            url.query_pairs_mut()
                .append_pair("minBalance", &mb.to_string());
        }

        trace!(url = %url, method = "GET", market_count = markets.len(), "sending HTTP request");

        for attempt in 0..=MAX_RETRIES {
            match self.http_client.get(url.clone()).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    trace!(status = %status, attempt = attempt, "received HTTP response");

                    if status.is_success() {
                        let holders_response: Vec<MarketTopHolders> = resp.json().await?;
                        trace!(count = holders_response.len(), "received holders data");
                        return Ok(holders_response);
                    }

                    let status_code = status.as_u16();
                    if attempt < MAX_RETRIES && is_retryable_status(status_code) {
                        trace!(status = status_code, attempt = attempt, "retrying request");
                        sleep(Duration::from_millis(RETRY_DELAY_MS * (attempt as u64 + 1))).await;
                        continue;
                    }

                    return Err(self.check_response(resp).await.unwrap_err());
                }
                Err(err) => {
                    trace!(error = %err, attempt = attempt, "HTTP request error");
                    if attempt < MAX_RETRIES && err.is_timeout() {
                        trace!(attempt = attempt, "retrying after timeout");
                        sleep(Duration::from_millis(RETRY_DELAY_MS * (attempt as u64 + 1))).await;
                        continue;
                    }

                    return Err(err.into());
                }
            }
        }

        unreachable!("retry loop should return success or error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable_status() {
        // Retryable statuses
        assert!(is_retryable_status(408)); // Request Timeout
        assert!(is_retryable_status(429)); // Too Many Requests
        assert!(is_retryable_status(500)); // Internal Server Error
        assert!(is_retryable_status(501)); // Not Implemented (within 500-504 range)
        assert!(is_retryable_status(502)); // Bad Gateway
        assert!(is_retryable_status(503)); // Service Unavailable
        assert!(is_retryable_status(504)); // Gateway Timeout

        // Non-retryable statuses
        assert!(!is_retryable_status(200)); // OK
        assert!(!is_retryable_status(400)); // Bad Request
        assert!(!is_retryable_status(401)); // Unauthorized
        assert!(!is_retryable_status(403)); // Forbidden
        assert!(!is_retryable_status(404)); // Not Found
        assert!(!is_retryable_status(499)); // Client error (outside retryable range)
        assert!(!is_retryable_status(505)); // HTTP Version Not Supported (outside 500-504)
    }
}
