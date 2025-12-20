//! Scrape configuration for scheduled data collection.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Root configuration for scrape jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeConfig {
    /// List of scrape jobs.
    pub scrape_jobs: Vec<ScrapeJob>,
}

impl ScrapeConfig {
    /// Loads configuration from a YAML file.
    pub fn from_yaml_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }

    /// Parses configuration from a YAML string.
    pub fn from_yaml(content: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(content).map_err(ConfigError::Yaml)
    }

    /// Loads configuration from a JSON file.
    pub fn from_json_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Parses configuration from a JSON string.
    pub fn from_json(content: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(content).map_err(ConfigError::Json)
    }

    /// Loads configuration from a file, auto-detecting format by extension.
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        if path.ends_with(".yaml") || path.ends_with(".yml") {
            Self::from_yaml_file(path)
        } else if path.ends_with(".json") {
            Self::from_json_file(path)
        } else {
            Err(ConfigError::UnsupportedFormat(path.to_string()))
        }
    }
}

/// A single scrape job configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeJob {
    /// Unique identifier for this job.
    pub id: String,
    /// Data source client name (e.g., "alternativeme", "coinmarketcap").
    pub source: String,
    /// API endpoint/method name to call.
    pub endpoint: String,
    /// Parameters to pass to the API call.
    #[serde(default)]
    pub params: serde_json::Value,
    /// Target destinations for the scraped data.
    pub targets: Vec<Target>,
    /// Schedule configuration.
    pub schedule: Schedule,
    /// TTL for state entries (overrides global default).
    #[serde(default, with = "humantime_serde")]
    pub state_ttl: Option<Duration>,
    /// Whether this job is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Target destination for scraped data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    /// Store as time-series metrics in TimescaleDB.
    Metrics,
    /// Cache as state in Redis.
    State,
}

/// Schedule configuration for a scrape job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Schedule {
    /// Run at fixed intervals.
    Interval {
        /// Interval between runs.
        #[serde(with = "humantime_serde")]
        interval: Duration,
    },
    /// Run on a cron schedule.
    Cron {
        /// Cron expression (e.g., "0 * * * *" for hourly).
        expression: String,
    },
}

/// Configuration error types.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unsupported config format: {0}")]
    UnsupportedFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_config() {
        let yaml = r#"
scrape_jobs:
  - id: alt_fear_and_greed
    source: alternativeme
    endpoint: get_fear_and_greed
    params: {}
    targets: [metrics, state]
    schedule:
      type: interval
      interval: 5m
    state_ttl: 15m
"#;

        let config = ScrapeConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.scrape_jobs.len(), 1);

        let job = &config.scrape_jobs[0];
        assert_eq!(job.id, "alt_fear_and_greed");
        assert_eq!(job.source, "alternativeme");
        assert_eq!(job.targets, vec![Target::Metrics, Target::State]);

        match &job.schedule {
            Schedule::Interval { interval } => {
                assert_eq!(*interval, Duration::from_secs(300));
            }
            _ => panic!("Expected interval schedule"),
        }

        assert_eq!(job.state_ttl, Some(Duration::from_secs(900)));
    }

    #[test]
    fn test_parse_cron_schedule() {
        let yaml = r#"
scrape_jobs:
  - id: hourly_job
    source: coinmarketcap
    endpoint: get_global_metrics
    targets: [metrics]
    schedule:
      type: cron
      expression: "0 * * * *"
"#;

        let config = ScrapeConfig::from_yaml(yaml).unwrap();
        let job = &config.scrape_jobs[0];

        match &job.schedule {
            Schedule::Cron { expression } => {
                assert_eq!(expression, "0 * * * *");
            }
            _ => panic!("Expected cron schedule"),
        }
    }
}
