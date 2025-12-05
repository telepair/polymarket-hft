//! Trades API methods.

use tracing::{instrument, trace};

use crate::data::types::{GetTradesRequest, Trade, UserTradedMarketsCount, validate_user};
use crate::error::Result;

use super::Client;

impl Client {
    /// Gets trades for a user or markets.
    ///
    /// # Arguments
    ///
    /// * `request` - Request parameters. See [`GetTradesRequest`] for details.
    ///
    /// # Returns
    ///
    /// Returns a vector of `Trade` containing the trade records.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::{Client, GetTradesRequest, TradeSide, TradeFilterType};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     
    ///     // Get trades for a user
    ///     let trades = client.get_trades(GetTradesRequest {
    ///         user: Some("0x56687bf447db6ffa42ffe2204a05edaa20f55839"),
    ///         limit: Some(100),
    ///         ..Default::default()
    ///     }).await?;
    ///     
    ///     for trade in &trades {
    ///         println!("Trade: {} {} @ {} - {}", trade.side, trade.size, trade.price, trade.title);
    ///     }
    ///     
    ///     // Get trades for a market with filters
    ///     let markets = vec![
    ///         "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",
    ///     ];
    ///     let trades = client.get_trades(GetTradesRequest {
    ///         markets: Some(&markets),
    ///         limit: Some(50),
    ///         taker_only: Some(true),
    ///         filter_type: Some(TradeFilterType::Cash),
    ///         filter_amount: Some(10.0),
    ///         side: Some(TradeSide::Buy),
    ///         ..Default::default()
    ///     }).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_trades(&self, request: GetTradesRequest<'_>) -> Result<Vec<Trade>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let trades: Vec<Trade> = response.json().await?;
        trace!(count = trades.len(), "received trades");
        Ok(trades)
    }

    /// Gets the total number of markets a user has traded.
    ///
    /// # Arguments
    ///
    /// * `user` - User Profile Address (0x-prefixed, 40 hex chars).
    ///
    /// # Returns
    ///
    /// Returns a `UserTradedMarketsCount` containing the user address and total traded markets.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let traded = client.get_user_traded_markets("0x56687bf447db6ffa42ffe2204a05edaa20f55839").await?;
    ///     println!("User {} has traded {} markets", traded.user, traded.traded);
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), fields(user = %user), level = "trace")]
    pub async fn get_user_traded_markets(&self, user: &str) -> Result<UserTradedMarketsCount> {
        validate_user(user)?;

        let mut url = self.build_url("traded");
        url.query_pairs_mut().append_pair("user", user);

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let traded_response: UserTradedMarketsCount = response.json().await?;
        trace!(
            traded = traded_response.traded,
            "received traded markets count"
        );
        Ok(traded_response)
    }
}
