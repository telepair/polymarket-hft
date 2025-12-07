//! Sports metadata and endpoints for the Gamma API.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

use crate::error::Result;

use super::Client;

/// Team representation returned by the Gamma API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: u64,
    pub name: Option<String>,
    pub league: Option<String>,
    pub record: Option<String>,
    pub logo: Option<String>,
    pub abbreviation: Option<String>,
    pub alias: Option<String>,
    #[serde(alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(alias = "updatedAt")]
    pub updated_at: Option<String>,
}

/// Sport metadata entry (e.g., league or competition).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SportMetadata {
    pub sport: String,
    pub image: Option<String>,
    pub resolution: Option<String>,
    pub ordering: Option<String>,
    /// Comma-separated tag IDs associated with the sport.
    pub tags: Option<String>,
    /// Series ID associated with this sport.
    pub series: Option<String>,
}

/// Request parameters for listing teams.
#[derive(Debug, Clone, Default)]
pub struct GetTeamsRequest<'a> {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order: Option<&'a str>,
    pub ascending: Option<bool>,
    pub league: Option<Vec<String>>,
    pub name: Option<Vec<String>>,
    pub abbreviation: Option<Vec<String>>,
}

impl<'a> GetTeamsRequest<'a> {
    /// Builds the request URL using the provided base URL.
    pub(crate) fn build_url(&self, base_url: &url::Url) -> url::Url {
        let mut url = base_url.clone();
        url.set_path("teams");
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
            if let Some(leagues) = &self.league {
                for league in leagues {
                    pairs.append_pair("league", league);
                }
            }
            if let Some(names) = &self.name {
                for name in names {
                    pairs.append_pair("name", name);
                }
            }
            if let Some(abbreviations) = &self.abbreviation {
                for abbreviation in abbreviations {
                    pairs.append_pair("abbreviation", abbreviation);
                }
            }
        }
        url
    }
}

// -----------------------------------------------------------------------------
// Client implementation
// -----------------------------------------------------------------------------

impl Client {
    /// Lists sports teams with optional filters.
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_teams(&self, request: GetTeamsRequest<'_>) -> Result<Vec<Team>> {
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let teams: Vec<Team> = response.json().await?;
        trace!(count = teams.len(), "received teams");
        Ok(teams)
    }

    /// Lists all sports metadata.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_sports(&self) -> Result<Vec<SportMetadata>> {
        let url = self.build_url("sports");
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let sports: Vec<SportMetadata> = response.json().await?;
        trace!(count = sports.len(), "received sports");
        Ok(sports)
    }
}
