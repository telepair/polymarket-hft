//! Shared HTTP client with retry middleware.

use std::time::Duration;

use reqwest::Client as HttpClient;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

/// Default request timeout in seconds.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default connection timeout in seconds.
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;

/// Default maximum idle connections per host.
pub const DEFAULT_POOL_MAX_IDLE_PER_HOST: usize = 10;

/// Default idle timeout in seconds.
pub const DEFAULT_POOL_IDLE_TIMEOUT_SECS: u64 = 90;

/// Default maximum retry attempts for transient failures.
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Configuration for building an HTTP client with retry middleware.
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Request timeout.
    pub timeout: Duration,
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Maximum idle connections per host.
    pub pool_max_idle_per_host: usize,
    /// Idle connection timeout.
    pub pool_idle_timeout: Duration,
    /// Maximum retry attempts for transient failures.
    pub max_retries: u32,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            pool_max_idle_per_host: DEFAULT_POOL_MAX_IDLE_PER_HOST,
            pool_idle_timeout: Duration::from_secs(DEFAULT_POOL_IDLE_TIMEOUT_SECS),
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }
}

impl HttpClientConfig {
    /// Creates a new configuration with custom max retries.
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Creates a new configuration with custom timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Creates a new configuration with custom connect timeout.
    pub fn with_connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }

    /// Builds an HTTP client with retry middleware using this configuration.
    pub fn build(self) -> Result<ClientWithMiddleware, reqwest::Error> {
        let client = HttpClient::builder()
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout)
            .pool_max_idle_per_host(self.pool_max_idle_per_host)
            .pool_idle_timeout(self.pool_idle_timeout)
            .build()?;

        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(self.max_retries);

        let client_with_middleware = ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(client_with_middleware)
    }
}

/// Builds a default HTTP client with retry middleware.
pub fn build_default_client() -> Result<ClientWithMiddleware, reqwest::Error> {
    HttpClientConfig::default().build()
}

/// Wraps an existing reqwest client with retry middleware.
pub fn wrap_with_retry(client: HttpClient, max_retries: u32) -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(max_retries);

    ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HttpClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(DEFAULT_TIMEOUT_SECS));
        assert_eq!(config.max_retries, DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn test_config_builder() {
        let config = HttpClientConfig::default()
            .with_max_retries(5)
            .with_timeout(Duration::from_secs(60));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
    )]
    #[test]
    fn test_build_default_client() {
        let result = build_default_client();
        assert!(result.is_ok());
    }

    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
    )]
    #[test]
    fn test_build_with_config() {
        let config = HttpClientConfig::default().with_max_retries(5);
        let result = config.build();
        assert!(result.is_ok());
    }
}
