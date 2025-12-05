//! Tag endpoints for Gamma API client.

use tracing::{instrument, trace};

use crate::error::Result;
use crate::gamma::types::{GetTagsRequest, Tag, TagRelationship, TagRelationshipStatus};

use super::Client;

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
