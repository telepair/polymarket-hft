//! Task manager for scheduling and running all background tasks.
//!
//! Manages:
//! - Data ingestion jobs (configured via YAML)
//! - System maintenance tasks (data cleanup)
//! - Metadata refresh tasks

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::client::DataSourceClient;
use crate::config::{IngestionJob, Schedule, StorageConfig};
use crate::storage::StorageBackend;

/// Manages all background tasks and coordinates data collection.
pub struct TaskManager {
    jobs: Vec<IngestionJob>,
    client: Arc<dyn DataSourceClient>,
    storage: Arc<dyn StorageBackend>,
    metadata_cache: Arc<RwLock<Vec<(String, String)>>>,
    config: StorageConfig,
}

impl TaskManager {
    /// Creates a new TaskManager.
    pub fn new(
        jobs: Vec<IngestionJob>,
        client: Arc<dyn DataSourceClient>,
        storage: Arc<dyn StorageBackend>,
        metadata_cache: Arc<RwLock<Vec<(String, String)>>>,
        config: StorageConfig,
    ) -> Self {
        Self {
            jobs,
            client,
            storage,
            metadata_cache,
            config,
        }
    }

    /// Runs all tasks (ingestion and system) until shutdown signal is received.
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("Starting task manager");

        let mut scheduler = JobScheduler::new().await?;

        // 1. Schedule Ingestion Jobs
        for job_config in &self.jobs {
            let job = self.create_ingestion_job(job_config).await?;
            scheduler.add(job).await?;
            tracing::info!(
                name = %job_config.name,
                method = %job_config.method,
                "Ingestion job scheduled"
            );
        }

        // 2. Schedule Cleanup Task
        if self.config.cleanup_interval_secs > 0 {
            let cleanup_job = self.create_cleanup_job().await?;
            scheduler.add(cleanup_job).await?;
            tracing::info!(
                interval_secs = self.config.cleanup_interval_secs,
                "Cleanup task scheduled"
            );
        }

        // 3. Schedule Metadata Refresh Task
        if self.config.metadata_refresh_interval_secs > 0 {
            let refresh_job = self.create_metadata_refresh_job().await?;
            scheduler.add(refresh_job).await?;
            tracing::info!(
                interval_secs = self.config.metadata_refresh_interval_secs,
                "Metadata refresh task scheduled"
            );

            // Run initial refresh immediately
            self.refresh_metadata().await;
        }

        scheduler.start().await?;

        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        tracing::info!("Shutdown signal received, stopping scheduler");

        scheduler.shutdown().await?;

        Ok(())
    }

    async fn create_ingestion_job(&self, job_config: &IngestionJob) -> anyhow::Result<Job> {
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
                        Self::execute_ingestion_job(&job_name, &method, params, &client, &storage)
                            .await;
                    })
                })?
            }
            Schedule::Cron { cron } => {
                let cron_expr = Self::normalize_cron(cron);
                Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
                    let client = Arc::clone(&client);
                    let storage = Arc::clone(&storage);
                    let job_name = job_name.clone();
                    let method = method.clone();
                    let params = params.clone();
                    Box::pin(async move {
                        Self::execute_ingestion_job(&job_name, &method, params, &client, &storage)
                            .await;
                    })
                })?
            }
        };

        Ok(job)
    }

    async fn create_cleanup_job(&self) -> anyhow::Result<Job> {
        let storage = Arc::clone(&self.storage);
        let retention_days = self.config.retention_days;
        let duration = Duration::from_secs(self.config.cleanup_interval_secs);

        Job::new_repeated_async(duration, move |_uuid, _lock| {
            let storage = Arc::clone(&storage);
            Box::pin(async move {
                Self::execute_cleanup_task(&storage, retention_days).await;
            })
        })
        .map_err(Into::into)
    }

    async fn create_metadata_refresh_job(&self) -> anyhow::Result<Job> {
        let storage = Arc::clone(&self.storage);
        let cache = Arc::clone(&self.metadata_cache);
        let duration = Duration::from_secs(self.config.metadata_refresh_interval_secs);

        Job::new_repeated_async(duration, move |_uuid, _lock| {
            let storage = Arc::clone(&storage);
            let cache = Arc::clone(&cache);
            Box::pin(async move {
                Self::execute_metadata_refresh_task(&storage, &cache).await;
            })
        })
        .map_err(Into::into)
    }

    fn normalize_cron(cron: &str) -> String {
        let fields: Vec<&str> = cron.split_whitespace().collect();
        if fields.len() == 5 {
            format!("0 {}", cron)
        } else {
            cron.to_string()
        }
    }

    async fn execute_ingestion_job(
        job_name: &str,
        method: &str,
        params: Option<serde_json::Value>,
        client: &Arc<dyn DataSourceClient>,
        storage: &Arc<dyn StorageBackend>,
    ) {
        tracing::debug!(job = %job_name, method = %method, "Executing ingestion job");

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

    async fn execute_cleanup_task(storage: &Arc<dyn StorageBackend>, retention_days: u32) {
        let cutoff = chrono::Utc::now().timestamp() - (retention_days as i64 * 86400);
        match storage.cleanup_before(cutoff).await {
            Ok(deleted) => {
                if deleted > 0 {
                    tracing::info!(
                        deleted_rows = deleted,
                        retention_days = retention_days,
                        "Cleaned up old metrics"
                    );
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to cleanup old metrics");
            }
        }
    }

    async fn execute_metadata_refresh_task(
        storage: &Arc<dyn StorageBackend>,
        cache: &Arc<RwLock<Vec<(String, String)>>>,
    ) {
        match storage.get_available_metrics().await {
            Ok(metrics) => {
                let mut guard = cache.write().await;
                *guard = metrics;
                tracing::debug!("Refreshed metadata cache");
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to refresh metadata");
            }
        }
    }

    async fn refresh_metadata(&self) {
        Self::execute_metadata_refresh_task(&self.storage, &self.metadata_cache).await;
    }
}
