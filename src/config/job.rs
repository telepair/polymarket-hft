//! Ingestion job configuration.

use serde::{Deserialize, Serialize};

use crate::DataSource;

/// Minimum interval in seconds for interval-based scheduling.
const MIN_INTERVAL_SECS: u64 = 10;

/// Ingestion job configuration.
///
/// Jobs are managed via web UI and stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionJob {
    /// Unique name for this job.
    pub name: String,
    /// Data source to fetch from.
    pub datasource: DataSource,
    /// Method name to invoke on the data source client.
    pub method: String,
    /// Schedule for this job.
    #[serde(flatten)]
    pub schedule: Schedule,
    /// Optional parameters to pass to the method.
    #[serde(default)]
    pub params: Option<serde_json::Value>,
    /// Data retention period in days (default: 7).
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    /// Whether this job is enabled (default: true).
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl IngestionJob {
    /// Validate the job configuration.
    ///
    /// Returns an error if:
    /// - The schedule is invalid (see `Schedule::validate`)
    /// - The name is empty
    /// - The method is empty
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.trim().is_empty() {
            anyhow::bail!("Job name cannot be empty");
        }
        if self.method.trim().is_empty() {
            anyhow::bail!("Job method cannot be empty");
        }
        self.schedule.validate()?;
        Ok(())
    }
}

fn default_retention_days() -> u32 {
    7
}

fn default_enabled() -> bool {
    true
}

/// Schedule configuration for ingestion jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Schedule {
    /// Interval-based scheduling.
    Interval {
        /// Interval in seconds between job executions.
        interval_secs: u64,
    },
    /// Cron-based scheduling.
    Cron {
        /// Cron expression (e.g., "0 * * * * *" for every minute).
        cron: String,
    },
}

impl Schedule {
    /// Create a new cron-based schedule with validation.
    ///
    /// The cron expression is normalized to 6-field format if needed.
    pub fn new_cron(cron_expr: &str) -> anyhow::Result<Self> {
        let normalized = Self::normalize_cron(cron_expr);
        // Validate by attempting to parse
        cron::Schedule::from_str(&normalized)
            .map_err(|e| anyhow::anyhow!("Invalid cron expression '{}': {}", cron_expr, e))?;
        Ok(Schedule::Cron {
            cron: cron_expr.to_string(),
        })
    }

    /// Create a new interval-based schedule with validation.
    pub fn new_interval(interval_secs: u64) -> anyhow::Result<Self> {
        if interval_secs < MIN_INTERVAL_SECS {
            anyhow::bail!(
                "Interval must be at least {} seconds, got {}",
                MIN_INTERVAL_SECS,
                interval_secs
            );
        }
        Ok(Schedule::Interval { interval_secs })
    }

    /// Validate the schedule configuration.
    pub fn validate(&self) -> anyhow::Result<()> {
        match self {
            Schedule::Interval { interval_secs } => {
                if *interval_secs < MIN_INTERVAL_SECS {
                    anyhow::bail!(
                        "Interval must be at least {} seconds, got {}",
                        MIN_INTERVAL_SECS,
                        interval_secs
                    );
                }
                Ok(())
            }
            Schedule::Cron { cron } => {
                let normalized = Self::normalize_cron(cron);
                cron::Schedule::from_str(&normalized)
                    .map_err(|e| anyhow::anyhow!("Invalid cron expression '{}': {}", cron, e))?;
                Ok(())
            }
        }
    }

    /// Normalize cron expression to 6-field format.
    fn normalize_cron(cron: &str) -> String {
        let fields: Vec<&str> = cron.split_whitespace().collect();
        if fields.len() == 5 {
            format!("0 {}", cron)
        } else {
            cron.to_string()
        }
    }
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interval_schedule() {
        let yaml = r#"
name: test_job
datasource: alternativeme
method: get_fear_and_greed
interval_secs: 3600
"#;
        let job: IngestionJob = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(job.name, "test_job");
        assert_eq!(job.datasource, DataSource::AlternativeMe);
        assert_eq!(job.method, "get_fear_and_greed");
        match job.schedule {
            Schedule::Interval { interval_secs } => assert_eq!(interval_secs, 3600),
            _ => panic!("Expected Interval schedule"),
        }
    }

    #[test]
    fn test_parse_cron_schedule() {
        let yaml = r#"
name: hourly_global
datasource: alternativeme
method: get_global
cron: "0 * * * * *"
"#;
        let job: IngestionJob = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(job.name, "hourly_global");
        match job.schedule {
            Schedule::Cron { cron } => assert_eq!(cron, "0 * * * * *"),
            _ => panic!("Expected Cron schedule"),
        }
    }

    #[test]
    fn test_parse_with_params() {
        let yaml = r#"
name: ticker_job
datasource: alternativeme
method: list_ticker
interval_secs: 300
params:
  limit: 10
  start: 1
"#;
        let job: IngestionJob = serde_yaml::from_str(yaml).unwrap();
        assert!(job.params.is_some());
        let params = job.params.unwrap();
        assert_eq!(params.get("limit").unwrap().as_i64(), Some(10));
        assert_eq!(params.get("start").unwrap().as_i64(), Some(1));
    }

    #[test]
    fn test_defaults() {
        let yaml = r#"
name: test_job
datasource: alternativeme
method: get_fear_and_greed
interval_secs: 3600
"#;
        let job: IngestionJob = serde_yaml::from_str(yaml).unwrap();
        assert!(job.enabled); // default is true
        assert_eq!(job.retention_days, 7); // default is 7
    }

    #[test]
    fn test_enabled_explicit() {
        let yaml = r#"
name: test_job
datasource: alternativeme
method: get_fear_and_greed
interval_secs: 3600
enabled: false
"#;
        let job: IngestionJob = serde_yaml::from_str(yaml).unwrap();
        assert!(!job.enabled);
    }

    #[test]
    fn test_schedule_new_cron_valid() {
        let schedule = Schedule::new_cron("0 * * * * *").unwrap();
        match schedule {
            Schedule::Cron { cron } => assert_eq!(cron, "0 * * * * *"),
            _ => panic!("Expected Cron schedule"),
        }
    }

    #[test]
    fn test_schedule_new_cron_5field() {
        // 5-field cron should be normalized to 6-field
        let schedule = Schedule::new_cron("* * * * *").unwrap();
        match schedule {
            Schedule::Cron { cron } => assert_eq!(cron, "* * * * *"),
            _ => panic!("Expected Cron schedule"),
        }
    }

    #[test]
    fn test_schedule_new_cron_invalid() {
        let result = Schedule::new_cron("invalid cron");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid cron"));
    }

    #[test]
    fn test_schedule_new_interval_valid() {
        let schedule = Schedule::new_interval(60).unwrap();
        match schedule {
            Schedule::Interval { interval_secs } => assert_eq!(interval_secs, 60),
            _ => panic!("Expected Interval schedule"),
        }
    }

    #[test]
    fn test_schedule_new_interval_too_small() {
        let result = Schedule::new_interval(5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least"));
    }

    #[test]
    fn test_job_validate_valid() {
        let job = IngestionJob {
            name: "test".to_string(),
            datasource: DataSource::AlternativeMe,
            method: "get_fear_and_greed".to_string(),
            schedule: Schedule::Interval { interval_secs: 60 },
            params: None,
            retention_days: 7,
            enabled: true,
        };
        assert!(job.validate().is_ok());
    }

    #[test]
    fn test_job_validate_empty_name() {
        let job = IngestionJob {
            name: "  ".to_string(),
            datasource: DataSource::AlternativeMe,
            method: "get_fear_and_greed".to_string(),
            schedule: Schedule::Interval { interval_secs: 60 },
            params: None,
            retention_days: 7,
            enabled: true,
        };
        let result = job.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_job_validate_invalid_cron() {
        let job = IngestionJob {
            name: "test".to_string(),
            datasource: DataSource::AlternativeMe,
            method: "get_fear_and_greed".to_string(),
            schedule: Schedule::Cron {
                cron: "not valid".to_string(),
            },
            params: None,
            retention_days: 7,
            enabled: true,
        };
        let result = job.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cron"));
    }
}
