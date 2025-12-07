//! Series metadata and endpoints for the Gamma API.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};
use url::Url;

use crate::error::{PolymarketError, Result};

use super::Client;
use super::events::{Category, Collection, EventChat};
use super::helpers::{deserialize_option_f64, deserialize_option_i64, deserialize_option_u64};
use super::tags::Tag;

// Forward declaration
use super::events::Event;

/// A Gamma series, often grouping recurring events or markets.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub id: String,
    pub ticker: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    #[serde(alias = "series_type")]
    pub series_type: Option<String>,
    pub recurrence: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub new: Option<bool>,
    pub featured: Option<bool>,
    pub restricted: Option<bool>,
    #[serde(alias = "isTemplate")]
    pub is_template: Option<bool>,
    #[serde(alias = "templateVariables")]
    pub template_variables: Option<String>,
    #[serde(alias = "published_at")]
    pub published_at: Option<String>,
    #[serde(alias = "created_by")]
    pub created_by: Option<String>,
    #[serde(alias = "updated_by")]
    pub updated_by: Option<String>,
    #[serde(alias = "created_at")]
    pub created_at: Option<String>,
    #[serde(alias = "updated_at")]
    pub updated_at: Option<String>,
    #[serde(alias = "commentsEnabled")]
    pub comments_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub competitive: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub volume24hr: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub volume: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub liquidity: Option<f64>,
    #[serde(alias = "startDate")]
    pub start_date: Option<String>,
    #[serde(alias = "pythTokenId")]
    pub pyth_token_id: Option<String>,
    #[serde(alias = "cgAssetName")]
    pub cg_asset_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_i64")]
    pub score: Option<i64>,
    pub events: Option<Vec<Event>>,
    pub collections: Option<Vec<Collection>>,
    pub categories: Option<Vec<Category>>,
    pub tags: Option<Vec<Tag>>,
    #[serde(default, deserialize_with = "deserialize_option_u64")]
    pub comment_count: Option<u64>,
    pub chats: Option<Vec<EventChat>>,
}

/// Lightweight series representation for nested responses.
pub type SeriesSummary = Series;

/// Request parameters for listing series.
#[derive(Debug, Clone, Default)]
pub struct GetSeriesRequest<'a> {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order: Option<&'a str>,
    pub ascending: Option<bool>,
    pub slug: Option<&'a str>,
    pub categories_ids: Option<Vec<String>>,
    pub categories_labels: Option<Vec<String>>,
    pub closed: Option<bool>,
    pub include_chat: Option<bool>,
    pub recurrence: Option<&'a str>,
}

impl<'a> GetSeriesRequest<'a> {
    /// Validates request parameters before sending.
    pub fn validate(&self) -> Result<()> {
        if let Some(slug) = self.slug
            && slug.trim().is_empty()
        {
            return Err(PolymarketError::bad_request("slug cannot be empty"));
        }
        Ok(())
    }

    /// Builds the request URL using the provided base URL.
    pub(crate) fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("series");
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(limit) = self.limit {
                pairs.append_pair("limit", &limit.to_string());
            }
            if let Some(offset) = self.offset {
                pairs.append_pair("offset", &offset.to_string());
            }
            if let Some(order) = self.order {
                pairs.append_pair("order", order);
            }
            if let Some(ascending) = self.ascending {
                pairs.append_pair("ascending", &ascending.to_string());
            }
            if let Some(slug) = self.slug {
                pairs.append_pair("slug", slug);
            }
            if let Some(categories_ids) = &self.categories_ids {
                for id in categories_ids {
                    pairs.append_pair("categories_ids", id);
                }
            }
            if let Some(categories_labels) = &self.categories_labels {
                for label in categories_labels {
                    pairs.append_pair("categories_labels", label);
                }
            }
            if let Some(closed) = self.closed {
                pairs.append_pair("closed", &closed.to_string());
            }
            if let Some(include_chat) = self.include_chat {
                pairs.append_pair("include_chat", &include_chat.to_string());
            }
            if let Some(recurrence) = self.recurrence {
                pairs.append_pair("recurrence", recurrence);
            }
        }
        url
    }
}

// -----------------------------------------------------------------------------
// Client implementation
// -----------------------------------------------------------------------------

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
