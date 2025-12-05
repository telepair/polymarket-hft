//! Search endpoints for Gamma API client.

use tracing::{instrument, trace};

use crate::error::Result;
use crate::gamma::types::{SearchRequest, SearchResults};

use super::Client;

impl Client {
    /// Searches markets, events, and profiles.
    #[instrument(skip(self, request), level = "trace")]
    pub async fn search(&self, request: SearchRequest<'_>) -> Result<SearchResults> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let results: SearchResults = response.json().await?;
        trace!("received search results");
        Ok(results)
    }
}
