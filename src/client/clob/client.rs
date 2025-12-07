//! CLOB API client implementation.

use std::time::Duration;

use reqwest::{Client as HttpClient, Response};
use tracing::trace;
use url::Url;

use crate::error::{PolymarketError, Result};

/// Default base URL for the Polymarket CLOB API.
pub const DEFAULT_BASE_URL: &str = "https://clob.polymarket.com";

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

/// Client for interacting with the Polymarket CLOB API.
#[derive(Debug, Clone)]
pub struct Client {
    /// HTTP client for making requests.
    pub(super) http_client: HttpClient,
    /// Base URL for the API (validated URL).
    pub(super) base_url: Url,
}

impl Client {
    /// Creates a new CLOB API client with the default base URL.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::clob::Client;
    ///
    /// let client = Client::new();
    /// ```
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL).expect("default CLOB base URL is valid")
    }

    /// Creates a new CLOB API client with a custom base URL.
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
    /// use polymarket_hft::client::clob::Client;
    ///
    /// let client = Client::with_base_url("https://custom-clob.example.com").unwrap();
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

    /// Creates a new CLOB API client with an existing HTTP client.
    ///
    /// # Arguments
    ///
    /// * `http_client` - An existing reqwest HTTP client.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::clob::Client;
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
            base_url: Url::parse(DEFAULT_BASE_URL).expect("default CLOB base URL is valid"),
        }
    }

    /// Checks if the response is successful and returns an appropriate error if not.
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

    /// Builds a URL for the given path.
    pub(super) fn build_url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        url.set_path(path);
        url
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
        let client = Client::with_base_url("https://example.com/").unwrap();
        assert_eq!(client.base_url.as_str(), "https://example.com/");
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
