//! Tag metadata and endpoints for the Gamma API.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};
use url::Url;

use crate::error::Result;

use super::Client;

/// Tag representation from the Gamma API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub label: Option<String>,
    pub slug: Option<String>,
    #[serde(alias = "forceShow")]
    pub force_show: Option<bool>,
    #[serde(alias = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(alias = "createdBy")]
    pub created_by: Option<i64>,
    #[serde(alias = "updatedBy")]
    pub updated_by: Option<i64>,
    #[serde(alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(alias = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(alias = "forceHide")]
    pub force_hide: Option<bool>,
    #[serde(alias = "isCarousel")]
    pub is_carousel: Option<bool>,
}

/// Relationship between tags with a ranked related tag ID.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagRelationship {
    pub id: String,
    #[serde(alias = "tagID")]
    pub tag_id: Option<i64>,
    #[serde(alias = "relatedTagID")]
    pub related_tag_id: Option<i64>,
    pub rank: Option<i64>,
}

/// Allowed status filters for tag relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagRelationshipStatus {
    Active,
    Closed,
    All,
}

impl TagRelationshipStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TagRelationshipStatus::Active => "active",
            TagRelationshipStatus::Closed => "closed",
            TagRelationshipStatus::All => "all",
        }
    }
}

impl std::fmt::Display for TagRelationshipStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for TagRelationshipStatus {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "active" => Ok(TagRelationshipStatus::Active),
            "closed" => Ok(TagRelationshipStatus::Closed),
            "all" => Ok(TagRelationshipStatus::All),
            _ => Err("status must be one of: active, closed, all".to_string()),
        }
    }
}

/// Request parameters for listing tags.
#[derive(Debug, Clone, Default)]
pub struct GetTagsRequest<'a> {
    /// Maximum number of tags to return.
    pub limit: Option<u32>,
    /// Number of tags to skip before returning results.
    pub offset: Option<u32>,
    /// Comma-separated fields to order by.
    pub order: Option<&'a str>,
    /// Sort results in ascending order when true.
    pub ascending: Option<bool>,
    /// Whether to include template tags.
    pub include_template: Option<bool>,
    /// Filter for carousel tags.
    pub is_carousel: Option<bool>,
}

impl<'a> GetTagsRequest<'a> {
    /// Builds the request URL using the provided base URL.
    pub(crate) fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("tags");
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
            if let Some(include_template) = self.include_template {
                pairs.append_pair("include_template", &include_template.to_string());
            }
            if let Some(is_carousel) = self.is_carousel {
                pairs.append_pair("is_carousel", &is_carousel.to_string());
            }
        }
        url
    }
}

// -----------------------------------------------------------------------------
// Client implementation
// -----------------------------------------------------------------------------

impl Client {
    /// Lists tags with optional pagination.
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_tags(&self, request: GetTagsRequest<'_>) -> Result<Vec<Tag>> {
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let tags: Vec<Tag> = response.json().await?;
        trace!(count = tags.len(), "received tags");
        Ok(tags)
    }

    /// Gets a tag by its numeric ID.
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_tag_by_id(&self, id: &str) -> Result<Tag> {
        let url = self.build_url(&format!("tags/{}", id));
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let tag: Tag = response.json().await?;
        trace!(tag_id = %tag.id, "received tag");
        Ok(tag)
    }

    /// Gets a tag by slug.
    #[instrument(skip(self), fields(slug = %slug), level = "trace")]
    pub async fn get_tag_by_slug(&self, slug: &str, include_template: Option<bool>) -> Result<Tag> {
        let mut url = self.build_url(&format!("tags/slug/{}", slug));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(include_template) = include_template {
                pairs.append_pair("include_template", &include_template.to_string());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let tag: Tag = response.json().await?;
        trace!(tag_id = %tag.id, "received tag");
        Ok(tag)
    }

    /// Lists related tags (relationships) by tag id.
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_tag_relationships_by_tag(
        &self,
        id: &str,
        omit_empty: Option<bool>,
        status: Option<TagRelationshipStatus>,
    ) -> Result<Vec<TagRelationship>> {
        let mut url = self.build_url(&format!("tags/{}/related-tags", id));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(omit_empty) = omit_empty {
                pairs.append_pair("omit_empty", &omit_empty.to_string());
            }
            if let Some(status) = status {
                pairs.append_pair("status", status.as_str());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let relationships: Vec<TagRelationship> = response.json().await?;
        trace!(count = relationships.len(), "received tag relationships");
        Ok(relationships)
    }

    /// Lists related tags (relationships) by tag slug.
    #[instrument(skip(self), fields(slug = %slug), level = "trace")]
    pub async fn get_tag_relationships_by_slug(
        &self,
        slug: &str,
        omit_empty: Option<bool>,
        status: Option<TagRelationshipStatus>,
    ) -> Result<Vec<TagRelationship>> {
        let mut url = self.build_url(&format!("tags/slug/{}/related-tags", slug));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(omit_empty) = omit_empty {
                pairs.append_pair("omit_empty", &omit_empty.to_string());
            }
            if let Some(status) = status {
                pairs.append_pair("status", status.as_str());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let relationships: Vec<TagRelationship> = response.json().await?;
        trace!(count = relationships.len(), "received tag relationships");
        Ok(relationships)
    }

    /// Gets tags related to a tag id (inverse relationships).
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_tags_related_to_tag(
        &self,
        id: &str,
        omit_empty: Option<bool>,
        status: Option<TagRelationshipStatus>,
    ) -> Result<Vec<Tag>> {
        let mut url = self.build_url(&format!("tags/{}/related-tags/tags", id));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(omit_empty) = omit_empty {
                pairs.append_pair("omit_empty", &omit_empty.to_string());
            }
            if let Some(status) = status {
                pairs.append_pair("status", status.as_str());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let tags: Vec<Tag> = response.json().await?;
        trace!(count = tags.len(), "received related tags");
        Ok(tags)
    }

    /// Gets tags related to a tag slug (inverse relationships).
    #[instrument(skip(self), fields(slug = %slug), level = "trace")]
    pub async fn get_tags_related_to_slug(
        &self,
        slug: &str,
        omit_empty: Option<bool>,
        status: Option<TagRelationshipStatus>,
    ) -> Result<Vec<Tag>> {
        let mut url = self.build_url(&format!("tags/slug/{}/related-tags/tags", slug));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(omit_empty) = omit_empty {
                pairs.append_pair("omit_empty", &omit_empty.to_string());
            }
            if let Some(status) = status {
                pairs.append_pair("status", status.as_str());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let tags: Vec<Tag> = response.json().await?;
        trace!(count = tags.len(), "received related tags");
        Ok(tags)
    }
}
