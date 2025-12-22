//! Application settings configuration.

use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

use crate::client::http::HttpClientConfig;

// ============================================================================
// Default Constants
// ============================================================================

const DEFAULT_DB_PATH: &str = "data/metrics.db";
const DEFAULT_CACHE_TTL_SECS: u64 = 900; // 15 minutes
const DEFAULT_CACHE_MAX_CAPACITY: u64 = 100_000;
const DEFAULT_CLEANUP_INTERVAL_SECS: u64 = 3600; // 1 hour
const DEFAULT_METADATA_REFRESH_INTERVAL_SECS: u64 = 300; // 5 minutes
const DEFAULT_RETENTION_DAYS: u32 = 365; // 1 year
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8080;

// ============================================================================
// Application Configuration
// ============================================================================

/// Top-level application configuration loaded from YAML file.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Server configuration.
    #[serde(default)]
    pub server: ServerConfig,
    /// Data source client configurations.
    #[serde(default)]
    pub datasources: DataSourcesConfig,
    /// Optional ingestion configuration.
    #[serde(default)]
    pub ingestion: Option<IngestionConfig>,
    /// Optional storage configuration.
    #[serde(default)]
    pub storage: Option<StorageConfig>,
}

impl AppConfig {
    /// Load configuration from a YAML file.
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

// ============================================================================
// Server Configuration
// ============================================================================

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind to.
    #[serde(default)]
    pub host: String,
    /// Port to listen on.
    #[serde(default)]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
        }
    }
}

// ============================================================================
// Storage Configuration
// ============================================================================

/// Storage backend configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type.
    #[serde(default)]
    pub backend: StorageBackendType,
    /// Cleanup interval in seconds (default: 3600 = 1 hour).
    #[serde(default)]
    pub cleanup_interval_secs: u64,
    /// Metadata refresh interval in seconds (default: 300 = 5 minutes).
    #[serde(default)]
    pub metadata_refresh_interval_secs: u64,
    /// Global data retention period in days (default: 365 = 1 year).
    #[serde(default)]
    pub retention_days: u32,
    /// Local storage configuration (when backend = "local").
    #[serde(default)]
    pub local: Option<LocalStorageConfigSerde>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackendType::default(),
            cleanup_interval_secs: DEFAULT_CLEANUP_INTERVAL_SECS,
            metadata_refresh_interval_secs: DEFAULT_METADATA_REFRESH_INTERVAL_SECS,
            retention_days: DEFAULT_RETENTION_DAYS,
            local: None,
        }
    }
}

/// Storage backend type.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackendType {
    /// Local SQLite + memory cache.
    #[default]
    Local,
    /// External Redis + TimescaleDB (future).
    External,
}

/// Serde-friendly version of LocalStorageConfig.
#[derive(Debug, Clone, Deserialize)]
pub struct LocalStorageConfigSerde {
    /// Path to the SQLite database file.
    #[serde(default)]
    pub db_path: PathBuf,
    /// TTL for cached entries in seconds.
    #[serde(default)]
    pub cache_ttl_secs: u64,
    /// Maximum number of entries in the cache.
    #[serde(default)]
    pub cache_max_capacity: u64,
}

impl Default for LocalStorageConfigSerde {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from(DEFAULT_DB_PATH),
            cache_ttl_secs: DEFAULT_CACHE_TTL_SECS,
            cache_max_capacity: DEFAULT_CACHE_MAX_CAPACITY,
        }
    }
}

impl From<LocalStorageConfigSerde> for crate::LocalStorageConfig {
    fn from(serde: LocalStorageConfigSerde) -> Self {
        crate::LocalStorageConfig {
            db_path: serde.db_path,
            cache_ttl: Duration::from_secs(serde.cache_ttl_secs),
            cache_max_capacity: serde.cache_max_capacity,
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
    fn from(s: HttpClientConfigSerde) -> Self {
        let mut config = HttpClientConfig::default();

        if let Some(v) = s.timeout_secs {
            config.timeout = Duration::from_secs(v);
        }
        if let Some(v) = s.connect_timeout_secs {
            config.connect_timeout = Duration::from_secs(v);
        }
        if let Some(v) = s.max_retries {
            config.max_retries = v;
        }
        if let Some(v) = s.user_agent {
            config.user_agent = v;
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
