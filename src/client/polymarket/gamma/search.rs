//! Search endpoint types and implementation for the Gamma API.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};
use url::Url;

use crate::error::Result;

use super::Client;
use super::comments::CommentProfile;
use super::events::Event;
use super::helpers::validate_tag_id;
use super::tags::Tag;

/// Flexible search response container.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResults {
    pub events: Option<Vec<Event>>,
    pub tags: Option<Vec<Tag>>,
    pub profiles: Option<Vec<CommentProfile>>,
    pub pagination: Option<Pagination>,
}

/// Pagination information for search results.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    #[serde(alias = "hasMore")]
    pub has_more: bool,
    #[serde(alias = "totalResults")]
    pub total_results: u32,
}

/// Parameters for searching markets, events, and profiles.
#[derive(Debug, Clone)]
pub struct SearchRequest<'a> {
    pub q: &'a str,
    pub cache: Option<bool>,
    pub events_status: Option<&'a str>,
    pub limit_per_type: Option<u32>,
    pub page: Option<u32>,
    pub events_tag: Option<Vec<String>>,
    pub keep_closed_markets: Option<u32>,
    pub sort: Option<&'a str>,
    pub ascending: Option<bool>,
    pub search_tags: Option<bool>,
    pub search_profiles: Option<bool>,
    pub recurrence: Option<&'a str>,
    pub exclude_tag_id: Option<Vec<String>>,
    pub optimized: Option<bool>,
}

impl<'a> SearchRequest<'a> {
    /// Validates request parameters.
    pub fn validate(&self) -> Result<()> {
        if self.q.trim().is_empty() {
            return Err(crate::error::PolymarketError::bad_request(
                "query cannot be empty",
            ));
        }
        if let Some(exclude_tag_ids) = &self.exclude_tag_id {
            for exclude_tag_id in exclude_tag_ids {
                validate_tag_id(Some(exclude_tag_id.as_str()))?;
            }
        }
        Ok(())
    }

    pub(crate) fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("public-search");
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("q", self.q);
            if let Some(cache) = self.cache {
                pairs.append_pair("cache", &cache.to_string());
            }
            if let Some(events_status) = self.events_status {
                pairs.append_pair("events_status", events_status);
            }
            if let Some(limit) = self.limit_per_type {
                pairs.append_pair("limit_per_type", &limit.to_string());
            }
            if let Some(page) = self.page {
                pairs.append_pair("page", &page.to_string());
            }
            if let Some(event_tags) = &self.events_tag {
                for tag in event_tags {
                    pairs.append_pair("events_tag", tag);
                }
            }
            if let Some(keep_closed_markets) = self.keep_closed_markets {
                pairs.append_pair("keep_closed_markets", &keep_closed_markets.to_string());
            }
            if let Some(sort) = self.sort {
                pairs.append_pair("sort", sort);
            }
            if let Some(ascending) = self.ascending {
                pairs.append_pair("ascending", &ascending.to_string());
            }
            if let Some(search_tags) = self.search_tags {
                pairs.append_pair("search_tags", &search_tags.to_string());
            }
            if let Some(search_profiles) = self.search_profiles {
                pairs.append_pair("search_profiles", &search_profiles.to_string());
            }
            if let Some(recurrence) = self.recurrence {
                pairs.append_pair("recurrence", recurrence);
            }
            if let Some(exclude_tag_ids) = &self.exclude_tag_id {
                for exclude_tag_id in exclude_tag_ids {
                    pairs.append_pair("exclude_tag_id", exclude_tag_id);
                }
            }
            if let Some(optimized) = self.optimized {
                pairs.append_pair("optimized", &optimized.to_string());
            }
        }
        url
    }
}

// -----------------------------------------------------------------------------
// Client implementation
// -----------------------------------------------------------------------------

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_query() {
        let req = SearchRequest {
            q: "   ",
            cache: None,
            events_status: None,
            limit_per_type: None,
            page: None,
            events_tag: None,
            keep_closed_markets: None,
            sort: None,
            ascending: None,
            search_tags: None,
            search_profiles: None,
            recurrence: None,
            exclude_tag_id: None,
            optimized: None,
        };

        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("query cannot be empty"));
    }

    #[test]
    fn validate_rejects_non_digit_exclude_tag_id() {
        let req = SearchRequest {
            q: "US",
            exclude_tag_id: Some(vec!["abc".to_string()]),
            cache: None,
            events_status: None,
            limit_per_type: None,
            page: None,
            events_tag: None,
            keep_closed_markets: None,
            sort: None,
            ascending: None,
            search_tags: None,
            search_profiles: None,
            recurrence: None,
            optimized: None,
        };

        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("digits"));
    }
}
