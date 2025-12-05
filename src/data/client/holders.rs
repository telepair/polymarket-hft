//! Holders API methods.

use tracing::{instrument, trace};

use crate::data::types::{
    MarketTopHolders, validate_limit, validate_market_id, validate_min_balance,
};
use crate::error::Result;

use tokio::time::{Duration, sleep};

use super::Client;

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
    /// Returns a vector of `HoldersResponse` containing token and holder information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::Client;
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
