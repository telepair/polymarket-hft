//! Activity API methods.

use tracing::{instrument, trace};

use crate::data::types::{Activity, GetUserActivityRequest};
use crate::error::Result;

use super::Client;

impl Client {
    /// Gets the on-chain activity for a user.
    ///
    /// # Arguments
    ///
    /// * `request` - Request parameters. See [`GetUserActivityRequest`] for details.
    ///
    /// # Returns
    ///
    /// Returns a vector of `Activity` containing the user's on-chain activity.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::{Client, GetUserActivityRequest, ActivityType, ActivitySortBy, SortDirection};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     
    ///     // Get all activity for a user
    ///     let activity = client.get_user_activity(GetUserActivityRequest {
    ///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ///         ..Default::default()
    ///     }).await?;
    ///     
    ///     for act in &activity {
    ///         println!("Activity: {:?} - Size: {} - Price: {}", act.activity_type, act.size, act.price);
    ///     }
    ///     
    ///     // Get activity with filters
    ///     let activity_types = vec![ActivityType::Trade];
    ///     let activity = client.get_user_activity(GetUserActivityRequest {
    ///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ///         limit: Some(50),
    ///         activity_types: Some(&activity_types),
    ///         sort_by: Some(ActivitySortBy::Timestamp),
    ///         sort_direction: Some(SortDirection::Desc),
    ///         ..Default::default()
    ///     }).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self, request), fields(user = %request.user), level = "trace")]
    pub async fn get_user_activity(
        &self,
        request: GetUserActivityRequest<'_>,
    ) -> Result<Vec<Activity>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let activity: Vec<Activity> = response.json().await?;
        trace!(count = activity.len(), "received activity records");
        Ok(activity)
    }
}
