//! Market endpoints for Gamma API client.

use tracing::{instrument, trace};

use crate::error::Result;
use crate::gamma::types::{GetMarketsRequest, Market, Tag};

use super::Client;

impl Client {
    /// Lists markets with optional filters.
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_markets(&self, request: GetMarketsRequest<'_>) -> Result<Vec<Market>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let markets: Vec<Market> = response.json().await?;
        trace!(count = markets.len(), "received markets");
        Ok(markets)
    }

    /// Gets a market by its ID.
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_market_by_id(&self, id: &str, include_tag: Option<bool>) -> Result<Market> {
        let mut url = self.build_url(&format!("markets/{}", id));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(include_tag) = include_tag {
                pairs.append_pair("include_tag", &include_tag.to_string());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let market: Market = response.json().await?;
        trace!(market_id = %market.id, "received market");
        Ok(market)
    }

    /// Lists tags attached to a market by ID.
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_market_tags(&self, id: &str) -> Result<Vec<Tag>> {
        let url = self.build_url(&format!("markets/{}/tags", id));
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let tags: Vec<Tag> = response.json().await?;
        trace!(count = tags.len(), "received tags");
        Ok(tags)
    }

    /// Gets a market by its slug.
    #[instrument(skip(self), fields(slug = %slug), level = "trace")]
    pub async fn get_market_by_slug(
        &self,
        slug: &str,
        include_tag: Option<bool>,
    ) -> Result<Market> {
        let mut url = self.build_url(&format!("markets/slug/{}", slug));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(include_tag) = include_tag {
                pairs.append_pair("include_tag", &include_tag.to_string());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let market: Market = response.json().await?;
        trace!(market_id = %market.id, "received market");
        Ok(market)
    }
}
