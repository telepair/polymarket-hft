//! Task manager for scheduling and running all background tasks.
//!
//! Manages:
//! - Data ingestion jobs (via SchedulerHandle)
//! - System maintenance tasks (data cleanup)
//! - Metadata refresh tasks

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio_cron_scheduler::Job;
use tokio_util::sync::CancellationToken;

use crate::config::StorageConfig;
use crate::scheduler::SchedulerHandle;
use crate::storage::StorageBackend;

/// Manages all background tasks and coordinates data collection.
///
/// The TaskManager now uses a shared SchedulerHandle for ingestion jobs,
/// while still managing system tasks (cleanup, metadata refresh) internally.
pub struct TaskManager {
    scheduler: SchedulerHandle,
    storage: Arc<dyn StorageBackend>,
    metadata_cache: Arc<RwLock<Vec<(String, String)>>>,
    config: StorageConfig,
    shutdown_token: CancellationToken,
}

impl TaskManager {
    /// Creates a new TaskManager.
    ///
    /// Jobs are managed via SchedulerHandle which is shared with web handlers.
    pub fn new(
        scheduler: SchedulerHandle,
        storage: Arc<dyn StorageBackend>,
        metadata_cache: Arc<RwLock<Vec<(String, String)>>>,
        config: StorageConfig,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            scheduler,
            storage,
            metadata_cache,
            config,
            shutdown_token,
        }
    }

    /// Runs all tasks (ingestion and system) until shutdown signal is received.
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("Starting task manager");

        // Load and schedule ingestion jobs from database
        let scheduled_count = self.scheduler.load_jobs_from_db().await?;
        tracing::info!(count = scheduled_count, "Ingestion jobs scheduled");

        // Schedule system tasks (cleanup, metadata refresh)
        self.schedule_system_tasks().await?;

        // Start the scheduler
        self.scheduler.start().await?;

        // Wait for shutdown signal from the shared cancellation token
        self.shutdown_token.cancelled().await;
        tracing::info!("Task manager received shutdown signal, stopping scheduler");

        self.scheduler.shutdown().await?;

        Ok(())
    }

    /// Schedule system maintenance tasks.
    async fn schedule_system_tasks(&self) -> anyhow::Result<()> {
        // Schedule Cleanup Task
        if self.config.cleanup_interval_secs > 0 {
            let cleanup_job = self.create_cleanup_job()?;
            // Access the internal scheduler to add system jobs
            // We use a separate method since these are not user-managed jobs
            self.add_system_job(cleanup_job).await?;
            tracing::info!(
                interval_secs = self.config.cleanup_interval_secs,
                "Cleanup task scheduled"
            );
        }

        // Schedule Metadata Refresh Task
        if self.config.metadata_refresh_interval_secs > 0 {
            let refresh_job = self.create_metadata_refresh_job()?;
            self.add_system_job(refresh_job).await?;
            tracing::info!(
                interval_secs = self.config.metadata_refresh_interval_secs,
                "Metadata refresh task scheduled"
            );

            // Run initial refresh immediately
            self.refresh_metadata().await;
        }

        Ok(())
    }

    /// Add a system job to the scheduler.
    ///
    /// System jobs are internal (cleanup, metadata refresh) and not tracked
    /// in the job_map since they don't have database IDs.
    async fn add_system_job(&self, job: Job) -> anyhow::Result<()> {
        // Access the scheduler directly through the handle
        // Note: We could expose an add_system_job method on SchedulerHandle,
        // but for now we'll use a workaround by accessing it internally.
        // This is safe because system jobs don't need to be removed dynamically.

        // Since SchedulerHandle wraps JobScheduler which is Clone,
        // we can add jobs directly. The job_map is only for user-managed jobs.
        self.scheduler.add_system_job(job).await
    }

    fn create_cleanup_job(&self) -> anyhow::Result<Job> {
        let storage = Arc::clone(&self.storage);
        let retention_days = self.config.retention_days;
        let duration = Duration::from_secs(self.config.cleanup_interval_secs);

        Job::new_repeated_async(duration, move |_uuid, _lock| {
            let storage = Arc::clone(&storage);
            Box::pin(async move {
                execute_cleanup_task(&storage, retention_days).await;
            })
        })
        .map_err(Into::into)
    }

    fn create_metadata_refresh_job(&self) -> anyhow::Result<Job> {
        let storage = Arc::clone(&self.storage);
        let cache = Arc::clone(&self.metadata_cache);
        let duration = Duration::from_secs(self.config.metadata_refresh_interval_secs);

        Job::new_repeated_async(duration, move |_uuid, _lock| {
            let storage = Arc::clone(&storage);
            let cache = Arc::clone(&cache);
            Box::pin(async move {
                execute_metadata_refresh_task(&storage, &cache).await;
            })
        })
        .map_err(Into::into)
    }

    async fn refresh_metadata(&self) {
        execute_metadata_refresh_task(&self.storage, &self.metadata_cache).await;
    }
}

// =============================================================================
// System task executors
// =============================================================================

async fn execute_cleanup_task(storage: &Arc<dyn StorageBackend>, retention_days: u32) {
    // Validate retention_days to prevent overflow (max ~24 years with millis in i64)
    const MAX_RETENTION_DAYS: u32 = 10000;
    let safe_retention = retention_days.min(MAX_RETENTION_DAYS);
    if retention_days > MAX_RETENTION_DAYS {
        tracing::warn!(
            requested = retention_days,
            capped = safe_retention,
            "retention_days exceeded maximum, capping value"
        );
    }

    // Calculate cutoff in milliseconds (86400 seconds/day * 1000 ms/second)
    const MILLIS_PER_DAY: i64 = 86_400_000;
    let cutoff = chrono::Utc::now().timestamp_millis() - (safe_retention as i64 * MILLIS_PER_DAY);
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
