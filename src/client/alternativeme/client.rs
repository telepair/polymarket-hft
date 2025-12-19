//! Alternative.me API client implementation.

use reqwest::Method;
use reqwest_middleware::ClientWithMiddleware;

use super::model::*;
use crate::client::http::HttpClientConfig;

const BASE_URL: &str = "https://api.alternative.me";

/// Helper macro to add optional query parameters to a request.
macro_rules! add_query_params {
    ($req:expr, $(($key:expr, $value:expr)),* $(,)?) => {{
        let mut req = $req;
        $(
            if let Some(ref v) = $value {
                req = req.query(&[($key, v)]);
            }
        )*
        req
    }};
}

/// Alternative.me API client.
///
/// This client provides access to the Alternative.me free Crypto API,
/// which includes cryptocurrency prices, global metrics, and Fear & Greed Index.
///
/// No API key is required.
#[derive(Clone)]
pub struct Client {
    http_client: ClientWithMiddleware,
    base_url: String,
}

impl Client {
    /// Creates a new Alternative.me API client with default configuration.
    pub fn new() -> Self {
        Self::with_config(HttpClientConfig::default())
    }

    /// Creates a new Alternative.me API client with custom configuration.
    pub fn with_config(config: HttpClientConfig) -> Self {
        let http_client = config
            .build()
            .expect("Failed to build HTTP client for Alternative.me");
        Self {
            http_client,
            base_url: BASE_URL.to_string(),
        }
    }

    /// Sets the base URL (internal use or testing).
    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// Helper to create a request builder.
    fn request(&self, method: Method, path: &str) -> reqwest_middleware::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.http_client.request(method, url)
    }

    /// Check response metadata for errors.
    fn check_metadata_error(error: &Option<String>) -> Result<(), AlternativeMeError> {
        if let Some(err) = error.as_ref().filter(|e| !e.is_empty()) {
            return Err(AlternativeMeError::Api(err.clone()));
        }
        Ok(())
    }

    /// Get all cryptocurrency listings.
    ///
    /// Returns a list of all cryptocurrencies with their id, name, symbol, and slug.
    pub async fn get_listings(&self) -> Result<ListingsResponse, AlternativeMeError> {
        let response: ListingsResponse = self
            .request(Method::GET, "/v2/listings/")
            .send()
            .await?
            .json()
            .await?;
        Self::check_metadata_error(&response.metadata.error)?;
        Ok(response)
    }

    /// Get cryptocurrency ticker data.
    ///
    /// Returns price, volume, market cap, and percentage changes for cryptocurrencies.
    pub async fn get_ticker(
        &self,
        request: GetTickerRequest,
    ) -> Result<TickerArrayResponse, AlternativeMeError> {
        let req = self.request(Method::GET, "/v2/ticker/");
        let req = add_query_params!(
            req,
            ("limit", request.limit),
            ("start", request.start),
            ("convert", request.convert),
            ("sort", request.sort),
        );
        // Force array structure for consistent response type
        let req = req.query(&[("structure", "array")]);

        let response: TickerArrayResponse = req.send().await?.json().await?;
        Self::check_metadata_error(&response.metadata.error)?;
        Ok(response)
    }

    /// Get ticker data for a specific cryptocurrency.
    ///
    /// The id can be either the numeric id or the website_slug (e.g., "bitcoin").
    pub async fn get_ticker_by_id(
        &self,
        id: &str,
        request: GetTickerByIdRequest,
    ) -> Result<TickerDictResponse, AlternativeMeError> {
        let path = format!("/v2/ticker/{}/", id);
        let req = self.request(Method::GET, &path);
        let req = add_query_params!(req, ("convert", request.convert),);

        let response: TickerDictResponse = req.send().await?.json().await?;
        Self::check_metadata_error(&response.metadata.error)?;
        Ok(response)
    }

    /// Get global market metrics.
    ///
    /// Returns total market cap, 24h volume, and Bitcoin dominance.
    pub async fn get_global(
        &self,
        request: GetGlobalRequest,
    ) -> Result<GlobalResponse, AlternativeMeError> {
        let req = self.request(Method::GET, "/v2/global/");
        let req = add_query_params!(req, ("convert", request.convert),);

        let response: GlobalResponse = req.send().await?.json().await?;
        Self::check_metadata_error(&response.metadata.error)?;
        Ok(response)
    }

    /// Get Fear and Greed Index.
    ///
    /// Returns the current Fear and Greed Index value (0-100) and classification.
    pub async fn get_fear_and_greed(
        &self,
        request: GetFearAndGreedRequest,
    ) -> Result<FearAndGreedResponse, AlternativeMeError> {
        let req = self.request(Method::GET, "/fng/");
        let req = add_query_params!(
            req,
            ("limit", request.limit),
            ("format", request.format),
            ("date_format", request.date_format),
        );

        let response: FearAndGreedResponse = req.send().await?.json().await?;
        Self::check_metadata_error(&response.metadata.error)?;
        Ok(response)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_listings() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "data": [
                {"id": "1", "name": "Bitcoin", "symbol": "BTC", "website_slug": "bitcoin"},
                {"id": "2", "name": "Litecoin", "symbol": "LTC", "website_slug": "litecoin"}
            ],
            "metadata": {"timestamp": 1537430627, "num_cryptocurrencies": 935, "error": null}
        }"#;

        Mock::given(method("GET"))
            .and(path("/v2/listings/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let client = Client::new().with_base_url(mock_server.uri());
        let result = client.get_listings().await;

        assert!(result.is_ok());
        let listings = result.unwrap();
        assert_eq!(listings.data.len(), 2);
        assert_eq!(listings.data[0].name, "Bitcoin");
        assert_eq!(listings.data[0].symbol, "BTC");
    }

    #[tokio::test]
    async fn test_get_ticker() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "data": [
                {
                    "id": 1,
                    "name": "Bitcoin",
                    "symbol": "BTC",
                    "website_slug": "bitcoin",
                    "rank": 1,
                    "circulating_supply": 17277612,
                    "total_supply": 17277612,
                    "max_supply": 21000000,
                    "quotes": {
                        "USD": {
                            "price": 6418.85,
                            "volume_24h": 4263700490.8,
                            "market_cap": 110902541529,
                            "percent_change_1h": 0.1,
                            "percent_change_24h": 0.84,
                            "percent_change_7d": -0.23
                        }
                    },
                    "last_updated": 1537428143
                }
            ],
            "metadata": {"timestamp": 1537428090, "num_cryptocurrencies": 935, "error": null}
        }"#;

        Mock::given(method("GET"))
            .and(path("/v2/ticker/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let client = Client::new().with_base_url(mock_server.uri());
        let result = client
            .get_ticker(GetTickerRequest {
                limit: Some(10),
                ..Default::default()
            })
            .await;

        assert!(result.is_ok());
        let ticker = result.unwrap();
        assert_eq!(ticker.data.len(), 1);
        assert_eq!(ticker.data[0].name, "Bitcoin");
        assert!(ticker.data[0].quotes.contains_key("USD"));
    }

    #[tokio::test]
    async fn test_get_ticker_by_id() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "data": {
                "1": {
                    "id": 1,
                    "name": "Bitcoin",
                    "symbol": "BTC",
                    "website_slug": "bitcoin",
                    "rank": 1,
                    "circulating_supply": 17277650,
                    "total_supply": 17277650,
                    "max_supply": 21000000,
                    "quotes": {
                        "USD": {
                            "price": 6420.75,
                            "volume_24h": 4234633625.35,
                            "market_cap": 110935522069,
                            "percent_change_1h": 0.08,
                            "percent_change_24h": 0.89,
                            "percent_change_7d": -0.26
                        }
                    },
                    "last_updated": 1537430662
                }
            },
            "metadata": {"timestamp": 1537430662, "num_cryptocurrencies": 935, "error": null}
        }"#;

        Mock::given(method("GET"))
            .and(path("/v2/ticker/bitcoin/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let client = Client::new().with_base_url(mock_server.uri());
        let result = client
            .get_ticker_by_id("bitcoin", GetTickerByIdRequest::default())
            .await;

        assert!(result.is_ok());
        let ticker = result.unwrap();
        assert!(ticker.data.contains_key("1"));
        assert_eq!(ticker.data["1"].name, "Bitcoin");
    }

    #[tokio::test]
    async fn test_get_global() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "data": {
                "active_cryptocurrencies": 935,
                "active_markets": 15625,
                "bitcoin_percentage_of_market_cap": 55.38,
                "quotes": {
                    "USD": {
                        "total_market_cap": 199872021187,
                        "total_volume_24h": 11884460489
                    }
                },
                "last_updated": 1537438163
            },
            "metadata": {"timestamp": 1537438413, "error": null}
        }"#;

        Mock::given(method("GET"))
            .and(path("/v2/global/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let client = Client::new().with_base_url(mock_server.uri());
        let result = client.get_global(GetGlobalRequest::default()).await;

        assert!(result.is_ok());
        let global = result.unwrap();
        assert_eq!(global.data.active_cryptocurrencies, 935);
        assert!(global.data.quotes.contains_key("USD"));
    }

    #[tokio::test]
    async fn test_get_fear_and_greed() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "name": "Fear and Greed Index",
            "data": [
                {
                    "value": "40",
                    "value_classification": "Fear",
                    "timestamp": "1551157200",
                    "time_until_update": "68499"
                }
            ],
            "metadata": {"error": null}
        }"#;

        Mock::given(method("GET"))
            .and(path("/fng/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let client = Client::new().with_base_url(mock_server.uri());
        let result = client
            .get_fear_and_greed(GetFearAndGreedRequest::default())
            .await;

        assert!(result.is_ok());
        let fng = result.unwrap();
        assert_eq!(fng.name, "Fear and Greed Index");
        assert_eq!(fng.data.len(), 1);
        assert_eq!(fng.data[0].value, "40");
        assert_eq!(fng.data[0].value_classification, "Fear");
    }

    #[tokio::test]
    async fn test_api_error_handling() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "data": [],
            "metadata": {"timestamp": 1537430627, "error": "Invalid request"}
        }"#;

        Mock::given(method("GET"))
            .and(path("/v2/listings/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let client = Client::new().with_base_url(mock_server.uri());
        let result = client.get_listings().await;

        assert!(result.is_err());
        match result {
            Err(AlternativeMeError::Api(msg)) => {
                assert_eq!(msg, "Invalid request");
            }
            _ => panic!("Expected API error"),
        }
    }
}
