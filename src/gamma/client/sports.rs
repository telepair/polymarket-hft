//! Sports endpoints for Gamma API client.

use tracing::{instrument, trace};

use crate::error::Result;
use crate::gamma::types::{GetTeamsRequest, SportMetadata, Team};

use super::Client;

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
