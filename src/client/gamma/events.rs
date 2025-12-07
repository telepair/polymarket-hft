//! Event models and endpoints for the Gamma API.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};
use url::Url;

use crate::error::Result;

use super::Client;
use super::helpers::{deserialize_option_f64, deserialize_option_u64, validate_tag_id};
use super::tags::Tag;

/// Optimized image metadata.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizedImage {
    pub id: String,
    #[serde(alias = "imageUrlSource")]
    pub image_url_source: Option<String>,
    #[serde(alias = "imageUrlOptimized")]
    pub image_url_optimized: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "imageSizeKbSource"
    )]
    pub image_size_kb_source: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "imageSizeKbOptimized"
    )]
    pub image_size_kb_optimized: Option<f64>,
    #[serde(alias = "imageOptimizedComplete")]
    pub image_optimized_complete: Option<bool>,
    #[serde(alias = "imageOptimizedLastUpdated")]
    pub image_optimized_last_updated: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_u64", alias = "relID")]
    pub rel_id: Option<u64>,
    pub field: Option<String>,
    pub relname: Option<String>,
}

/// Category representation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub label: Option<String>,
    #[serde(alias = "parentCategory")]
    pub parent_category: Option<String>,
    pub slug: Option<String>,
    #[serde(alias = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(alias = "createdBy")]
    pub created_by: Option<String>,
    #[serde(alias = "updatedBy")]
    pub updated_by: Option<String>,
    #[serde(alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(alias = "updatedAt")]
    pub updated_at: Option<String>,
}

/// Collection representation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub ticker: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    #[serde(alias = "collectionType")]
    pub collection_type: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    #[serde(alias = "headerImage")]
    pub header_image: Option<String>,
    pub layout: Option<String>,
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
    #[serde(alias = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(alias = "createdBy")]
    pub created_by: Option<String>,
    #[serde(alias = "updatedBy")]
    pub updated_by: Option<String>,
    #[serde(alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(alias = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(alias = "commentsEnabled")]
    pub comments_enabled: Option<bool>,
    #[serde(alias = "imageOptimized")]
    pub image_optimized: Option<OptimizedImage>,
    #[serde(alias = "iconOptimized")]
    pub icon_optimized: Option<OptimizedImage>,
    #[serde(alias = "headerImageOptimized")]
    pub header_image_optimized: Option<OptimizedImage>,
}

/// Event creator information.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventCreator {
    pub id: String,
    #[serde(alias = "creatorName")]
    pub creator_name: Option<String>,
    #[serde(alias = "creatorHandle")]
    pub creator_handle: Option<String>,
    #[serde(alias = "creatorUrl")]
    pub creator_url: Option<String>,
    #[serde(alias = "creatorImage")]
    pub creator_image: Option<String>,
    #[serde(alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(alias = "updatedAt")]
    pub updated_at: Option<String>,
}

/// Event chat information.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventChat {
    pub id: String,
    #[serde(alias = "channelId")]
    pub channel_id: Option<String>,
    #[serde(alias = "channelName")]
    pub channel_name: Option<String>,
    #[serde(alias = "channelImage")]
    pub channel_image: Option<String>,
    pub live: Option<bool>,
    #[serde(alias = "startTime")]
    pub start_time: Option<String>,
    #[serde(alias = "endTime")]
    pub end_time: Option<String>,
}

/// Event template information.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventTemplate {
    pub id: String,
    #[serde(alias = "eventTitle")]
    pub event_title: Option<String>,
    #[serde(alias = "eventSlug")]
    pub event_slug: Option<String>,
    #[serde(alias = "eventImage")]
    pub event_image: Option<String>,
    #[serde(alias = "marketTitle")]
    pub market_title: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "resolutionSource")]
    pub resolution_source: Option<String>,
    #[serde(alias = "negRisk")]
    pub neg_risk: Option<bool>,
    #[serde(alias = "sortBy")]
    pub sort_by: Option<String>,
    #[serde(alias = "showMarketImages")]
    pub show_market_images: Option<bool>,
    #[serde(alias = "seriesSlug")]
    pub series_slug: Option<String>,
    pub outcomes: Option<String>,
}

/// Compact event representation used inside nested responses.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSummary {
    pub id: String,
    pub ticker: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
    #[serde(alias = "startDate")]
    pub start_date: Option<String>,
    #[serde(alias = "endDate")]
    pub end_date: Option<String>,
    pub category: Option<String>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
}

// Forward declarations for circular dependencies
use super::markets::Market;
use super::series::SeriesSummary;

/// Event representation from the Gamma API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub ticker: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "resolutionSource")]
    pub resolution_source: Option<String>,
    #[serde(alias = "startDate")]
    pub start_date: Option<String>,
    #[serde(alias = "creationDate")]
    pub creation_date: Option<String>,
    #[serde(alias = "endDate")]
    pub end_date: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub new: Option<bool>,
    pub featured: Option<bool>,
    pub restricted: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub liquidity: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub volume: Option<f64>,
    #[serde(
        default,
        alias = "openInterest",
        deserialize_with = "deserialize_option_f64"
    )]
    pub open_interest: Option<f64>,
    #[serde(alias = "sortBy")]
    pub sort_by: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    #[serde(alias = "isTemplate")]
    pub is_template: Option<bool>,
    #[serde(alias = "templateVariables")]
    pub template_variables: Option<String>,
    #[serde(alias = "published_at")]
    pub published_at: Option<String>,
    #[serde(alias = "createdBy")]
    pub created_by: Option<String>,
    #[serde(alias = "updatedBy")]
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
    pub volume1wk: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub volume1mo: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub volume1yr: Option<f64>,
    #[serde(alias = "featuredImage")]
    pub featured_image: Option<String>,
    #[serde(alias = "disqusThread")]
    pub disqus_thread: Option<String>,
    #[serde(alias = "parentEvent")]
    pub parent_event: Option<String>,
    #[serde(alias = "enableOrderBook")]
    pub enable_order_book: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub liquidity_amm: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub liquidity_clob: Option<f64>,
    #[serde(alias = "negRisk")]
    pub neg_risk: Option<bool>,
    #[serde(alias = "negRiskMarketID")]
    pub neg_risk_market_id: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "negRiskFeeBips"
    )]
    pub neg_risk_fee_bips: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_option_u64")]
    pub comment_count: Option<u64>,
    #[serde(alias = "imageOptimized")]
    pub image_optimized: Option<OptimizedImage>,
    #[serde(alias = "iconOptimized")]
    pub icon_optimized: Option<OptimizedImage>,
    #[serde(alias = "featuredImageOptimized")]
    pub featured_image_optimized: Option<OptimizedImage>,
    #[serde(alias = "subEvents")]
    pub sub_events: Option<Vec<String>>,
    pub markets: Option<Vec<Market>>,
    pub series: Option<Vec<SeriesSummary>>,
    pub categories: Option<Vec<Category>>,
    pub collections: Option<Vec<Collection>>,
    pub tags: Option<Vec<Tag>>,
    pub cyom: Option<bool>,
    #[serde(alias = "closedTime")]
    pub closed_time: Option<String>,
    #[serde(alias = "showAllOutcomes")]
    pub show_all_outcomes: Option<bool>,
    #[serde(alias = "showMarketImages")]
    pub show_market_images: Option<bool>,
    #[serde(alias = "automaticallyResolved")]
    pub automatically_resolved: Option<bool>,
    #[serde(alias = "enableNegRisk")]
    pub enable_neg_risk: Option<bool>,
    #[serde(alias = "automaticallyActive")]
    pub automatically_active: Option<bool>,
    #[serde(alias = "negRiskAugmented")]
    pub neg_risk_augmented: Option<bool>,
    #[serde(alias = "eventDate")]
    pub event_date: Option<String>,
    #[serde(alias = "startTime")]
    pub start_time: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "eventWeek"
    )]
    pub event_week: Option<u64>,
    pub series_slug: Option<String>,
    pub score: Option<String>,
    pub elapsed: Option<String>,
    pub period: Option<String>,
    pub live: Option<bool>,
    pub ended: Option<bool>,
    #[serde(alias = "finishedTimestamp")]
    pub finished_timestamp: Option<String>,
    #[serde(alias = "gmpChartMode")]
    pub gmp_chart_mode: Option<String>,
    #[serde(alias = "eventCreators")]
    pub event_creators: Option<Vec<EventCreator>>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "tweetCount"
    )]
    pub tweet_count: Option<u64>,
    pub chats: Option<Vec<EventChat>>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "featuredOrder"
    )]
    pub featured_order: Option<u64>,
    #[serde(alias = "estimateValue")]
    pub estimate_value: Option<bool>,
    #[serde(alias = "cantEstimate")]
    pub cant_estimate: Option<bool>,
    #[serde(alias = "estimatedValue")]
    pub estimated_value: Option<String>,
    pub templates: Option<Vec<EventTemplate>>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "spreadsMainLine"
    )]
    pub spreads_main_line: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "totalsMainLine"
    )]
    pub totals_main_line: Option<f64>,
    #[serde(alias = "carouselMap")]
    pub carousel_map: Option<String>,
    #[serde(alias = "pendingDeployment")]
    pub pending_deployment: Option<bool>,
    pub deploying: Option<bool>,
    #[serde(alias = "deployingTimestamp")]
    pub deploying_timestamp: Option<String>,
    #[serde(alias = "scheduledDeploymentTimestamp")]
    pub scheduled_deployment_timestamp: Option<String>,
    #[serde(alias = "gameStatus")]
    pub game_status: Option<String>,
}

/// Request parameters for listing events.
#[derive(Debug, Clone, Default)]
pub struct GetEventsRequest<'a> {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order: Option<&'a str>,
    pub ascending: Option<bool>,
    pub id: Option<Vec<String>>,
    pub tag_id: Option<&'a str>,
    pub exclude_tag_id: Option<Vec<String>>,
    pub slug: Option<Vec<String>>,
    pub tag_slug: Option<&'a str>,
    pub related_tags: Option<bool>,
    pub active: Option<bool>,
    pub archived: Option<bool>,
    pub featured: Option<bool>,
    pub cyom: Option<bool>,
    pub include_chat: Option<bool>,
    pub include_template: Option<bool>,
    pub recurrence: Option<&'a str>,
    pub closed: Option<bool>,
    pub liquidity_min: Option<f64>,
    pub liquidity_max: Option<f64>,
    pub volume_min: Option<f64>,
    pub volume_max: Option<f64>,
    pub start_date_min: Option<&'a str>,
    pub start_date_max: Option<&'a str>,
    pub end_date_min: Option<&'a str>,
    pub end_date_max: Option<&'a str>,
}

impl<'a> GetEventsRequest<'a> {
    /// Validates request parameters before sending.
    pub fn validate(&self) -> Result<()> {
        if let Some(ids) = &self.id {
            for id in ids {
                validate_tag_id(Some(id.as_str()))?;
            }
        }
        validate_tag_id(self.tag_id)?;
        if let Some(exclude_ids) = &self.exclude_tag_id {
            for exclude_id in exclude_ids {
                validate_tag_id(Some(exclude_id.as_str()))?;
            }
        }
        Ok(())
    }

    /// Builds the request URL using the provided base URL.
    pub(crate) fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("events");
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
            if let Some(ids) = &self.id {
                for id in ids {
                    pairs.append_pair("id", id);
                }
            }
            if let Some(tag_id) = self.tag_id {
                pairs.append_pair("tag_id", tag_id);
            }
            if let Some(exclude_tag_ids) = &self.exclude_tag_id {
                for exclude_tag_id in exclude_tag_ids {
                    pairs.append_pair("exclude_tag_id", exclude_tag_id);
                }
            }
            if let Some(slugs) = &self.slug {
                for slug in slugs {
                    pairs.append_pair("slug", slug);
                }
            }
            if let Some(tag_slug) = self.tag_slug {
                pairs.append_pair("tag_slug", tag_slug);
            }
            if let Some(related_tags) = self.related_tags {
                pairs.append_pair("related_tags", &related_tags.to_string());
            }
            if let Some(active) = self.active {
                pairs.append_pair("active", &active.to_string());
            }
            if let Some(archived) = self.archived {
                pairs.append_pair("archived", &archived.to_string());
            }
            if let Some(featured) = self.featured {
                pairs.append_pair("featured", &featured.to_string());
            }
            if let Some(cyom) = self.cyom {
                pairs.append_pair("cyom", &cyom.to_string());
            }
            if let Some(include_chat) = self.include_chat {
                pairs.append_pair("include_chat", &include_chat.to_string());
            }
            if let Some(include_template) = self.include_template {
                pairs.append_pair("include_template", &include_template.to_string());
            }
            if let Some(recurrence) = self.recurrence {
                pairs.append_pair("recurrence", recurrence);
            }
            if let Some(closed) = self.closed {
                pairs.append_pair("closed", &closed.to_string());
            }
            if let Some(liquidity_min) = self.liquidity_min {
                pairs.append_pair("liquidity_min", &liquidity_min.to_string());
            }
            if let Some(liquidity_max) = self.liquidity_max {
                pairs.append_pair("liquidity_max", &liquidity_max.to_string());
            }
            if let Some(volume_min) = self.volume_min {
                pairs.append_pair("volume_min", &volume_min.to_string());
            }
            if let Some(volume_max) = self.volume_max {
                pairs.append_pair("volume_max", &volume_max.to_string());
            }
            if let Some(start_date_min) = self.start_date_min {
                pairs.append_pair("start_date_min", start_date_min);
            }
            if let Some(start_date_max) = self.start_date_max {
                pairs.append_pair("start_date_max", start_date_max);
            }
            if let Some(end_date_min) = self.end_date_min {
                pairs.append_pair("end_date_min", end_date_min);
            }
            if let Some(end_date_max) = self.end_date_max {
                pairs.append_pair("end_date_max", end_date_max);
            }
        }
        url
    }
}

// -----------------------------------------------------------------------------
// Client implementation
// -----------------------------------------------------------------------------

impl Client {
    /// Lists events with optional filters.
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_events(&self, request: GetEventsRequest<'_>) -> Result<Vec<Event>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let events: Vec<Event> = response.json().await?;
        trace!(count = events.len(), "received events");
        Ok(events)
    }

    /// Gets an event by its ID.
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_event_by_id(
        &self,
        id: &str,
        include_chat: Option<bool>,
        include_template: Option<bool>,
    ) -> Result<Event> {
        let mut url = self.build_url(&format!("events/{}", id));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(include_chat) = include_chat {
                pairs.append_pair("include_chat", &include_chat.to_string());
            }
            if let Some(include_template) = include_template {
                pairs.append_pair("include_template", &include_template.to_string());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let event: Event = response.json().await?;
        trace!(event_id = %event.id, "received event");
        Ok(event)
    }

    /// Lists tags associated with an event.
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_event_tags(&self, id: &str) -> Result<Vec<Tag>> {
        let url = self.build_url(&format!("events/{}/tags", id));
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let tags: Vec<Tag> = response.json().await?;
        trace!(count = tags.len(), "received tags");
        Ok(tags)
    }

    /// Gets an event by its slug.
    #[instrument(skip(self), fields(slug = %slug), level = "trace")]
    pub async fn get_event_by_slug(
        &self,
        slug: &str,
        include_chat: Option<bool>,
        include_template: Option<bool>,
    ) -> Result<Event> {
        let mut url = self.build_url(&format!("events/slug/{}", slug));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(include_chat) = include_chat {
                pairs.append_pair("include_chat", &include_chat.to_string());
            }
            if let Some(include_template) = include_template {
                pairs.append_pair("include_template", &include_template.to_string());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let event: Event = response.json().await?;
        trace!(event_id = %event.id, "received event");
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_non_digit_id() {
        let req = GetEventsRequest {
            id: Some(vec!["abc".to_string()]),
            ..Default::default()
        };

        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("digits"));
    }

    #[test]
    fn validate_rejects_non_digit_exclude_tag_id() {
        let req = GetEventsRequest {
            exclude_tag_id: Some(vec!["abc".to_string()]),
            ..Default::default()
        };

        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("digits"));
    }

    #[test]
    fn build_url_includes_filters() {
        let base = Url::parse("https://example.com").unwrap();
        let url = GetEventsRequest {
            limit: Some(5),
            offset: Some(2),
            id: Some(vec!["1".to_string()]),
            tag_id: Some("2"),
            exclude_tag_id: Some(vec!["3".to_string()]),
            slug: Some(vec!["slug-a".to_string()]),
            related_tags: Some(true),
            active: Some(true),
            archived: Some(false),
            featured: Some(false),
            cyom: Some(true),
            include_chat: Some(true),
            include_template: Some(false),
            recurrence: Some("weekly"),
            closed: Some(false),
            liquidity_min: Some(1.0),
            liquidity_max: Some(2.0),
            volume_min: Some(3.0),
            volume_max: Some(4.0),
            start_date_min: Some("2023-01-01"),
            start_date_max: Some("2023-12-31"),
            end_date_min: Some("2023-02-01"),
            end_date_max: Some("2023-11-30"),
            ..Default::default()
        }
        .build_url(&base);

        let query = url.query().unwrap_or_default();
        for expected in [
            "limit=5",
            "offset=2",
            "id=1",
            "tag_id=2",
            "exclude_tag_id=3",
            "slug=slug-a",
            "related_tags=true",
            "active=true",
            "archived=false",
            "featured=false",
            "cyom=true",
            "include_chat=true",
            "include_template=false",
            "recurrence=weekly",
            "closed=false",
            "liquidity_min=1",
            "liquidity_max=2",
            "volume_min=3",
            "volume_max=4",
            "start_date_min=2023-01-01",
            "start_date_max=2023-12-31",
            "end_date_min=2023-02-01",
            "end_date_max=2023-11-30",
        ] {
            assert!(
                query.contains(expected),
                "missing '{expected}' in query: {query}"
            );
        }
    }
}
