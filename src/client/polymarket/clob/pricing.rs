//! Pricing types and endpoints for CLOB API.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

use crate::error::Result;

use super::Client;

/// Market side for pricing operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Side {
    /// Buy side.
    #[serde(rename = "BUY")]
    #[default]
    Buy,
    /// Sell side.
    #[serde(rename = "SELL")]
    Sell,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

impl std::str::FromStr for Side {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BUY" => Ok(Side::Buy),
            "SELL" => Ok(Side::Sell),
            _ => Err(format!("Invalid side: '{}'. Valid options: BUY, SELL", s)),
        }
    }
}

/// Market price response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrice {
    /// The market price (as string to maintain precision).
    pub price: String,
}

/// Midpoint price response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidpointPrice {
    /// The midpoint price (as string to maintain precision).
    pub mid: String,
}

/// Price history point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryPoint {
    /// UTC timestamp.
    pub t: i64,
    /// Price.
    pub p: f64,
}

/// Price history response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistory {
    /// List of timestamp/price pairs.
    pub history: Vec<PriceHistoryPoint>,
}

/// Price history interval.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum PriceHistoryInterval {
    /// 1 minute.
    #[serde(rename = "1m")]
    OneMinute,
    /// 1 hour.
    #[serde(rename = "1h")]
    OneHour,
    /// 6 hours.
    #[serde(rename = "6h")]
    SixHours,
    /// 1 day.
    #[serde(rename = "1d")]
    #[default]
    OneDay,
    /// 1 week.
    #[serde(rename = "1w")]
    OneWeek,
    /// Maximum available history.
    #[serde(rename = "max")]
    Max,
}

impl std::fmt::Display for PriceHistoryInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PriceHistoryInterval::OneMinute => write!(f, "1m"),
            PriceHistoryInterval::OneHour => write!(f, "1h"),
            PriceHistoryInterval::SixHours => write!(f, "6h"),
            PriceHistoryInterval::OneDay => write!(f, "1d"),
            PriceHistoryInterval::OneWeek => write!(f, "1w"),
            PriceHistoryInterval::Max => write!(f, "max"),
        }
    }
}

impl std::str::FromStr for PriceHistoryInterval {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1m" => Ok(PriceHistoryInterval::OneMinute),
            "1h" => Ok(PriceHistoryInterval::OneHour),
            "6h" => Ok(PriceHistoryInterval::SixHours),
            "1d" => Ok(PriceHistoryInterval::OneDay),
            "1w" => Ok(PriceHistoryInterval::OneWeek),
            "max" => Ok(PriceHistoryInterval::Max),
            _ => Err(format!(
                "Invalid interval: '{}'. Valid options: 1m, 1h, 6h, 1d, 1w, max",
                s
            )),
        }
    }
}

/// Request for getting price history.
#[derive(Debug, Clone, Default)]
pub struct GetPriceHistoryRequest<'a> {
    /// The CLOB token ID for which to fetch price history.
    pub market: &'a str,
    /// The start time, a Unix timestamp in UTC.
    pub start_ts: Option<i64>,
    /// The end time, a Unix timestamp in UTC.
    pub end_ts: Option<i64>,
    /// A string representing a duration ending at the current time.
    /// Mutually exclusive with start_ts and end_ts.
    pub interval: Option<PriceHistoryInterval>,
    /// The resolution of the data, in minutes.
    pub fidelity: Option<i32>,
}

impl GetPriceHistoryRequest<'_> {
    /// Validates the request parameters.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if parameters are valid, or an error describing the issue.
    pub fn validate(&self) -> Result<()> {
        // Market is required and cannot be empty
        if self.market.is_empty() {
            return Err(crate::error::PolymarketError::bad_request(
                "market token ID is required".to_string(),
            ));
        }

        // interval is mutually exclusive with start_ts/end_ts
        if self.interval.is_some() && (self.start_ts.is_some() || self.end_ts.is_some()) {
            return Err(crate::error::PolymarketError::bad_request(
                "interval is mutually exclusive with start_ts and end_ts".to_string(),
            ));
        }

        // start_ts must be <= end_ts if both are provided
        if let (Some(start), Some(end)) = (self.start_ts, self.end_ts)
            && start > end
        {
            return Err(crate::error::PolymarketError::bad_request(
                "start_ts must be <= end_ts".to_string(),
            ));
        }

        // fidelity must be positive if provided
        if let Some(f) = self.fidelity
            && f <= 0
        {
            return Err(crate::error::PolymarketError::bad_request(
                "fidelity must be > 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// Request item for getting multiple market prices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPriceRequest {
    /// The unique identifier for the token.
    pub token_id: String,
    /// The side of the market (BUY or SELL).
    pub side: Side,
}

// -----------------------------------------------------------------------------
// Client implementation
// -----------------------------------------------------------------------------

impl Client {
    /// Gets the market price for a specific token and side.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The unique identifier for the token.
    /// * `side` - The side of the market (BUY or SELL).
    ///
    /// # Returns
    ///
    /// Returns a `MarketPrice` containing the price.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::{Client, Side};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let price = client.get_market_price("1234567890", Side::Buy).await?;
    ///     println!("Price: {}", price.price);
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), fields(token_id = %token_id, side = %side), level = "trace")]
    pub async fn get_market_price(&self, token_id: &str, side: Side) -> Result<MarketPrice> {
        let mut url = self.build_url("price");
        url.query_pairs_mut()
            .append_pair("token_id", token_id)
            .append_pair("side", &side.to_string());

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let price: MarketPrice = response.json().await?;
        trace!(price = %price.price, "received market price");
        Ok(price)
    }

    /// Gets market prices for all tokens.
    ///
    /// # Returns
    ///
    /// Returns a map of token_id to side to price.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let prices = client.get_market_prices().await?;
    ///     for (token_id, side_prices) in prices {
    ///         println!("Token: {}", token_id);
    ///         for (side, price) in side_prices {
    ///             println!("  {}: {}", side, price);
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), level = "trace")]
    pub async fn get_market_prices(&self) -> Result<HashMap<String, HashMap<String, String>>> {
        let url = self.build_url("prices");

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let prices: HashMap<String, HashMap<String, String>> = response.json().await?;
        trace!(count = prices.len(), "received market prices");
        Ok(prices)
    }

    /// Gets market prices for specified tokens and sides via POST request.
    ///
    /// # Arguments
    ///
    /// * `request` - A slice of request items specifying token IDs and sides.
    ///
    /// # Returns
    ///
    /// Returns a map of token_id to side to price.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::{Client, MarketPriceRequest, Side};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let request = vec![
    ///         MarketPriceRequest { token_id: "123".to_string(), side: Side::Buy },
    ///         MarketPriceRequest { token_id: "456".to_string(), side: Side::Sell },
    ///     ];
    ///     let prices = client.get_market_prices_by_request(&request).await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_market_prices_by_request(
        &self,
        request: &[MarketPriceRequest],
    ) -> Result<HashMap<String, HashMap<String, String>>> {
        let url = self.build_url("prices");

        trace!(url = %url, method = "POST", count = request.len(), "sending HTTP request");
        let response = self.http_client.post(url).json(request).send().await?;
        let response = self.check_response(response).await?;
        let prices: HashMap<String, HashMap<String, String>> = response.json().await?;
        trace!(count = prices.len(), "received market prices");
        Ok(prices)
    }

    /// Gets the midpoint price for a specific token.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The unique identifier for the token.
    ///
    /// # Returns
    ///
    /// Returns a `MidpointPrice` containing the midpoint price.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let midpoint = client.get_midpoint_price("1234567890").await?;
    ///     println!("Midpoint: {}", midpoint.mid);
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), fields(token_id = %token_id), level = "trace")]
    pub async fn get_midpoint_price(&self, token_id: &str) -> Result<MidpointPrice> {
        let mut url = self.build_url("midpoint");
        url.query_pairs_mut().append_pair("token_id", token_id);

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let midpoint: MidpointPrice = response.json().await?;
        trace!(mid = %midpoint.mid, "received midpoint price");
        Ok(midpoint)
    }

    /// Gets price history for a traded token.
    ///
    /// # Arguments
    ///
    /// * `request` - Request parameters including market ID and time range.
    ///
    /// # Returns
    ///
    /// Returns a `PriceHistory` containing historical timestamp/price pairs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::{Client, GetPriceHistoryRequest, PriceHistoryInterval};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let history = client.get_price_history(GetPriceHistoryRequest {
    ///         market: "1234567890",
    ///         interval: Some(PriceHistoryInterval::OneDay),
    ///         ..Default::default()
    ///     }).await?;
    ///     for point in history.history {
    ///         println!("Time: {}, Price: {}", point.t, point.p);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), fields(market = %request.market), level = "trace")]
    pub async fn get_price_history(
        &self,
        request: GetPriceHistoryRequest<'_>,
    ) -> Result<PriceHistory> {
        let mut url = self.build_url("prices-history");
        url.query_pairs_mut().append_pair("market", request.market);

        if let Some(start_ts) = request.start_ts {
            url.query_pairs_mut()
                .append_pair("startTs", &start_ts.to_string());
        }
        if let Some(end_ts) = request.end_ts {
            url.query_pairs_mut()
                .append_pair("endTs", &end_ts.to_string());
        }
        if let Some(interval) = request.interval {
            url.query_pairs_mut()
                .append_pair("interval", &interval.to_string());
        }
        if let Some(fidelity) = request.fidelity {
            url.query_pairs_mut()
                .append_pair("fidelity", &fidelity.to_string());
        }

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let history: PriceHistory = response.json().await?;
        trace!(count = history.history.len(), "received price history");
        Ok(history)
    }

    /// Gets the last trade price for a specific token.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The unique identifier for the token.
    ///
    /// # Returns
    ///
    /// Returns the last trade price as a string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let price = client.get_last_trade_price("1234567890").await?;
    ///     println!("Last trade price: {}", price);
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), fields(token_id = %token_id), level = "trace")]
    pub async fn get_last_trade_price(&self, token_id: &str) -> Result<String> {
        let mut url = self.build_url("last-trade-price");
        url.query_pairs_mut().append_pair("token_id", token_id);

        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;

        #[derive(Deserialize)]
        struct LastTradePriceResponse {
            price: String,
        }

        let result: LastTradePriceResponse = response.json().await?;
        trace!(price = %result.price, "received last trade price");
        Ok(result.price)
    }

    /// Gets last trade prices for multiple tokens.
    ///
    /// # Arguments
    ///
    /// * `token_ids` - A slice of token IDs.
    ///
    /// # Returns
    ///
    /// Returns a map of token_id to last trade price.
    #[instrument(skip(self, token_ids), level = "trace")]
    pub async fn get_last_trades_prices(
        &self,
        token_ids: &[String],
    ) -> Result<HashMap<String, String>> {
        let url = self.build_url("last-trades-prices");

        #[derive(Serialize)]
        struct Request {
            token_ids: Vec<String>,
        }

        let request = Request {
            token_ids: token_ids.to_vec(),
        };

        trace!(url = %url, method = "POST", count = token_ids.len(), "sending HTTP request");
        let response = self.http_client.post(url).json(&request).send().await?;
        let response = self.check_response(response).await?;
        let prices: HashMap<String, String> = response.json().await?;
        trace!(count = prices.len(), "received last trades prices");
        Ok(prices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_history_request_validate_empty_market() {
        let req = GetPriceHistoryRequest {
            market: "",
            ..Default::default()
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("market"));
    }

    #[test]
    fn test_price_history_request_validate_valid() {
        let req = GetPriceHistoryRequest {
            market: "token123",
            interval: Some(PriceHistoryInterval::OneDay),
            ..Default::default()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_price_history_request_validate_interval_exclusive_with_timestamps() {
        let req = GetPriceHistoryRequest {
            market: "token123",
            interval: Some(PriceHistoryInterval::OneDay),
            start_ts: Some(1000),
            ..Default::default()
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("mutually exclusive")
        );
    }

    #[test]
    fn test_price_history_request_validate_start_greater_than_end() {
        let req = GetPriceHistoryRequest {
            market: "token123",
            start_ts: Some(2000),
            end_ts: Some(1000),
            ..Default::default()
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("start_ts must be <= end_ts")
        );
    }

    #[test]
    fn test_price_history_request_validate_timestamps_valid() {
        let req = GetPriceHistoryRequest {
            market: "token123",
            start_ts: Some(1000),
            end_ts: Some(2000),
            ..Default::default()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_price_history_request_validate_fidelity_invalid() {
        let req = GetPriceHistoryRequest {
            market: "token123",
            fidelity: Some(0),
            ..Default::default()
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fidelity"));

        let req = GetPriceHistoryRequest {
            market: "token123",
            fidelity: Some(-5),
            ..Default::default()
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_price_history_request_validate_fidelity_valid() {
        let req = GetPriceHistoryRequest {
            market: "token123",
            fidelity: Some(5),
            ..Default::default()
        };
        assert!(req.validate().is_ok());
    }
}
