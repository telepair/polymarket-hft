//! Spreads types and endpoints for CLOB API.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

use crate::error::Result;

use super::Client;
use super::pricing::Side;

/// Request item for getting bid-ask spreads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadRequest {
    /// The unique identifier for the token.
    pub token_id: String,
    /// Optional side parameter for certain operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

// -----------------------------------------------------------------------------
// Client implementation
// -----------------------------------------------------------------------------

impl Client {
    /// Gets bid-ask spreads for multiple tokens.
    ///
    /// # Arguments
    ///
    /// * `request` - A slice of request items specifying token IDs and optional sides.
    ///
    /// # Returns
    ///
    /// Returns a map of token_id to spread value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::{Client, SpreadRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let request = vec![
    ///         SpreadRequest { token_id: "123".to_string(), side: None },
    ///         SpreadRequest { token_id: "456".to_string(), side: None },
    ///     ];
    ///     let spreads = client.get_spreads(&request).await?;
    ///     for (token_id, spread) in spreads {
    ///         println!("Token {}: spread {}", token_id, spread);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_spreads(&self, request: &[SpreadRequest]) -> Result<HashMap<String, String>> {
        let url = self.build_url("spreads");

        trace!(url = %url, method = "POST", count = request.len(), "sending HTTP request");
        let response = self.http_client.post(url).json(request).send().await?;
        let response = self.check_response(response).await?;
        let spreads: HashMap<String, String> = response.json().await?;
        trace!(count = spreads.len(), "received spreads");
        Ok(spreads)
    }
}
