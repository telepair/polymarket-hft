use reqwest::Method;
use reqwest_middleware::ClientWithMiddleware;

use super::model::*;
use crate::client::http::{self, HttpClientConfig};

const BASE_URL: &str = "https://pro-api.coinmarketcap.com";

/// Helper macro to add optional query parameters to a request.
macro_rules! add_optional_query {
    ($req:expr, $($key:literal => $value:expr),* $(,)?) => {{
        let mut req = $req;
        $(
            if let Some(ref v) = $value {
                req = req.query(&[($key, v)]);
            }
        )*
        req
    }};
}

/// CoinMarketCap API client.
#[derive(Clone)]
pub struct Client {
    inner: ClientWithMiddleware,
    api_key: String,
    base_url: String,
}

impl Client {
    /// Creates a new CoinMarketCap API client.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: http::build_default_client().expect("Failed to build default HTTP client"),
            api_key: api_key.into(),
            base_url: BASE_URL.to_string(),
        }
    }

    /// Creates a new CoinMarketCap API client with custom configuration.
    pub fn with_config(api_key: impl Into<String>, config: HttpClientConfig) -> Self {
        Self {
            inner: config
                .build()
                .expect("Failed to build HTTP client with config"),
            api_key: api_key.into(),
            base_url: BASE_URL.to_string(),
        }
    }

    /// Sets the base URL (internal use or testing).
    #[allow(dead_code)]
    pub(crate) fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// Helper to create a request builder with the API key header.
    fn request(&self, method: Method, path: &str) -> reqwest_middleware::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.inner
            .request(method, &url)
            .header("X-CMC_PRO_API_KEY", &self.api_key)
            .header("Accept", "application/json")
    }

    /// Check response status and return error if API returned an error.
    fn check_status(status: &Status) -> Result<(), CmcError> {
        if status.error_code != 0 {
            return Err(CmcError::Api {
                code: status.error_code,
                message: status.error_message.clone().unwrap_or_default(),
            });
        }
        Ok(())
    }

    /// Get latest cryptocurrency listings.
    pub async fn get_listings_latest(
        &self,
        request: GetListingsLatestRequest,
    ) -> Result<ListingsLatestResponse, CmcError> {
        let req = self.request(Method::GET, "/v1/cryptocurrency/listings/latest");

        let req = add_optional_query!(req,
            "start" => request.start,
            "limit" => request.limit,
            "price_min" => request.price_min,
            "price_max" => request.price_max,
            "market_cap_min" => request.market_cap_min,
            "market_cap_max" => request.market_cap_max,
            "volume_24h_min" => request.volume_24h_min,
            "volume_24h_max" => request.volume_24h_max,
            "circulating_supply_min" => request.circulating_supply_min,
            "circulating_supply_max" => request.circulating_supply_max,
            "percent_change_24h_min" => request.percent_change_24h_min,
            "percent_change_24h_max" => request.percent_change_24h_max,
            "convert" => request.convert,
            "convert_id" => request.convert_id,
            "sort" => request.sort,
            "sort_dir" => request.sort_dir,
            "cryptocurrency_type" => request.cryptocurrency_type,
            "tag" => request.tag,
            "aux" => request.aux,
        );

        let response = req.send().await?;
        let data = response.json::<ListingsLatestResponse>().await?;
        Self::check_status(&data.status)?;
        Ok(data)
    }

    /// Get latest global metrics quotes.
    pub async fn get_global_metrics_quotes_latest(
        &self,
        request: GetGlobalMetricsQuotesLatestRequest,
    ) -> Result<GlobalMetricsQuotesLatestResponse, CmcError> {
        let req = self.request(Method::GET, "/v1/global-metrics/quotes/latest");

        let req = add_optional_query!(req,
            "convert" => request.convert,
            "convert_id" => request.convert_id,
        );

        let response = req.send().await?;
        let data = response.json::<GlobalMetricsQuotesLatestResponse>().await?;
        Self::check_status(&data.status)?;
        Ok(data)
    }

    /// Get latest fear and greed index.
    pub async fn get_fear_and_greed_latest(
        &self,
        _request: GetFearAndGreedLatestRequest,
    ) -> Result<FearAndGreedResponse, CmcError> {
        let req = self.request(Method::GET, "/v3/fear-and-greed/latest");
        let response = req.send().await?;
        let data = response.json::<FearAndGreedResponse>().await?;
        Self::check_status(&data.status)?;
        Ok(data)
    }

    /// Get API key usage information.
    ///
    /// Returns information about your API plan and current usage,
    /// including daily and monthly credit limits and consumption.
    pub async fn get_key_info(&self) -> Result<KeyInfoResponse, CmcError> {
        let req = self.request(Method::GET, "/v1/key/info");
        let response = req.send().await?;
        let data = response.json::<KeyInfoResponse>().await?;
        Self::check_status(&data.status)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_listings_latest() {
        let mock_server = MockServer::start().await;
        let client = Client::new("test-key").with_base_url(mock_server.uri());

        let response_body = r#"{
            "status": {
                "timestamp": "2024-01-01T00:00:00.000Z",
                "error_code": 0,
                "error_message": null,
                "elapsed": 10,
                "credit_count": 1,
                "notice": null
            },
            "data": [
                {
                    "id": 1,
                    "name": "Bitcoin",
                    "symbol": "BTC",
                    "slug": "bitcoin",
                    "num_market_pairs": 1000,
                    "date_added": "2010-07-13T00:00:00.000Z",
                    "tags": ["mineable"],
                    "max_supply": 21000000,
                    "circulating_supply": 19000000.0,
                    "total_supply": 19000000.0,
                    "infinite_supply": false,
                    "platform": null,
                    "cmc_rank": 1,
                    "self_reported_circulating_supply": null,
                    "self_reported_market_cap": null,
                    "tvl_ratio": null,
                    "last_updated": "2024-01-01T00:00:00.000Z",
                    "quote": {
                        "USD": {
                            "price": 50000.0,
                            "volume_24h": 1000000000.0,
                            "volume_change_24h": 0.5,
                            "percent_change_1h": 0.1,
                            "percent_change_24h": 1.5,
                            "percent_change_7d": 5.0,
                            "market_cap": 950000000000.0,
                            "market_cap_dominance": 50.0,
                            "fully_diluted_market_cap": 1050000000000.0,
                            "last_updated": "2024-01-01T00:00:00.000Z"
                        }
                    }
                }
            ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/v1/cryptocurrency/listings/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let request = GetListingsLatestRequest {
            limit: Some(1),
            ..Default::default()
        };

        let result = client.get_listings_latest(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].symbol, "BTC");
    }

    #[tokio::test]
    async fn test_get_global_metrics_quotes_latest() {
        let mock_server = MockServer::start().await;
        let client = Client::new("test-key").with_base_url(mock_server.uri());

        let response_body = r#"{
            "status": {
                "timestamp": "2024-01-01T00:00:00.000Z",
                "error_code": 0,
                "error_message": null,
                "elapsed": 10,
                "credit_count": 1,
                "notice": null
            },
            "data": {
                "active_cryptocurrencies": 10000,
                "total_cryptocurrencies": 20000,
                "active_market_pairs": 50000,
                "active_exchanges": 500,
                "total_exchanges": 1000,
                "eth_dominance": 18.5,
                "btc_dominance": 50.5,
                "eth_dominance_yesterday": 18.0,
                "btc_dominance_yesterday": 50.0,
                "defi_volume_24h_reported": 5000000000.0,
                "stablecoin_volume_24h_reported": 40000000000.0,
                "der_volume_24h_reported": 30000000000.0,
                "quote": {
                    "USD": {
                        "total_market_cap": 2000000000000.0,
                        "total_volume_24h": 60000000000.0,
                        "total_volume_24h_reported": 60000000000.0,
                        "altcoin_volume_24h": 30000000000.0,
                        "altcoin_market_cap": 1000000000000.0,
                        "defi_volume_24h": 5000000000.0,
                        "defi_market_cap": 80000000000.0,
                        "defi_24h_percentage_change": 1.2,
                        "stablecoin_volume_24h": 40000000000.0,
                        "stablecoin_market_cap": 150000000000.0,
                        "stablecoin_24h_percentage_change": 0.1,
                        "der_volume_24h": 30000000000.0,
                        "der_24h_percentage_change": 2.5,
                        "last_updated": "2024-01-01T00:00:00.000Z"
                    }
                }
            }
        }"#;

        Mock::given(method("GET"))
            .and(path("/v1/global-metrics/quotes/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let request = GetGlobalMetricsQuotesLatestRequest::default();
        let result = client.get_global_metrics_quotes_latest(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.btc_dominance, 50.5);
    }

    #[tokio::test]
    async fn test_get_fear_and_greed_latest() {
        let mock_server = MockServer::start().await;
        let client = Client::new("test-key").with_base_url(mock_server.uri());

        let response_body = r#"{
            "status": {
                "timestamp": "2024-01-01T00:00:00.000Z",
                "error_code": 0,
                "error_message": null,
                "elapsed": 10,
                "credit_count": 1,
                "notice": null
            },
            "data": {
                "value": 75,
                "value_classification": "Greed",
                "timestamp": "1704067200",
                "time_until_update": "3600"
            }
        }"#;

        Mock::given(method("GET"))
            .and(path("/v3/fear-and-greed/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let request = GetFearAndGreedLatestRequest::default();
        let result = client.get_fear_and_greed_latest(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.value, 75.0);
        assert_eq!(response.data.value_classification, "Greed");
    }

    #[tokio::test]
    async fn test_get_key_info() {
        let mock_server = MockServer::start().await;
        let client = Client::new("test-key").with_base_url(mock_server.uri());

        let response_body = r#"{
            "status": {
                "timestamp": "2024-01-01T00:00:00.000Z",
                "error_code": 0,
                "error_message": null,
                "elapsed": 10,
                "credit_count": 1,
                "notice": null
            },
            "data": {
                "plan": {
                    "credit_limit_daily": 333,
                    "credit_limit_daily_reset": "2024-01-02T00:00:00.000Z",
                    "credit_limit_monthly": 10000,
                    "credit_limit_monthly_reset": "2024-02-01T00:00:00.000Z",
                    "rate_limit_minute": 30
                },
                "usage": {
                    "current_day": {
                        "credits_used": 50,
                        "credits_left": 283
                    },
                    "current_month": {
                        "credits_used": 1500,
                        "credits_left": 8500
                    }
                }
            }
        }"#;

        Mock::given(method("GET"))
            .and(path("/v1/key/info"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let result = client.get_key_info().await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.plan.credit_limit_daily, Some(333));
        assert_eq!(response.data.plan.credit_limit_monthly, Some(10000));
        let current_day = response.data.usage.current_day.as_ref().unwrap();
        assert_eq!(current_day.credits_used, Some(50));
        let current_month = response.data.usage.current_month.as_ref().unwrap();
        assert_eq!(current_month.credits_left, Some(8500));
    }

    #[tokio::test]
    async fn test_api_error_handling() {
        let mock_server = MockServer::start().await;
        let client = Client::new("invalid-key").with_base_url(mock_server.uri());

        let response_body = r#"{
            "status": {
                "timestamp": "2024-01-01T00:00:00.000Z",
                "error_code": 1001,
                "error_message": "This API Key is invalid.",
                "elapsed": 0,
                "credit_count": 0,
                "notice": null
            },
            "data": []
        }"#;

        Mock::given(method("GET"))
            .and(path("/v1/cryptocurrency/listings/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let result = client
            .get_listings_latest(GetListingsLatestRequest::default())
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            CmcError::Api { code, message } => {
                assert_eq!(code, 1001);
                assert_eq!(message, "This API Key is invalid.");
            }
            _ => panic!("Expected CmcError::Api, got {:?}", err),
        }
    }
}
