//! Series endpoints for Gamma API client.

use tracing::{instrument, trace};

use crate::error::Result;
use crate::gamma::types::{GetSeriesRequest, Series};

use super::Client;

impl Client {
    /// Lists series with optional filters.
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_series(&self, request: GetSeriesRequest<'_>) -> Result<Vec<Series>> {
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let series: Vec<Series> = response.json().await?;
        trace!(count = series.len(), "received series");
        Ok(series)
    }

    /// Gets a series by its ID.
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_series_by_id(&self, id: &str, include_chat: Option<bool>) -> Result<Series> {
        let mut url = self.build_url(&format!("series/{}", id));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(include_chat) = include_chat {
                pairs.append_pair("include_chat", &include_chat.to_string());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let series: Series = response.json().await?;
        trace!(series_id = %series.id, "received series");
        Ok(series)
    }
}
