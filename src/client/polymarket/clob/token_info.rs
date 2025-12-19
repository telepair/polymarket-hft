//! Token information endpoints for CLOB API.

use serde::Deserialize;
use tracing::{instrument, trace};

use super::Client;
use super::types::TickSize;
use crate::error::Result;

/// Token info response from the server (tick-size endpoint).
#[derive(Debug, Clone, Deserialize)]
struct TickSizeResponse {
    minimum_tick_size: f64,
}

/// Token info response from the server (neg-risk endpoint).
#[derive(Debug, Clone, Deserialize)]
struct NegRiskResponse {
    neg_risk: bool,
}

// =============================================================================
// Client Implementation
// =============================================================================

impl Client {
    /// Gets the minimum tick size for a token.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The unique identifier for the token.
    ///
    /// # Returns
    ///
    /// Returns the `TickSize` enum value for the token.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let tick_size = client.get_tick_size("token_id").await?;
    ///     println!("Tick size: {}", tick_size);
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), level = "trace")]
    pub async fn get_tick_size(&self, token_id: &str) -> Result<TickSize> {
        let mut url = self.build_url("tick-size");
        url.query_pairs_mut().append_pair("token_id", token_id);

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let info: TickSizeResponse = response.json().await?;

        // Parse tick size float to enum
        let tick_size = if (info.minimum_tick_size - 0.1).abs() < f64::EPSILON {
            TickSize::PointOne
        } else if (info.minimum_tick_size - 0.01).abs() < f64::EPSILON {
            TickSize::PointZeroOne
        } else if (info.minimum_tick_size - 0.001).abs() < f64::EPSILON {
            TickSize::PointZeroZeroOne
        } else if (info.minimum_tick_size - 0.0001).abs() < f64::EPSILON {
            TickSize::PointZeroZeroZeroOne
        } else {
            TickSize::PointZeroOne // Default
        };

        trace!(tick_size = %tick_size, "received tick size");
        Ok(tick_size)
    }

    /// Checks if a token uses negative risk.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The unique identifier for the token.
    ///
    /// # Returns
    ///
    /// Returns `true` if the token uses neg_risk exchange contract.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let neg_risk = client.get_neg_risk("token_id").await?;
    ///     println!("Neg risk: {}", neg_risk);
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), level = "trace")]
    pub async fn get_neg_risk(&self, token_id: &str) -> Result<bool> {
        let mut url = self.build_url("neg-risk");
        url.query_pairs_mut().append_pair("token_id", token_id);

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let info: NegRiskResponse = response.json().await?;

        trace!(neg_risk = %info.neg_risk, "received neg_risk");
        Ok(info.neg_risk)
    }

    /// Gets the fee rate in basis points for a token.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The unique identifier for the token.
    ///
    /// # Returns
    ///
    /// Returns the fee rate in basis points (e.g., 100 = 1%).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let fee_bps = client.get_fee_rate_bps("token_id").await?;
    ///     println!("Fee rate: {} bps", fee_bps);
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), level = "trace")]
    pub async fn get_fee_rate_bps(&self, token_id: &str) -> Result<u32> {
        let mut url = self.build_url("fee-rate-bps");
        url.query_pairs_mut().append_pair("token_id", token_id);

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;

        #[derive(Deserialize)]
        struct FeeRateResponse {
            fee_rate_bps: u32,
        }

        let info: FeeRateResponse = response.json().await?;
        trace!(fee_rate_bps = info.fee_rate_bps, "received fee rate");
        Ok(info.fee_rate_bps)
    }
}
