//! Data API client implementation.

use reqwest::{Client as HttpClient, Response};
use reqwest_middleware::ClientWithMiddleware;
use tracing::{instrument, trace};
use url::Url;

use crate::client::http::{DEFAULT_MAX_RETRIES, HttpClientConfig, wrap_with_retry};
use crate::error::{PolymarketError, Result};

use super::HealthStatus;

/// Default base URL for the Polymarket Data API.
pub const DEFAULT_BASE_URL: &str = "https://data-api.polymarket.com";

/// Maximum error message length to prevent sensitive data leakage.
const MAX_ERROR_MESSAGE_LEN: usize = 500;

/// Client for interacting with the Polymarket Data API.
#[derive(Debug, Clone)]
pub struct Client {
    /// HTTP client with retry middleware.
    pub(super) http_client: ClientWithMiddleware,
    /// Base URL for the API (validated URL).
    pub(super) base_url: Url,
}

impl Client {
    /// Creates a new Data API client with the default base URL.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::data::Client;
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
    /// use polymarket_hft::client::polymarket::data::Client;
    ///
    /// let client = Client::with_base_url("https://custom-api.example.com").unwrap();
    /// ```
    pub fn with_base_url(base_url: &str) -> Result<Self> {
        let url = Url::parse(base_url)?;
        let http_client = HttpClientConfig::default()
            .build()
            .map_err(|e| PolymarketError::other(format!("failed to create HTTP client: {}", e)))?;
        Ok(Self {
            http_client,
            base_url: url,
        })
    }

    /// Creates a new Data API client with a custom base URL and retry configuration.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL for the API.
    /// * `max_retries` - Maximum number of retry attempts for transient failures.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Client)` if the URL is valid, or an error if parsing fails.
    pub fn with_retries(base_url: &str, max_retries: u32) -> Result<Self> {
        let url = Url::parse(base_url)?;
        let http_client = HttpClientConfig::default()
            .with_max_retries(max_retries)
            .build()
            .map_err(|e| PolymarketError::other(format!("failed to create HTTP client: {}", e)))?;
        Ok(Self {
            http_client,
            base_url: url,
        })
    }

    /// Creates a new Data API client with an existing HTTP client.
    ///
    /// The provided client will be wrapped with retry middleware using default settings.
    ///
    /// # Arguments
    ///
    /// * `http_client` - An existing reqwest HTTP client.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::data::Client;
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
            http_client: wrap_with_retry(http_client, DEFAULT_MAX_RETRIES),
            base_url: Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"),
        }
    }

    /// Creates a new Data API client with an existing middleware-enabled HTTP client.
    ///
    /// # Arguments
    ///
    /// * `http_client` - An existing reqwest-middleware HTTP client.
    pub fn with_middleware_client(http_client: ClientWithMiddleware) -> Self {
        Self {
            http_client,
            base_url: Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"),
        }
    }

    /// Checks if the response is successful and returns an appropriate error if not.
    ///
    /// This helper method centralizes error handling and sanitizes error messages
    /// to prevent sensitive information leakage.
    pub(super) async fn check_response(&self, response: Response) -> Result<Response> {
        let status = response.status();
        trace!(status = %status, "received HTTP response");

        if status.is_success() {
            return Ok(response);
        }

        let status_code = status.as_u16();
        let message = response
            .text()
            .await
            .unwrap_or_default()
            .chars()
            .take(MAX_ERROR_MESSAGE_LEN)
            .collect::<String>();

        // Sanitize error message based on status code
        let error_msg = match status_code {
            400..=499 => format!("client error ({}): {}", status_code, message),
            500..=599 => {
                if message.is_empty() {
                    format!("server error ({})", status_code)
                } else {
                    format!("server error ({}): {}", status_code, message)
                }
            }
            _ => format!("unexpected status ({})", status_code),
        };

        trace!(error = %error_msg, "HTTP request failed");
        Err(PolymarketError::api(error_msg))
    }

    /// Builds a URL for the given path, preserving any base path prefix.
    ///
    /// This avoids dropping path components when users provide a base URL like
    /// `https://example.com/api/v1`, where we still need `/api/v1/<path>`.
    pub(super) fn build_url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();

        let base_path = url.path().trim_end_matches('/');
        let suffix = path.trim_start_matches('/');

        let merged = if base_path.is_empty() {
            format!("/{}", suffix)
        } else {
            format!("{}/{}", base_path, suffix)
        };

        url.set_path(&merged);
        url
    }

    /// Performs a health check on the Data API.
    ///
    /// This endpoint is used to verify that the API is operational.
    ///
    /// # Returns
    ///
    /// Returns a `HealthStatus` containing the health status.
    /// A successful response will have `data` set to "OK".
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::data::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let health = client.health().await?;
    ///     println!("API status: {}", health.data);
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self), level = "trace")]
    pub async fn health(&self) -> Result<HealthStatus> {
        let url = self.base_url.as_str();
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let health_response: HealthStatus = response.json().await?;
        trace!(data = %health_response.data, "health check completed");
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

    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
    )]
    #[test]
    fn test_client_creation() {
        let client = Client::new();
        assert!(client.base_url.as_str().starts_with(DEFAULT_BASE_URL));
    }

    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
    )]
    #[test]
    fn test_client_with_custom_url() {
        let client = Client::with_base_url("https://custom-api.example.com/").unwrap();
        assert_eq!(client.base_url.as_str(), "https://custom-api.example.com/");
    }

    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
    )]
    #[test]
    fn test_client_with_invalid_url() {
        let result = Client::with_base_url("not-a-valid-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_url_preserves_base_path_prefix() {
        let client = Client::with_base_url("https://example.com/api/v1").unwrap();
        let url = client.build_url("trades");
        assert_eq!(url.as_str(), "https://example.com/api/v1/trades");
    }

    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
    )]
    #[test]
    fn test_client_with_http_client() {
        let http_client = HttpClient::new();
        let client = Client::with_http_client(http_client);
        assert!(client.base_url.as_str().starts_with(DEFAULT_BASE_URL));
    }

    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
    )]
    #[test]
    fn test_default_trait() {
        let client = Client::default();
        assert!(client.base_url.as_str().starts_with(DEFAULT_BASE_URL));
    }
}
