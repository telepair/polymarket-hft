//! Data API client implementation.

use std::time::Duration;

use reqwest::{Client as HttpClient, Response};
use url::Url;

use crate::error::{PolymarketError, Result};

mod activity;
mod holders;
mod market;
mod positions;
mod trades;

// Re-export only types needed by this module
pub use super::types::HealthStatus;

/// Default base URL for the Polymarket Data API.
pub const DEFAULT_BASE_URL: &str = "https://data-api.polymarket.com";

/// Default request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default connection timeout in seconds.
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;

/// Maximum error message length to prevent sensitive data leakage.
const MAX_ERROR_MESSAGE_LEN: usize = 500;

/// Default maximum idle connections per host.
const DEFAULT_POOL_MAX_IDLE_PER_HOST: usize = 10;

/// Default idle timeout in seconds.
const DEFAULT_POOL_IDLE_TIMEOUT_SECS: u64 = 90;

/// Client for interacting with the Polymarket Data API.
#[derive(Debug, Clone)]
pub struct Client {
    /// HTTP client for making requests.
    http_client: HttpClient,
    /// Base URL for the API (validated URL).
    base_url: Url,
}

impl Client {
    /// Creates a new Data API client with the default base URL.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::Client;
    ///
    /// let client = Client::new();
    /// ```
    pub fn new() -> Self {
        // DEFAULT_BASE_URL is known to be valid, unwrap is safe
        Self::with_base_url(DEFAULT_BASE_URL).expect("default base URL is valid")
    }

    /// Creates a new Data API client with a custom base URL.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL for the API.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Client)` if the URL is valid, or an error if parsing fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::Client;
    ///
    /// let client = Client::with_base_url("https://custom-api.example.com").unwrap();
    /// ```
    pub fn with_base_url(base_url: &str) -> Result<Self> {
        let url = Url::parse(base_url)?;
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS))
            .pool_max_idle_per_host(DEFAULT_POOL_MAX_IDLE_PER_HOST)
            .pool_idle_timeout(Duration::from_secs(DEFAULT_POOL_IDLE_TIMEOUT_SECS))
            .build()
            .map_err(|e| PolymarketError::other(format!("failed to create HTTP client: {}", e)))?;
        Ok(Self {
            http_client,
            base_url: url,
        })
    }

    /// Creates a new Data API client with an existing HTTP client.
    ///
    /// # Arguments
    ///
    /// * `http_client` - An existing reqwest HTTP client.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::Client;
    /// use reqwest::Client as HttpClient;
    ///
    /// let http_client = HttpClient::builder()
    ///     .timeout(std::time::Duration::from_secs(30))
    ///     .build()
    ///     .unwrap();
    ///
    /// let client = Client::with_http_client(http_client);
    /// ```
    pub fn with_http_client(http_client: HttpClient) -> Self {
        Self {
            http_client,
            // DEFAULT_BASE_URL is known to be valid, unwrap is safe
            base_url: Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"),
        }
    }

    /// Checks if the response is successful and returns an appropriate error if not.
    ///
    /// This helper method centralizes error handling and sanitizes error messages
    /// to prevent sensitive information leakage.
    async fn check_response(&self, response: Response) -> Result<Response> {
        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status().as_u16();
        let message = response
            .text()
            .await
            .unwrap_or_default()
            .chars()
            .take(MAX_ERROR_MESSAGE_LEN)
            .collect::<String>();

        // Sanitize error message based on status code
        let error_msg = match status {
            400..=499 => format!("client error ({}): {}", status, message),
            500..=599 => format!("server error ({})", status),
            _ => format!("unexpected status ({})", status),
        };

        Err(PolymarketError::api(error_msg))
    }

    /// Builds a URL for the given path.
    ///
    /// This helper centralizes URL construction to reduce duplication.
    fn build_url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        url.set_path(path);
        url
    }

    /// Performs a health check on the Data API.
    ///
    /// This endpoint is used to verify that the API is operational.
    ///
    /// # Returns
    ///
    /// Returns a `HealthResponse` containing the health status.
    /// A successful response will have `data` set to "OK".
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::data::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let health = client.health().await?;
    ///     println!("API status: {}", health.data);
    ///     Ok(())
    /// }
    /// ```
    pub async fn health(&self) -> Result<HealthStatus> {
        let response = self.http_client.get(self.base_url.as_str()).send().await?;
        let response = self.check_response(response).await?;
        let health_response: HealthStatus = response.json().await?;
        Ok(health_response)
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

    #[test]
    fn test_client_creation() {
        let client = Client::new();
        assert!(client.base_url.as_str().starts_with(DEFAULT_BASE_URL));
    }

    #[test]
    fn test_client_with_custom_url() {
        let client = Client::with_base_url("https://custom-api.example.com/").unwrap();
        assert_eq!(client.base_url.as_str(), "https://custom-api.example.com/");
    }

    #[test]
    fn test_client_with_invalid_url() {
        let result = Client::with_base_url("not-a-valid-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_client_with_http_client() {
        let http_client = HttpClient::new();
        let client = Client::with_http_client(http_client);
        assert!(client.base_url.as_str().starts_with(DEFAULT_BASE_URL));
    }

    #[test]
    fn test_default_trait() {
        let client = Client::default();
        assert!(client.base_url.as_str().starts_with(DEFAULT_BASE_URL));
    }
}
