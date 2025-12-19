//! Comment representations and endpoints for the Gamma API.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};
use url::Url;

use crate::error::Result;

use super::Client;
use super::events::OptimizedImage;
use super::helpers::{deserialize_option_u64, validate_comment_parent};

/// Comment reaction.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentReaction {
    pub id: String,
    #[serde(alias = "commentId")]
    pub comment_id: String,
    #[serde(alias = "reactionType")]
    pub reaction_type: String,
    pub icon: Option<String>,
    #[serde(alias = "userAddress")]
    pub user_address: Option<String>,
    #[serde(alias = "createdAt")]
    pub created_at: Option<String>,
    pub profile: Option<CommentProfile>,
}

/// Position information attached to a comment profile.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    #[serde(alias = "tokenId")]
    pub token_id: Option<String>,
    #[serde(alias = "positionSize")]
    pub position_size: Option<f64>,
}

/// Lightweight profile information attached to a comment.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentProfile {
    pub name: Option<String>,
    pub pseudonym: Option<String>,
    #[serde(alias = "displayUsernamePublic")]
    pub display_username_public: Option<bool>,
    pub bio: Option<String>,
    #[serde(alias = "isMod")]
    pub is_mod: Option<bool>,
    #[serde(alias = "isCreator")]
    pub is_creator: Option<bool>,
    #[serde(alias = "proxyWallet")]
    pub proxy_wallet: Option<String>,
    #[serde(alias = "baseAddress")]
    pub base_address: Option<String>,
    #[serde(alias = "profileImage")]
    pub profile_image: Option<String>,
    #[serde(alias = "profileImageOptimized")]
    pub profile_image_optimized: Option<OptimizedImage>,
    pub positions: Option<Vec<Position>>,
}

/// A comment returned by the Gamma API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub body: Option<String>,
    #[serde(alias = "parent_entity_type")]
    pub parent_entity_type: Option<String>,
    #[serde(alias = "parent_entity_id")]
    pub parent_entity_id: Option<String>,
    #[serde(alias = "parent_comment_id")]
    pub parent_comment_id: Option<String>,
    #[serde(alias = "user_address")]
    pub user_address: Option<String>,
    #[serde(alias = "reply_address")]
    pub reply_address: Option<String>,
    #[serde(alias = "created_at")]
    pub created_at: Option<String>,
    #[serde(alias = "updated_at")]
    pub updated_at: Option<String>,
    pub profile: Option<CommentProfile>,
    pub reactions: Option<Vec<CommentReaction>>,
    #[serde(alias = "reportCount", deserialize_with = "deserialize_option_u64")]
    pub report_count: Option<u64>,
    #[serde(alias = "reactionCount", deserialize_with = "deserialize_option_u64")]
    pub reaction_count: Option<u64>,
}

/// Parameters for listing comments.
#[derive(Debug, Clone, Default)]
pub struct GetCommentsRequest<'a> {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order: Option<&'a str>,
    pub ascending: Option<bool>,
    pub parent_entity_type: Option<&'a str>,
    pub parent_entity_id: Option<&'a str>,
    pub get_positions: Option<bool>,
    pub holders_only: Option<bool>,
}

impl<'a> GetCommentsRequest<'a> {
    /// Validates request parameters.
    pub fn validate(&self) -> Result<()> {
        validate_comment_parent(self.parent_entity_type, self.parent_entity_id)?;
        Ok(())
    }

    pub(crate) fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("comments");
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
            pairs.append_pair(
                "parent_entity_type",
                self.parent_entity_type.expect("validated"),
            );
            pairs.append_pair(
                "parent_entity_id",
                self.parent_entity_id.expect("validated"),
            );
            if let Some(get_positions) = self.get_positions {
                pairs.append_pair("get_positions", &get_positions.to_string());
            }
            if let Some(holders_only) = self.holders_only {
                pairs.append_pair("holders_only", &holders_only.to_string());
            }
        }
        url
    }
}

/// Parameters for listing comments by user address.
#[derive(Debug, Clone, Default)]
pub struct GetCommentsByUserAddressRequest<'a> {
    pub user_address: &'a str,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order: Option<&'a str>,
    pub ascending: Option<bool>,
}

impl<'a> GetCommentsByUserAddressRequest<'a> {
    pub fn validate(&self) -> Result<()> {
        if self.user_address.trim().is_empty() {
            return Err(crate::error::PolymarketError::bad_request(
                "user_address cannot be empty",
            ));
        }
        Ok(())
    }

    pub(crate) fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path(&format!("comments/user_address/{}", self.user_address));
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
        }
        url
    }
}

// -----------------------------------------------------------------------------
// Client implementation
// -----------------------------------------------------------------------------

impl Client {
    /// Lists comments with optional pagination and sorting.
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_comments(&self, request: GetCommentsRequest<'_>) -> Result<Vec<Comment>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let comments: Vec<Comment> = response.json().await?;
        trace!(count = comments.len(), "received comments");
        Ok(comments)
    }

    /// Fetches a single comment by ID.
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_comment_by_id(
        &self,
        id: &str,
        get_positions: Option<bool>,
    ) -> Result<Comment> {
        let mut url = self.build_url(&format!("comments/{}", id));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(get_positions) = get_positions {
                pairs.append_pair("get_positions", &get_positions.to_string());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let comment: Comment = response.json().await?;
        trace!(comment_id = %comment.id, "received comment");
        Ok(comment)
    }

    /// Lists comments authored by a specific user address.
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_comments_by_user_address(
        &self,
        request: GetCommentsByUserAddressRequest<'_>,
    ) -> Result<Vec<Comment>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let comments: Vec<Comment> = response.json().await?;
        trace!(count = comments.len(), "received comments");
        Ok(comments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_requires_parent_fields() {
        let req = GetCommentsRequest::default();
        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("required"));
    }

    #[test]
    fn validate_user_address_cannot_be_empty() {
        let req = GetCommentsByUserAddressRequest {
            user_address: "   ",
            ..Default::default()
        };
        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn build_url_includes_parent_filters() {
        let base = Url::parse("https://example.com").unwrap();
        let req = GetCommentsRequest {
            parent_entity_type: Some("Event"),
            parent_entity_id: Some("123"),
            limit: Some(5),
            offset: Some(1),
            order: Some("created_at"),
            ascending: Some(true),
            get_positions: Some(true),
            holders_only: Some(false),
        };
        let url = req.build_url(&base);
        let query = url.query().unwrap_or_default();
        for expected in [
            "limit=5",
            "offset=1",
            "order=created_at",
            "ascending=true",
            "parent_entity_type=Event",
            "parent_entity_id=123",
            "get_positions=true",
            "holders_only=false",
        ] {
            assert!(
                query.contains(expected),
                "missing '{expected}' in query: {query}"
            );
        }
    }
}
