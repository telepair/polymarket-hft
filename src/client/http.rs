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

/// Default minimum retry interval in milliseconds.
pub const DEFAULT_MIN_RETRY_INTERVAL_MS: u64 = 100;

/// Default maximum retry interval in milliseconds.
pub const DEFAULT_MAX_RETRY_INTERVAL_MS: u64 = 30_000;

/// Default User-Agent header value.
pub const DEFAULT_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Helper macro to add optional query parameters to a request.
///
/// This macro takes a request builder and a list of (key, value) tuples,
/// where values are `Option<T>`. Only parameters with `Some` values are added.
///
/// # Example
///
/// ```ignore
/// let req = add_query_params!(
///     request_builder,
///     ("limit", Some(10)),
/// );
/// ```
#[macro_export]
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
    /// Minimum interval between retries (initial backoff).
    pub min_retry_interval: Duration,
    /// Maximum interval between retries (backoff cap).
    pub max_retry_interval: Duration,
    /// User-Agent header value.
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            pool_max_idle_per_host: DEFAULT_POOL_MAX_IDLE_PER_HOST,
            pool_idle_timeout: Duration::from_secs(DEFAULT_POOL_IDLE_TIMEOUT_SECS),
            max_retries: DEFAULT_MAX_RETRIES,
            min_retry_interval: Duration::from_millis(DEFAULT_MIN_RETRY_INTERVAL_MS),
            max_retry_interval: Duration::from_millis(DEFAULT_MAX_RETRY_INTERVAL_MS),
            user_agent: DEFAULT_USER_AGENT.to_string(),
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

    /// Creates a new configuration with custom pool max idle connections per host.
    pub fn with_pool_max_idle_per_host(mut self, max_idle: usize) -> Self {
        self.pool_max_idle_per_host = max_idle;
        self
    }

    /// Creates a new configuration with custom pool idle timeout.
    pub fn with_pool_idle_timeout(mut self, idle_timeout: Duration) -> Self {
        self.pool_idle_timeout = idle_timeout;
        self
    }

    /// Creates a new configuration with custom minimum retry interval.
    ///
    /// This is the initial backoff duration before the first retry.
    pub fn with_min_retry_interval(mut self, interval: Duration) -> Self {
        self.min_retry_interval = interval;
        self
    }

    /// Creates a new configuration with custom maximum retry interval.
    ///
    /// This caps the exponential backoff to prevent excessively long waits.
    pub fn with_max_retry_interval(mut self, interval: Duration) -> Self {
        self.max_retry_interval = interval;
        self
    }

    /// Creates a new configuration with custom User-Agent header.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Builds an HTTP client with retry middleware using this configuration.
    pub fn build(self) -> Result<ClientWithMiddleware, reqwest::Error> {
        let client = HttpClient::builder()
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout)
            .pool_max_idle_per_host(self.pool_max_idle_per_host)
            .pool_idle_timeout(self.pool_idle_timeout)
            .user_agent(&self.user_agent)
            .build()?;

        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(self.min_retry_interval, self.max_retry_interval)
            .build_with_max_retries(self.max_retries);

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
///
/// This function is infallible because it only wraps an already-constructed client
/// with middleware. The underlying client has already been validated during its creation.
///
/// Uses default exponential backoff intervals. For custom retry intervals,
/// use [`HttpClientConfig`] instead.
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
        assert_eq!(
            config.connect_timeout,
            Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS)
        );
        assert_eq!(
            config.pool_max_idle_per_host,
            DEFAULT_POOL_MAX_IDLE_PER_HOST
        );
        assert_eq!(
            config.pool_idle_timeout,
            Duration::from_secs(DEFAULT_POOL_IDLE_TIMEOUT_SECS)
        );
        assert_eq!(config.max_retries, DEFAULT_MAX_RETRIES);
        assert_eq!(
            config.min_retry_interval,
            Duration::from_millis(DEFAULT_MIN_RETRY_INTERVAL_MS)
        );
        assert_eq!(
            config.max_retry_interval,
            Duration::from_millis(DEFAULT_MAX_RETRY_INTERVAL_MS)
        );
        assert_eq!(config.user_agent, DEFAULT_USER_AGENT);
    }

    #[test]
    fn test_config_builder() {
        let config = HttpClientConfig::default()
            .with_max_retries(5)
            .with_timeout(Duration::from_secs(60));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_pool_config_builder() {
        let config = HttpClientConfig::default()
            .with_pool_max_idle_per_host(20)
            .with_pool_idle_timeout(Duration::from_secs(120));
        assert_eq!(config.pool_max_idle_per_host, 20);
        assert_eq!(config.pool_idle_timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_retry_interval_builder() {
        let config = HttpClientConfig::default()
            .with_min_retry_interval(Duration::from_millis(200))
            .with_max_retry_interval(Duration::from_secs(60));
        assert_eq!(config.min_retry_interval, Duration::from_millis(200));
        assert_eq!(config.max_retry_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_user_agent_builder() {
        let config = HttpClientConfig::default().with_user_agent("custom-agent/1.0");
        assert_eq!(config.user_agent, "custom-agent/1.0");
    }

    #[test]
    fn test_builder_chaining() {
        let config = HttpClientConfig::default()
            .with_timeout(Duration::from_secs(45))
            .with_connect_timeout(Duration::from_secs(15))
            .with_pool_max_idle_per_host(5)
            .with_pool_idle_timeout(Duration::from_secs(60))
            .with_max_retries(5)
            .with_min_retry_interval(Duration::from_millis(50))
            .with_max_retry_interval(Duration::from_secs(10))
            .with_user_agent("test-agent");

        assert_eq!(config.timeout, Duration::from_secs(45));
        assert_eq!(config.connect_timeout, Duration::from_secs(15));
        assert_eq!(config.pool_max_idle_per_host, 5);
        assert_eq!(config.pool_idle_timeout, Duration::from_secs(60));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.min_retry_interval, Duration::from_millis(50));
        assert_eq!(config.max_retry_interval, Duration::from_secs(10));
        assert_eq!(config.user_agent, "test-agent");
    }

    #[test]
    fn test_connect_timeout_builder() {
        let config = HttpClientConfig::default().with_connect_timeout(Duration::from_secs(20));
        assert_eq!(config.connect_timeout, Duration::from_secs(20));
    }

    #[test]
    fn test_config_clone() {
        let config = HttpClientConfig::default()
            .with_timeout(Duration::from_secs(45))
            .with_user_agent("clone-test");

        let cloned = config.clone();

        assert_eq!(cloned.timeout, Duration::from_secs(45));
        assert_eq!(cloned.user_agent, "clone-test");
        // Ensure original is unchanged
        assert_eq!(config.timeout, Duration::from_secs(45));
        assert_eq!(config.user_agent, "clone-test");
    }

    #[test]
    fn test_zero_max_retries() {
        let config = HttpClientConfig::default().with_max_retries(0);
        assert_eq!(config.max_retries, 0);
    }

    #[test]
    fn test_zero_timeout() {
        let config = HttpClientConfig::default().with_timeout(Duration::ZERO);
        assert_eq!(config.timeout, Duration::ZERO);
    }

    #[test]
    fn test_empty_user_agent() {
        let config = HttpClientConfig::default().with_user_agent("");
        assert_eq!(config.user_agent, "");
    }

    #[test]
    fn test_user_agent_from_string() {
        // Test that with_user_agent accepts both &str and String
        let config = HttpClientConfig::default().with_user_agent(String::from("string-agent"));
        assert_eq!(config.user_agent, "string-agent");
    }

    #[test]
    fn test_large_pool_values() {
        let config = HttpClientConfig::default()
            .with_pool_max_idle_per_host(1000)
            .with_pool_idle_timeout(Duration::from_secs(3600));
        assert_eq!(config.pool_max_idle_per_host, 1000);
        assert_eq!(config.pool_idle_timeout, Duration::from_secs(3600));
    }

    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
    )]
    #[test]
    fn test_wrap_with_retry() {
        let client = HttpClient::builder().build().unwrap();
        let _wrapped = wrap_with_retry(client, 5);
        // wrap_with_retry is infallible, so if we get here, it succeeded
    }

    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
    )]
    #[test]
    fn test_wrap_with_zero_retries() {
        let client = HttpClient::builder().build().unwrap();
        let _wrapped = wrap_with_retry(client, 0);
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

    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
    )]
    #[test]
    fn test_build_with_all_custom_config() {
        let config = HttpClientConfig::default()
            .with_timeout(Duration::from_secs(60))
            .with_connect_timeout(Duration::from_secs(20))
            .with_pool_max_idle_per_host(20)
            .with_pool_idle_timeout(Duration::from_secs(120))
            .with_max_retries(5)
            .with_min_retry_interval(Duration::from_millis(200))
            .with_max_retry_interval(Duration::from_secs(60))
            .with_user_agent("full-test/1.0");
        let result = config.build();
        assert!(result.is_ok());
    }
}
