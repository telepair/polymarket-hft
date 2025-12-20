//! Core types for metrics and state management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// =============================================================================
// Metric Types
// =============================================================================

/// A metric data point stored in TimescaleDB.
///
/// # Example
///
/// ```rust
/// use polymarket_hft::engine::Metric;
/// use chrono::Utc;
///
/// let metric = Metric::new("cmc", "fear_and_greed_index", 75.0)
///     .with_label("classification", "Greed");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Timestamp of the metric (used as TimescaleDB hypertable partition key).
    pub time: DateTime<Utc>,
    /// Data source identifier (e.g., "cmc", "cg", "alt", "polymarket").
    pub source: String,
    /// Metric name (e.g., "fear_and_greed_index", "btc_price").
    pub name: String,
    /// Numeric value of the metric.
    pub value: f64,
    /// Additional labels for filtering and grouping.
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

impl Metric {
    /// Creates a new metric with the current timestamp.
    pub fn new(source: impl Into<String>, name: impl Into<String>, value: f64) -> Self {
        Self {
            time: Utc::now(),
            source: source.into(),
            name: name.into(),
            value,
            labels: HashMap::new(),
        }
    }

    /// Creates a new metric with a specific timestamp.
    pub fn with_time(
        time: DateTime<Utc>,
        source: impl Into<String>,
        name: impl Into<String>,
        value: f64,
    ) -> Self {
        Self {
            time,
            source: source.into(),
            name: name.into(),
            value,
            labels: HashMap::new(),
        }
    }

    /// Adds a label to the metric.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Adds multiple labels to the metric.
    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels.extend(labels);
        self
    }
}

// =============================================================================
// State Types
// =============================================================================

/// A state entry stored in Redis with optional TTL.
///
/// State entries represent the current snapshot of data that can be
/// quickly accessed for real-time queries and dashboards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry {
    /// Redis key (e.g., "state:cmc:fear_and_greed").
    pub key: String,
    /// JSON value for complex state data.
    pub value: serde_json::Value,
    /// Optional TTL override. If None, uses config default.
    #[serde(default, with = "humantime_serde")]
    pub ttl: Option<Duration>,
}

impl StateEntry {
    /// Creates a new state entry.
    pub fn new(key: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            key: key.into(),
            value,
            ttl: None,
        }
    }

    /// Sets the TTL for this state entry.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

// =============================================================================
// Conversion Traits
// =============================================================================

/// Trait for converting API responses to metrics.
///
/// Implement this trait for API response types that should be stored
/// as time-series metrics in TimescaleDB.
pub trait ToMetrics {
    /// Converts the response to a vector of metrics.
    ///
    /// # Arguments
    ///
    /// * `source` - The data source identifier (e.g., "cmc", "cg").
    fn to_metrics(&self, source: &str) -> Vec<Metric>;
}

/// Trait for converting API responses to state entries.
///
/// Implement this trait for API response types that should be cached
/// in Redis for real-time access.
pub trait ToState {
    /// Converts the response to a vector of state entries.
    ///
    /// # Arguments
    ///
    /// * `source` - The data source identifier (e.g., "cmc", "cg").
    fn to_state(&self, source: &str) -> Vec<StateEntry>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        let metric =
            Metric::new("cmc", "fear_and_greed_index", 75.0).with_label("classification", "Greed");

        assert_eq!(metric.source, "cmc");
        assert_eq!(metric.name, "fear_and_greed_index");
        assert_eq!(metric.value, 75.0);
        assert_eq!(
            metric.labels.get("classification"),
            Some(&"Greed".to_string())
        );
    }

    #[test]
    fn test_state_entry_creation() {
        let entry = StateEntry::new(
            "state:cmc:fear_and_greed",
            serde_json::json!({"value": 75, "classification": "Greed"}),
        )
        .with_ttl(Duration::from_secs(900));

        assert_eq!(entry.key, "state:cmc:fear_and_greed");
        assert_eq!(entry.ttl, Some(Duration::from_secs(900)));
    }
}
