//! OrderBook types and endpoints for CLOB API.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

use crate::error::Result;

use super::Client;
use super::pricing::Side;

/// Price level in an order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Price at this level (as string to maintain precision).
    pub price: String,
    /// Total size at this price level.
    pub size: String,
}

/// Order book summary response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSummary {
    /// Market identifier.
    pub market: String,
    /// Asset identifier.
    pub asset_id: String,
    /// Timestamp of the order book snapshot.
    pub timestamp: String,
    /// Hash of the order book state.
    pub hash: String,
    /// Array of bid levels.
    pub bids: Vec<PriceLevel>,
    /// Array of ask levels.
    pub asks: Vec<PriceLevel>,
    /// Minimum order size for this market.
    pub min_order_size: String,
    /// Minimum price increment.
    pub tick_size: String,
    /// Whether negative risk is enabled.
    pub neg_risk: bool,
}

/// Request item for getting multiple order books.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrderBooksRequestItem {
    /// The unique identifier for the token.
    pub token_id: String,
    /// Optional side filter for this token (BUY or SELL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

// -----------------------------------------------------------------------------
// Client implementation
// -----------------------------------------------------------------------------

impl Client {
    /// Gets the order book summary for a specific token.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The unique identifier for the token.
    ///
    /// # Returns
    ///
    /// Returns an `OrderBookSummary` containing bids, asks, and market metadata.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let order_book = client.get_order_book("1234567890").await?;
    ///     println!("Market: {}", order_book.market);
    ///     println!("Bids: {:?}", order_book.bids);
    ///     println!("Asks: {:?}", order_book.asks);
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), fields(token_id = %token_id), level = "trace")]
    pub async fn get_order_book(&self, token_id: &str) -> Result<OrderBookSummary> {
        let mut url = self.build_url("book");
        url.query_pairs_mut().append_pair("token_id", token_id);

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let order_book: OrderBookSummary = response.json().await?;
        trace!(
            market = %order_book.market,
            bids_count = order_book.bids.len(),
            asks_count = order_book.asks.len(),
            "received order book"
        );
        Ok(order_book)
    }

    /// Gets order book summaries for multiple tokens.
    ///
    /// # Arguments
    ///
    /// * `request` - A slice of request items specifying token IDs and optional sides.
    ///
    /// # Returns
    ///
    /// Returns a vector of `OrderBookSummary` for the requested tokens.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::{Client, GetOrderBooksRequestItem};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let request = vec![
    ///         GetOrderBooksRequestItem { token_id: "123".to_string(), side: None },
    ///         GetOrderBooksRequestItem { token_id: "456".to_string(), side: None },
    ///     ];
    ///     let order_books = client.get_order_books(&request).await?;
    ///     for book in order_books {
    ///         println!("Market: {}", book.market);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_order_books(
        &self,
        request: &[GetOrderBooksRequestItem],
    ) -> Result<Vec<OrderBookSummary>> {
        let url = self.build_url("books");

        trace!(url = %url, method = "POST", count = request.len(), "sending HTTP request");
        let response = self.http_client.post(url).json(request).send().await?;
        let response = self.check_response(response).await?;
        let order_books: Vec<OrderBookSummary> = response.json().await?;
        trace!(count = order_books.len(), "received order books");
        Ok(order_books)
    }
}
