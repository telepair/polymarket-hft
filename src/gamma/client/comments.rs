//! Comment endpoints for Gamma API client.

use tracing::{instrument, trace};

use crate::error::Result;
use crate::gamma::types::{Comment, GetCommentsByUserAddressRequest, GetCommentsRequest};

use super::Client;

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
