//! CoinGecko API client.

use reqwest::Method;
use reqwest_middleware::ClientWithMiddleware;

use super::model::*;
use crate::client::http::{self, HttpClientConfig};

const BASE_URL: &str = "https://api.coingecko.com/api/v3";

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

/// Helper macro to add optional boolean query parameters.
macro_rules! add_optional_bool_query {
    ($req:expr, $($key:literal => $value:expr),* $(,)?) => {{
        let mut req = $req;
        $(
            if let Some(v) = $value {
                req = req.query(&[($key, if v { "true" } else { "false" })]);
            }
        )*
        req
    }};
}

/// CoinGecko API client.
#[derive(Clone)]
pub struct Client {
    inner: ClientWithMiddleware,
    api_key: String,
    base_url: String,
}

impl Client {
    /// Creates a new CoinGecko API client.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: http::build_default_client().expect("Failed to build default HTTP client"),
            api_key: api_key.into(),
            base_url: BASE_URL.to_string(),
        }
    }

    /// Creates a new CoinGecko API client with custom configuration.
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
            .header("x-cg-demo-api-key", &self.api_key)
            .header("Accept", "application/json")
    }

    /// Get simple price for one or more coins.
    ///
    /// Returns simple price data for the specified coin IDs in the target currencies.
    pub async fn get_simple_price(
        &self,
        request: SimplePriceRequest,
    ) -> Result<SimplePriceResponse, CgError> {
        let req = self.request(Method::GET, "/simple/price");

        let req = req
            .query(&[("ids", &request.ids)])
            .query(&[("vs_currencies", &request.vs_currencies)]);

        let req = add_optional_bool_query!(req,
            "include_market_cap" => request.include_market_cap,
            "include_24hr_vol" => request.include_24hr_vol,
            "include_24hr_change" => request.include_24hr_change,
            "include_last_updated_at" => request.include_last_updated_at,
        );

        let response = req.send().await?;
        let data = response.json::<SimplePriceResponse>().await?;
        Ok(data)
    }

    /// Get list of all supported coins.
    ///
    /// Returns a list of all supported coins with id, name, and symbol.
    pub async fn get_coins_list(
        &self,
        request: CoinsListRequest,
    ) -> Result<CoinsListResponse, CgError> {
        let req = self.request(Method::GET, "/coins/list");

        let req = add_optional_bool_query!(req,
            "include_platform" => request.include_platform,
        );

        let response = req.send().await?;
        let data = response.json::<CoinsListResponse>().await?;
        Ok(data)
    }

    /// Get coin market data.
    ///
    /// Returns market data including price, market cap, volume for coins.
    pub async fn get_coins_markets(
        &self,
        request: CoinsMarketsRequest,
    ) -> Result<CoinsMarketsResponse, CgError> {
        let req = self.request(Method::GET, "/coins/markets");

        let req = req.query(&[("vs_currency", &request.vs_currency)]);

        let per_page_str = request.per_page.map(|v| v.to_string());
        let page_str = request.page.map(|v| v.to_string());

        let req = add_optional_query!(req,
            "ids" => request.ids,
            "category" => request.category,
            "order" => request.order,
            "per_page" => per_page_str,
            "page" => page_str,
            "price_change_percentage" => request.price_change_percentage,
            "locale" => request.locale,
            "precision" => request.precision,
        );

        let req = add_optional_bool_query!(req,
            "sparkline" => request.sparkline,
        );

        let response = req.send().await?;
        let data = response.json::<CoinsMarketsResponse>().await?;
        Ok(data)
    }

    /// Get trending search coins, NFTs, and categories.
    ///
    /// Returns the top trending coins, NFTs, and categories based on user searches.
    pub async fn get_trending(&self) -> Result<TrendingResponse, CgError> {
        let req = self.request(Method::GET, "/search/trending");
        let response = req.send().await?;
        let data = response.json::<TrendingResponse>().await?;
        Ok(data)
    }

    /// Get global cryptocurrency data.
    ///
    /// Returns global crypto statistics including total market cap, volume, and dominance.
    pub async fn get_global(&self) -> Result<GlobalResponse, CgError> {
        let req = self.request(Method::GET, "/global");
        let response = req.send().await?;
        let data = response.json::<GlobalResponse>().await?;
        Ok(data)
    }

    /// Get list of supported vs currencies.
    ///
    /// Returns all supported vs currencies for price queries.
    pub async fn get_supported_vs_currencies(
        &self,
    ) -> Result<SupportedVsCurrenciesResponse, CgError> {
        let req = self.request(Method::GET, "/simple/supported_vs_currencies");
        let response = req.send().await?;
        let data = response.json::<SupportedVsCurrenciesResponse>().await?;
        Ok(data)
    }

    /// Get list of exchanges.
    ///
    /// Returns all exchanges with trading volume data.
    pub async fn get_exchanges(
        &self,
        request: ExchangesRequest,
    ) -> Result<ExchangesResponse, CgError> {
        let req = self.request(Method::GET, "/exchanges");

        let per_page_str = request.per_page.map(|v| v.to_string());
        let page_str = request.page.map(|v| v.to_string());

        let req = add_optional_query!(req,
            "per_page" => per_page_str,
            "page" => page_str,
        );

        let response = req.send().await?;
        let data = response.json::<ExchangesResponse>().await?;
        Ok(data)
    }

    /// Get coin detail by ID.
    ///
    /// Returns comprehensive data for a specific coin.
    pub async fn get_coin(
        &self,
        request: CoinDetailRequest,
    ) -> Result<CoinDetailResponse, CgError> {
        let path = format!("/coins/{}", request.id);
        let req = self.request(Method::GET, &path);

        let req = add_optional_bool_query!(req,
            "localization" => request.localization,
            "tickers" => request.tickers,
            "market_data" => request.market_data,
            "community_data" => request.community_data,
            "developer_data" => request.developer_data,
            "sparkline" => request.sparkline,
        );

        let response = req.send().await?;
        let data = response.json::<CoinDetailResponse>().await?;
        Ok(data)
    }

    /// Get historical market chart data for a coin.
    ///
    /// Returns price, market cap, and volume over time.
    pub async fn get_coin_market_chart(
        &self,
        request: MarketChartRequest,
    ) -> Result<MarketChartResponse, CgError> {
        let path = format!("/coins/{}/market_chart", request.id);
        let req = self.request(Method::GET, &path);

        let req = req
            .query(&[("vs_currency", &request.vs_currency)])
            .query(&[("days", &request.days)]);

        let req = add_optional_query!(req,
            "interval" => request.interval,
        );

        let response = req.send().await?;
        let data = response.json::<MarketChartResponse>().await?;
        Ok(data)
    }

    /// Get historical data for a coin at a specific date.
    ///
    /// Returns market data snapshot for the specified date.
    pub async fn get_coin_history(
        &self,
        request: CoinHistoryRequest,
    ) -> Result<CoinHistoryResponse, CgError> {
        let path = format!("/coins/{}/history", request.id);
        let req = self.request(Method::GET, &path);

        let req = req.query(&[("date", &request.date)]);

        let req = add_optional_bool_query!(req,
            "localization" => request.localization,
        );

        let response = req.send().await?;
        let data = response.json::<CoinHistoryResponse>().await?;
        Ok(data)
    }

    /// Get OHLC candlestick data for a coin.
    ///
    /// Returns Open, High, Low, Close data for charting.
    pub async fn get_coin_ohlc(&self, request: OhlcRequest) -> Result<OhlcResponse, CgError> {
        let path = format!("/coins/{}/ohlc", request.id);
        let req = self.request(Method::GET, &path);

        let req = req
            .query(&[("vs_currency", &request.vs_currency)])
            .query(&[("days", &request.days)]);

        let response = req.send().await?;
        let data = response.json::<OhlcResponse>().await?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_simple_price() {
        let mock_server = MockServer::start().await;
        let client = Client::new("test-key").with_base_url(mock_server.uri());

        let response_body = r#"{
            "bitcoin": {
                "usd": 50000.0,
                "usd_market_cap": 950000000000.0,
                "usd_24h_vol": 20000000000.0,
                "usd_24h_change": 1.5
            }
        }"#;

        Mock::given(method("GET"))
            .and(path("/simple/price"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let request = SimplePriceRequest {
            ids: "bitcoin".to_string(),
            vs_currencies: "usd".to_string(),
            include_market_cap: Some(true),
            include_24hr_vol: Some(true),
            include_24hr_change: Some(true),
            ..Default::default()
        };

        let result = client.get_simple_price(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains_key("bitcoin"));
        let bitcoin = response.get("bitcoin").unwrap();
        assert_eq!(bitcoin.get("usd").unwrap(), &Some(50000.0));
    }

    #[tokio::test]
    async fn test_get_coins_list() {
        let mock_server = MockServer::start().await;
        let client = Client::new("test-key").with_base_url(mock_server.uri());

        let response_body = r#"[
            {"id": "bitcoin", "symbol": "btc", "name": "Bitcoin"},
            {"id": "ethereum", "symbol": "eth", "name": "Ethereum"}
        ]"#;

        Mock::given(method("GET"))
            .and(path("/coins/list"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let request = CoinsListRequest::default();
        let result = client.get_coins_list(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.len(), 2);
        assert_eq!(response[0].id, "bitcoin");
        assert_eq!(response[1].id, "ethereum");
    }

    #[tokio::test]
    async fn test_get_coins_markets() {
        let mock_server = MockServer::start().await;
        let client = Client::new("test-key").with_base_url(mock_server.uri());

        let response_body = r#"[
            {
                "id": "bitcoin",
                "symbol": "btc",
                "name": "Bitcoin",
                "image": "https://example.com/btc.png",
                "current_price": 50000.0,
                "market_cap": 950000000000,
                "market_cap_rank": 1,
                "fully_diluted_valuation": 1050000000000,
                "total_volume": 20000000000,
                "high_24h": 51000.0,
                "low_24h": 49000.0,
                "price_change_24h": 1000.0,
                "price_change_percentage_24h": 2.0,
                "market_cap_change_24h": 10000000000,
                "market_cap_change_percentage_24h": 1.5,
                "circulating_supply": 19000000,
                "total_supply": 21000000,
                "max_supply": 21000000,
                "ath": 69000.0,
                "ath_change_percentage": -27.5,
                "ath_date": "2021-11-10T00:00:00.000Z",
                "atl": 67.81,
                "atl_change_percentage": 73700.0,
                "atl_date": "2013-07-06T00:00:00.000Z",
                "roi": null,
                "last_updated": "2024-01-01T00:00:00.000Z"
            }
        ]"#;

        Mock::given(method("GET"))
            .and(path("/coins/markets"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let request = CoinsMarketsRequest {
            vs_currency: "usd".to_string(),
            ids: Some("bitcoin".to_string()),
            per_page: Some(1),
            ..Default::default()
        };

        let result = client.get_coins_markets(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "bitcoin");
        assert_eq!(response[0].current_price, Some(50000.0));
    }

    #[tokio::test]
    async fn test_get_trending() {
        let mock_server = MockServer::start().await;
        let client = Client::new("test-key").with_base_url(mock_server.uri());

        let response_body = r#"{
            "coins": [
                {
                    "item": {
                        "id": "bitcoin",
                        "coin_id": 1,
                        "name": "Bitcoin",
                        "symbol": "BTC",
                        "market_cap_rank": 1,
                        "thumb": "https://example.com/btc-thumb.png",
                        "small": "https://example.com/btc-small.png",
                        "large": "https://example.com/btc-large.png",
                        "slug": "bitcoin",
                        "price_btc": 1.0,
                        "score": 0
                    }
                }
            ],
            "nfts": [],
            "categories": []
        }"#;

        Mock::given(method("GET"))
            .and(path("/search/trending"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let result = client.get_trending().await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.coins.len(), 1);
        assert_eq!(response.coins[0].item.id, "bitcoin");
    }

    #[tokio::test]
    async fn test_get_global() {
        let mock_server = MockServer::start().await;
        let client = Client::new("test-key").with_base_url(mock_server.uri());

        let response_body = r#"{
            "data": {
                "active_cryptocurrencies": 10000,
                "upcoming_icos": 0,
                "ongoing_icos": 50,
                "ended_icos": 3000,
                "markets": 800,
                "total_market_cap": {
                    "usd": 2000000000000.0
                },
                "total_volume": {
                    "usd": 80000000000.0
                },
                "market_cap_percentage": {
                    "btc": 50.5,
                    "eth": 18.5
                },
                "market_cap_change_percentage_24h_usd": 1.5,
                "updated_at": 1704067200
            }
        }"#;

        Mock::given(method("GET"))
            .and(path("/global"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let result = client.get_global().await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.active_cryptocurrencies, Some(10000));
        assert_eq!(response.data.market_cap_percentage.get("btc"), Some(&50.5));
    }
}
