//! Storage data models.
//!
//! This module defines core data structures for the storage layer,
//! including metrics and states that can be collected from various data sources.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// DataSource
// =============================================================================

/// Data source identifier for metrics and states.
///
/// This enum represents all available data sources that can provide metrics and states.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DataSource {
    /// Alternative.me API (Fear & Greed Index, etc.)
    AlternativeMe,
    /// CoinGecko API (cryptocurrency prices, market data)
    CoinGecko,
    /// CoinMarketCap API (cryptocurrency prices, market data)
    CoinMarketCap,
    /// Polymarket API (prediction markets)
    Polymarket,
    /// Custom data source.
    Custom(String),
}

impl std::fmt::Display for DataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSource::AlternativeMe => write!(f, "alternativeme"),
            DataSource::CoinGecko => write!(f, "coingecko"),
            DataSource::CoinMarketCap => write!(f, "coinmarketcap"),
            DataSource::Polymarket => write!(f, "polymarket"),
            DataSource::Custom(source) => write!(f, "custom::{}", source.to_lowercase()),
        }
    }
}

impl std::str::FromStr for DataSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alternativeme" => Ok(DataSource::AlternativeMe),
            "coingecko" => Ok(DataSource::CoinGecko),
            "coinmarketcap" => Ok(DataSource::CoinMarketCap),
            "polymarket" => Ok(DataSource::Polymarket),
            s if s.starts_with("custom::") => {
                let name = s.strip_prefix("custom::").unwrap_or("");
                Ok(DataSource::Custom(name.to_string()))
            }
            _ => anyhow::bail!("Unknown data source: {}", s),
        }
    }
}

// =============================================================================
// Metric
// =============================================================================

/// A metric represents a numeric measurement collected from a data source.
///
/// Metrics are typically time-series data points that can be stored in TimescaleDB
/// for historical analysis and aggregation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metric {
    /// Data source identifier.
    pub source: DataSource,

    /// Unique identifier for the metric (e.g., "fear_and_greed_index").
    pub name: String,

    /// Numeric value of the metric.
    pub value: f64,

    /// Unix timestamp when the metric was collected.
    pub timestamp: i64,

    /// Optional labels for metric categorization and filtering.
    #[serde(default)]
    pub labels: std::collections::HashMap<String, String>,
}

impl Metric {
    /// Creates a new metric with the given parameters.
    /// The timestamp defaults to the current time (now).
    pub fn new(source: DataSource, name: impl Into<String>, value: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Self {
            name: name.into(),
            value,
            timestamp,
            labels: std::collections::HashMap::new(),
            source,
        }
    }

    /// Sets a custom timestamp for the metric.
    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Adds a label to the metric.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Returns the state key in the format "source::name".
    pub fn state_key(&self) -> String {
        format!("{}::{}", self.source, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_new() {
        let metric =
            Metric::new(DataSource::AlternativeMe, "test_metric", 42.0).with_timestamp(1234567890);
        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.value, 42.0);
        assert_eq!(metric.timestamp, 1234567890);
        assert_eq!(metric.source, DataSource::AlternativeMe);
        assert!(metric.labels.is_empty());
    }

    #[test]
    fn test_metric_with_label() {
        let metric = Metric::new(DataSource::Polymarket, "test_metric", 42.0)
            .with_timestamp(1234567890)
            .with_label("env", "production")
            .with_label("region", "us-west");
        assert_eq!(metric.labels.len(), 2);
        assert_eq!(metric.labels.get("env"), Some(&"production".to_string()));
        assert_eq!(metric.labels.get("region"), Some(&"us-west".to_string()));
    }

    #[test]
    fn test_data_source_display() {
        assert_eq!(DataSource::AlternativeMe.to_string(), "alternativeme");
        assert_eq!(DataSource::CoinGecko.to_string(), "coingecko");
        assert_eq!(DataSource::CoinMarketCap.to_string(), "coinmarketcap");
        assert_eq!(DataSource::Polymarket.to_string(), "polymarket");
        assert_eq!(
            DataSource::Custom("custom".to_string()).to_string(),
            "custom::custom".to_string()
        );
    }

    #[test]
    fn test_metric_state_key() {
        let metric = Metric::new(DataSource::AlternativeMe, "fear_and_greed_index", 75.0);
        assert_eq!(metric.state_key(), "alternativeme::fear_and_greed_index");

        let metric = Metric::new(
            DataSource::Custom("binance".to_string()),
            "btc_price",
            100000.0,
        );
        assert_eq!(metric.state_key(), "custom::binance::btc_price");
    }
}
