//! Positions API methods.

use crate::data::types::{
    ClosedPosition, GetUserClosedPositionsRequest, GetUserPositionsRequest, Position,
    UserPositionValue, validate_market_id, validate_user,
};
use crate::error::Result;

use super::Client;

impl Client {
    /// Gets the current positions for a user.
    ///
    /// # Arguments
    ///
    /// * `request` - Request parameters. See [`GetUserPositionsRequest`] for details.
    ///
    /// # Returns
    ///
    /// Returns a vector of `Position` containing the user's positions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::{Client, GetUserPositionsRequest, PositionSortBy, SortDirection};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     
    ///     // Get all positions for a user
    ///     let positions = client.get_user_positions(GetUserPositionsRequest {
    ///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ///         ..Default::default()
    ///     }).await?;
    ///     
    ///     for pos in &positions {
    ///         println!("Position: {} - Size: {} - PnL: {}", pos.title, pos.size, pos.cash_pnl);
    ///     }
    ///     
    ///     // Get positions with filters
    ///     let positions = client.get_user_positions(GetUserPositionsRequest {
    ///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ///         size_threshold: Some(1.0),
    ///         redeemable: Some(false),
    ///         mergeable: Some(false),
    ///         limit: Some(10),
    ///         offset: Some(0),
    ///         sort_by: Some(PositionSortBy::CashPnl),
    ///         sort_direction: Some(SortDirection::Desc),
    ///         ..Default::default()
    ///     }).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_user_positions(
        &self,
        request: GetUserPositionsRequest<'_>,
    ) -> Result<Vec<Position>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let positions: Vec<Position> = response.json().await?;
        Ok(positions)
    }

    /// Gets the closed positions for a user.
    ///
    /// # Arguments
    ///
    /// * `request` - Request parameters. See [`GetUserClosedPositionsRequest`] for details.
    ///
    /// # Returns
    ///
    /// Returns a vector of `ClosedPosition` containing the user's closed positions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::{Client, GetUserClosedPositionsRequest, ClosedPositionSortBy, SortDirection};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     
    ///     // Get all closed positions for a user
    ///     let positions = client.get_user_closed_positions(GetUserClosedPositionsRequest {
    ///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ///         ..Default::default()
    ///     }).await?;
    ///     
    ///     for pos in &positions {
    ///         println!("Closed Position: {} - Realized PnL: {}", pos.title, pos.realized_pnl);
    ///     }
    ///     
    ///     // Get closed positions with filters
    ///     let positions = client.get_user_closed_positions(GetUserClosedPositionsRequest {
    ///         user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ///         limit: Some(10),
    ///         offset: Some(0),
    ///         sort_by: Some(ClosedPositionSortBy::RealizedPnl),
    ///         sort_direction: Some(SortDirection::Desc),
    ///         ..Default::default()
    ///     }).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_user_closed_positions(
        &self,
        request: GetUserClosedPositionsRequest<'_>,
    ) -> Result<Vec<ClosedPosition>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let positions: Vec<ClosedPosition> = response.json().await?;
        Ok(positions)
    }

    /// Gets the total value of a user's positions.
    ///
    /// # Arguments
    ///
    /// * `user` - User Profile Address (0x-prefixed, 40 hex chars).
    /// * `markets` - Optional slice of market IDs to filter by (0x-prefixed, 64 hex chars each).
    ///
    /// # Returns
    ///
    /// Returns a vector of `ValueResponse` containing the user address and total position value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     // Get total value across all markets
    ///     let values = client.get_user_portfolio_value("0x56687bf447db6ffa42ffe2204a05edaa20f55839", None).await?;
    ///     for value in &values {
    ///         println!("User {} has total value: {}", value.user, value.value);
    ///     }
    ///
    ///     // Get value for specific markets
    ///     let markets = vec![
    ///         "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",
    ///     ];
    ///     let values = client.get_user_portfolio_value("0x56687bf447db6ffa42ffe2204a05edaa20f55839", Some(&markets)).await?;
    ///     for value in &values {
    ///         println!("User {} has value: {}", value.user, value.value);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_user_portfolio_value(
        &self,
        user: &str,
        markets: Option<&[&str]>,
    ) -> Result<Vec<UserPositionValue>> {
        validate_user(user)?;

        // Validate all market IDs if provided
        if let Some(market_ids) = markets {
            for market_id in market_ids {
                validate_market_id(market_id)?;
            }
        }

        let mut url = self.build_url("value");
        url.query_pairs_mut().append_pair("user", user);

        // Add market query parameter (comma-separated) if provided
        if let Some(market_ids) = markets.filter(|ids| !ids.is_empty()) {
            let market_value = market_ids.join(",");
            url.query_pairs_mut().append_pair("market", &market_value);
        }

        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let value_response: Vec<UserPositionValue> = response.json().await?;
        Ok(value_response)
    }
}
