//! Event endpoints for Gamma API client.

use tracing::{instrument, trace};

use crate::error::Result;
use crate::gamma::types::{Event, GetEventsRequest, Tag};

use super::Client;

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
