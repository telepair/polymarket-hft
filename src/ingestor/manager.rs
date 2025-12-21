//! Ingestor manager for scheduling and running data collection tasks.

use std::sync::Arc;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::client::DataSourceClient;
use crate::config::{IngestionJob, Schedule};
use crate::storage::StorageBackend;

/// Manages ingestion jobs and coordinates data collection.
pub struct IngestorManager {
    jobs: Vec<IngestionJob>,
    client: Arc<dyn DataSourceClient>,
    storage: Arc<dyn StorageBackend>,
}

impl IngestorManager {
    /// Creates a new IngestorManager.
    pub fn new(
        jobs: Vec<IngestionJob>,
        client: Arc<dyn DataSourceClient>,
        storage: Arc<dyn StorageBackend>,
    ) -> Self {
        Self {
            jobs,
            client,
            storage,
        }
    }

    /// Runs all ingestion jobs until shutdown signal is received.
    pub async fn run(&self) -> anyhow::Result<()> {
        if self.jobs.is_empty() {
            tracing::info!("No ingestion jobs configured");
            return Ok(());
        }

        tracing::info!("Starting {} ingestion job(s)", self.jobs.len());

        let mut scheduler = JobScheduler::new().await?;

        for job_config in &self.jobs {
            let job = self.create_job(job_config).await?;
            scheduler.add(job).await?;
            tracing::info!(
                name = %job_config.name,
                method = %job_config.method,
                "Job scheduled"
            );
        }

        scheduler.start().await?;

        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        tracing::info!("Shutdown signal received, stopping scheduler");

        scheduler.shutdown().await?;

        Ok(())
    }

    async fn create_job(&self, job_config: &IngestionJob) -> anyhow::Result<Job> {
        let client = Arc::clone(&self.client);
        let storage = Arc::clone(&self.storage);
        let job_name = job_config.name.clone();
        let method = job_config.method.clone();
        let params = job_config.params.clone();

        let job = match &job_config.schedule {
            Schedule::Interval { interval_secs } => {
                let duration = Duration::from_secs(*interval_secs);
                Job::new_repeated_async(duration, move |_uuid, _lock| {
                    let client = Arc::clone(&client);
                    let storage = Arc::clone(&storage);
                    let job_name = job_name.clone();
                    let method = method.clone();
                    let params = params.clone();
                    Box::pin(async move {
                        Self::execute_job(&job_name, &method, params, &client, &storage).await;
                    })
                })?
            }
            Schedule::Cron { cron } => {
                // Normalize cron: convert 5-field (standard) to 6-field (with seconds)
                let cron_expr = Self::normalize_cron(cron);
                Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
                let client = Arc::clone(&client);
                let storage = Arc::clone(&storage);
                let job_name = job_name.clone();
                let method = method.clone();
                let params = params.clone();
                Box::pin(async move {
                    Self::execute_job(&job_name, &method, params, &client, &storage).await;
                })
            })? }
        };

        Ok(job)
    }

    /// Normalize cron expression: convert 5-field (standard) to 6-field (with seconds).
    ///
    /// tokio_cron_scheduler requires 6-field cron: `sec min hour day month weekday`
    /// Standard cron uses 5 fields: `min hour day month weekday`
    fn normalize_cron(cron: &str) -> String {
        let fields: Vec<&str> = cron.split_whitespace().collect();
        if fields.len() == 5 {
            // Standard 5-field cron, prepend "0" for seconds
            format!("0 {}", cron)
        } else {
            // Already 6-field or other format, use as-is
            cron.to_string()
        }
    }

    async fn execute_job(
        job_name: &str,
        method: &str,
        params: Option<serde_json::Value>,
        client: &Arc<dyn DataSourceClient>,
        storage: &Arc<dyn StorageBackend>,
    ) {
        tracing::debug!(job = %job_name, method = %method, "Executing job");

        match client.fetch(method, params).await {
            Ok(metrics) => {
                tracing::debug!(
                    job = %job_name,
                    count = metrics.len(),
                    "Fetched metrics"
                );
                if let Err(e) = storage.store(&metrics).await {
                    tracing::error!(
                        job = %job_name,
                        error = %e,
                        "Failed to store metrics"
                    );
                }
            }
            Err(e) => {
                tracing::error!(
                    job = %job_name,
                    error = %e,
                    "Failed to fetch metrics"
                );
            }
        }
    }
}
