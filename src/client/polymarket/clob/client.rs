//! CLOB API client implementation.

use reqwest::{Client as HttpClient, Response};
use reqwest_middleware::ClientWithMiddleware;
use tracing::trace;
use url::Url;

use crate::client::http::{DEFAULT_MAX_RETRIES, HttpClientConfig, wrap_with_retry};
use crate::error::{PolymarketError, Result};

/// Default base URL for the Polymarket CLOB API.
pub const DEFAULT_BASE_URL: &str = "https://clob.polymarket.com";

/// Maximum error message length to prevent sensitive data leakage.
const MAX_ERROR_MESSAGE_LEN: usize = 500;

/// Client for interacting with the Polymarket CLOB API.
#[derive(Debug, Clone)]
pub struct Client {
    /// HTTP client with retry middleware.
    pub(super) http_client: ClientWithMiddleware,
    /// Base URL for the API (validated URL).
    pub(super) base_url: Url,
}

impl Client {
    /// Creates a new CLOB API client with the default base URL.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
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
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// let client = Client::with_base_url("https://custom-clob.example.com").unwrap();
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

    /// Creates a new CLOB API client with a custom base URL and retry configuration.
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

    /// Creates a new CLOB API client with an existing HTTP client.
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
    /// use polymarket_hft::client::polymarket::clob::Client;
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
            base_url: Url::parse(DEFAULT_BASE_URL).expect("default CLOB base URL is valid"),
        }
    }

    /// Creates a new CLOB API client with an existing middleware-enabled HTTP client.
    ///
    /// # Arguments
    ///
    /// * `http_client` - An existing reqwest-middleware HTTP client.
    pub fn with_middleware_client(http_client: ClientWithMiddleware) -> Self {
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

    /// Builds a URL for the given path, preserving any base path prefix.
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

    // =========================================================================
    // Server Endpoints
    // =========================================================================

    /// Health check - verifies the server is operational.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the server is healthy.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     client.get_ok().await?;
    ///     println!("Server is healthy!");
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_ok(&self) -> Result<()> {
        let url = self.build_url("");
        trace!(url = %url, method = "GET", "sending health check request");
        let response = self.http_client.get(url).send().await?;
        self.check_response(response).await?;
        trace!("server health check passed");
        Ok(())
    }

    /// Gets the current server time in Unix milliseconds.
    ///
    /// This is useful for synchronizing timestamps for order signatures.
    ///
    /// # Returns
    ///
    /// Returns the server time as Unix milliseconds.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_hft::client::polymarket::clob::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new();
    ///     let time = client.get_server_time().await?;
    ///     println!("Server time: {} ms", time);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_server_time(&self) -> Result<u64> {
        let url = self.build_url("time");
        trace!(url = %url, method = "GET", "sending server time request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;

        #[derive(serde::Deserialize)]
        struct TimeResponse {
            time: u64,
        }

        let result: TimeResponse = response.json().await?;
        trace!(time = result.time, "received server time");
        Ok(result.time)
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

    #[test]
    fn test_build_url_preserves_base_path_prefix() {
        let client = Client::with_base_url("https://example.com/api/v1").unwrap();
        let url = client.build_url("book");
        assert_eq!(url.as_str(), "https://example.com/api/v1/book");
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
