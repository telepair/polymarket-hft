//! Alternative.me API client implementation.

use anyhow::anyhow;
use reqwest::Method;
use reqwest_middleware::ClientWithMiddleware;
use serde_json::Value;

use super::model::*;
use crate::Metric;
use crate::add_query_params;
use crate::client::http::HttpClientConfig;
use crate::client::{DataSourceClient, MethodMetadata, MethodParam, MetricInfo};

const BASE_URL: &str = "https://api.alternative.me";

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

    /// Sets the base URL
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

    /// Get cryptocurrency ticker data.
    ///
    /// Returns price, volume, market cap, and percentage changes for cryptocurrencies.
    pub async fn list_ticker(
        &self,
        limit: Option<i32>,
        start: Option<i32>,
        sort: Option<String>,
    ) -> Result<TickerArrayResponse, AlternativeMeError> {
        let req = self.request(Method::GET, "/v2/ticker/");
        let req = add_query_params!(req, ("limit", limit), ("start", start), ("sort", sort),);
        // Force array structure for consistent response type
        let req = req.query(&[("structure", "array")]);

        let response: TickerArrayResponse = req.send().await?.json().await?;
        Self::check_metadata_error(&response.metadata.error)?;
        Ok(response)
    }

    /// Get ticker data for a specific cryptocurrency.
    ///
    /// The target can be either the numeric id or the name (e.g., "bitcoin").
    pub async fn get_ticker(
        &self,
        target: String,
    ) -> Result<TickerArrayResponse, AlternativeMeError> {
        let path = format!("/v2/ticker/{}/", target);
        let req = self.request(Method::GET, &path);
        let req = req.query(&[("structure", "array")]);

        let response: TickerArrayResponse = req.send().await?.json().await?;
        Self::check_metadata_error(&response.metadata.error)?;
        Ok(response)
    }

    /// Get global market metrics.
    ///
    /// Returns total market cap, 24h volume, and Bitcoin dominance.
    pub async fn get_global(&self) -> Result<GlobalResponse, AlternativeMeError> {
        let req = self.request(Method::GET, "/v2/global/");

        let response: GlobalResponse = req.send().await?.json().await?;
        Self::check_metadata_error(&response.metadata.error)?;
        Ok(response)
    }

    /// Get Fear and Greed Index.
    ///
    /// Returns the current Fear and Greed Index value (0-100) and classification.
    /// The `limit` parameter controls how many historical data points to return (0 for all, default: 1).
    pub async fn get_fear_and_greed(
        &self,
        limit: Option<i32>,
    ) -> Result<FearAndGreedResponse, AlternativeMeError> {
        let req = self.request(Method::GET, "/fng/");
        let req = add_query_params!(req, ("limit", limit),);

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

impl DataSourceClient for Client {
    fn supported_methods(&self) -> Vec<MethodMetadata> {
        vec![
            MethodMetadata {
                method: "get_fear_and_greed",
                description: "Fear and Greed Index",
                params: vec![],
                metrics: vec![MetricInfo {
                    name: "fear_and_greed_index",
                    description: "Fear and Greed Index (0-100)",
                }],
            },
            MethodMetadata {
                method: "get_global",
                description: "Global market metrics",
                params: vec![],
                metrics: vec![
                    MetricInfo {
                        name: "active_cryptocurrencies",
                        description: "Active Cryptocurrencies",
                    },
                    MetricInfo {
                        name: "active_markets",
                        description: "Active Markets",
                    },
                    MetricInfo {
                        name: "bitcoin_dominance",
                        description: "Bitcoin Dominance",
                    },
                    MetricInfo {
                        name: "total_market_cap",
                        description: "Total Market Cap (USD)",
                    },
                    MetricInfo {
                        name: "total_volume_24h",
                        description: "Total Volume 24h (USD)",
                    },
                ],
            },
            MethodMetadata {
                method: "get_ticker",
                description: "Get ticker for a specific cryptocurrency",
                params: vec![MethodParam {
                    name: "target",
                    description: "Cryptocurrency ID or slug (e.g., 'bitcoin')",
                    required: true,
                }],
                metrics: vec![
                    MetricInfo {
                        name: "<symbol>_price",
                        description: "Price (per currency)",
                    },
                    MetricInfo {
                        name: "<symbol>_market_cap",
                        description: "Market Capitalization (per currency)",
                    },
                    MetricInfo {
                        name: "<symbol>_volume_24h",
                        description: "Volume 24h (per currency)",
                    },
                    MetricInfo {
                        name: "<symbol>_percent_change_1h",
                        description: "1h Percent Change (per currency)",
                    },
                    MetricInfo {
                        name: "<symbol>_percent_change_24h",
                        description: "24h Percent Change (per currency)",
                    },
                    MetricInfo {
                        name: "<symbol>_percent_change_7d",
                        description: "7d Percent Change (per currency)",
                    },
                ],
            },
        ]
    }

    fn fetch<'a>(
        &'a self,
        method: &'a str,
        params: Option<Value>,
    ) -> crate::client::BoxFuture<'a, anyhow::Result<Vec<Metric>>> {
        Box::pin(async move {
            match method {
                "get_fear_and_greed" => {
                    let resp = self.get_fear_and_greed(Some(1)).await?;
                    Ok(resp.to_metric())
                }
                "get_global" => {
                    let resp = self.get_global().await?;
                    Ok(resp.to_metrics())
                }
                "get_ticker" => {
                    let target = params
                        .as_ref()
                        .and_then(|p| p.get("target")?.as_str())
                        .ok_or_else(|| anyhow!("get_ticker requires 'target' parameter"))?
                        .to_string();
                    let resp = self.get_ticker(target).await?;
                    Ok(resp.to_metrics())
                }
                _ => Err(anyhow!("Unknown method: {}", method)),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_list_ticker() {
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
        let result = client.list_ticker(Some(10), None, None).await;

        assert!(result.is_ok());
        let ticker = result.unwrap();
        assert_eq!(ticker.data.len(), 1);
        assert_eq!(ticker.data[0].name, "Bitcoin");
        assert!(ticker.data[0].quotes.contains_key("USD"));
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
            ],
            "metadata": {"timestamp": 1537430662, "num_cryptocurrencies": 935, "error": null}
        }"#;

        Mock::given(method("GET"))
            .and(path("/v2/ticker/bitcoin/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let client = Client::new().with_base_url(mock_server.uri());
        let result = client.get_ticker("bitcoin".to_string()).await;

        assert!(result.is_ok());
        let ticker = result.unwrap();
        assert!(!ticker.data.is_empty());
        let btc = ticker.data.iter().find(|t| t.id == 1).unwrap();
        assert_eq!(btc.name, "Bitcoin");
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
        let result = client.get_global().await;

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
        let result = client.get_fear_and_greed(None).await;

        assert!(result.is_ok());
        let fng = result.unwrap();
        assert_eq!(fng.name, "Fear and Greed Index");
        assert_eq!(fng.data.len(), 1);
        assert_eq!(fng.data[0].value, "40");
        assert_eq!(fng.data[0].value_classification, "Fear");
    }
}
