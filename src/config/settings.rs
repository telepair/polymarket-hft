//! Application settings configuration.

use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

use crate::client::http::HttpClientConfig;

/// Top-level application configuration loaded from YAML file.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Server configuration.
    pub server: ServerConfig,
    /// Data source client configurations.
    pub datasources: DataSourcesConfig,
    /// Optional ingestion configuration.
    #[serde(default)]
    pub ingestion: Option<IngestionConfig>,
}

impl AppConfig {
    /// Load configuration from a YAML file.
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind to.
    #[serde(default = "default_host")]
    pub host: String,
    /// Port to listen on.
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

/// Data source client configurations.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DataSourcesConfig {
    /// Common HTTP client configuration (fallback for all sources).
    #[serde(default)]
    pub common: Option<HttpClientConfigSerde>,
    /// Alternative.me client configuration.
    #[serde(default)]
    pub alternativeme: Option<HttpClientConfigSerde>,
    // Future: coingecko, coinmarketcap, polymarket
}

/// Serde-friendly version of HttpClientConfig.
#[derive(Debug, Clone, Deserialize)]
pub struct HttpClientConfigSerde {
    /// Request timeout in seconds.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    /// Connection timeout in seconds.
    #[serde(default)]
    pub connect_timeout_secs: Option<u64>,
    /// Maximum retry attempts.
    #[serde(default)]
    pub max_retries: Option<u32>,
    /// User-Agent header value.
    #[serde(default)]
    pub user_agent: Option<String>,
}

impl From<HttpClientConfigSerde> for HttpClientConfig {
    fn from(serde: HttpClientConfigSerde) -> Self {
        let mut config = HttpClientConfig::default();
        if let Some(timeout) = serde.timeout_secs {
            config = config.with_timeout(Duration::from_secs(timeout));
        }
        if let Some(connect_timeout) = serde.connect_timeout_secs {
            config = config.with_connect_timeout(Duration::from_secs(connect_timeout));
        }
        if let Some(max_retries) = serde.max_retries {
            config = config.with_max_retries(max_retries);
        }
        if let Some(user_agent) = serde.user_agent {
            config = config.with_user_agent(user_agent);
        }
        config
    }
}

/// Ingestion configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct IngestionConfig {
    /// Directory containing ingestion job YAML files.
    pub jobs_dir: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_http_client_config_serde_to_config() {
        let serde = HttpClientConfigSerde {
            timeout_secs: Some(60),
            connect_timeout_secs: Some(20),
            max_retries: Some(5),
            user_agent: Some("test-agent".to_string()),
        };
        let config: HttpClientConfig = serde.into();
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.connect_timeout, Duration::from_secs(20));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.user_agent, "test-agent");
    }
}
