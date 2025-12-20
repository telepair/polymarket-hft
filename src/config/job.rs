//! Ingestion job configuration.

use serde::Deserialize;
use std::path::PathBuf;

use crate::DataSource;

/// Ingestion job configuration loaded from a YAML file.
#[derive(Debug, Clone, Deserialize)]
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
}

/// Schedule configuration for ingestion jobs.
#[derive(Debug, Clone, Deserialize)]
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

/// Container for multiple jobs in a single file.
#[derive(Debug, Clone, Deserialize)]
struct JobsFile {
    jobs: Vec<IngestionJob>,
}

impl IngestionJob {
    /// Load all ingestion jobs from a directory.
    ///
    /// Reads all `.yaml` and `.yml` files from the specified directory.
    /// Each file can contain either:
    /// - A single job (top-level job definition)
    /// - Multiple jobs (under `jobs:` key)
    pub fn load_from_dir(dir: &PathBuf) -> anyhow::Result<Vec<Self>> {
        let mut jobs = Vec::new();

        if !dir.exists() {
            return Ok(jobs);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
            {
                let content = std::fs::read_to_string(&path)?;

                // Try parsing as multiple jobs first
                if let Ok(jobs_file) = serde_yaml::from_str::<JobsFile>(&content) {
                    jobs.extend(jobs_file.jobs);
                } else {
                    // Fall back to single job
                    let job: IngestionJob = serde_yaml::from_str(&content)?;
                    jobs.push(job);
                }
            }
        }

        Ok(jobs)
    }
}

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
    fn test_parse_multiple_jobs() {
        let yaml = r#"
jobs:
  - name: job1
    datasource: alternativeme
    method: get_fear_and_greed
    interval_secs: 300
  - name: job2
    datasource: alternativeme
    method: get_global
    interval_secs: 600
"#;
        let jobs_file: JobsFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(jobs_file.jobs.len(), 2);
        assert_eq!(jobs_file.jobs[0].name, "job1");
        assert_eq!(jobs_file.jobs[1].name, "job2");
    }
}
