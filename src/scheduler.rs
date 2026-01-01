//! Dynamic job scheduler with runtime add/remove support.
//!
//! Provides a shared scheduler handle that can be accessed from both
//! TaskManager (for system tasks) and web handlers (for user-managed jobs).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

use crate::client::DataSourceClient;
use crate::config::{IngestionJob, Schedule};
use crate::storage::{Event, EventType, StorageBackend};

/// Handle for dynamic job scheduling operations.
///
/// Wraps `JobScheduler` with a mapping from database job IDs to scheduler UUIDs,
/// enabling runtime add/remove/update of scheduled jobs.
///
/// This struct is `Clone` and can be safely shared across handlers and tasks.
#[derive(Clone)]
pub struct SchedulerHandle {
    scheduler: JobScheduler,
    /// Maps database job ID -> scheduler job UUID
    job_map: Arc<RwLock<HashMap<i64, Uuid>>>,
    client: Arc<dyn DataSourceClient>,
    storage: Arc<dyn StorageBackend>,
    instance_id: String,
}

impl SchedulerHandle {
    /// Create a new SchedulerHandle.
    pub async fn new(
        client: Arc<dyn DataSourceClient>,
        storage: Arc<dyn StorageBackend>,
        instance_id: String,
    ) -> anyhow::Result<Self> {
        let scheduler = JobScheduler::new().await?;
        Ok(Self {
            scheduler,
            job_map: Arc::new(RwLock::new(HashMap::new())),
            client,
            storage,
            instance_id,
        })
    }

    /// Schedule a job by database ID.
    ///
    /// Returns the scheduler UUID if successful.
    /// If the job is disabled or already scheduled, returns an error.
    pub async fn schedule_job(&self, job_id: i64, job: &IngestionJob) -> anyhow::Result<Uuid> {
        if !job.enabled {
            anyhow::bail!("Cannot schedule disabled job '{}'", job.name);
        }

        // Check for duplicate scheduling to prevent orphan tasks
        if self.is_scheduled(job_id).await {
            anyhow::bail!(
                "Job '{}' (id={}) is already scheduled. Use reschedule_job() to update.",
                job.name,
                job_id
            );
        }

        let cron_job = self.create_ingestion_job(job)?;
        let uuid = self.scheduler.add(cron_job).await?;

        {
            let mut map = self.job_map.write().await;
            map.insert(job_id, uuid);
        }

        // Record TaskScheduled event
        let event = Event::new(
            &self.instance_id,
            EventType::TaskScheduled,
            format!("Task '{}' scheduled", job.name),
        );
        if let Err(e) = self.storage.store_event(&event).await {
            tracing::warn!(error = %e, "Failed to record task scheduled event");
        }

        tracing::info!(
            job_id = job_id,
            uuid = %uuid,
            name = %job.name,
            "Job scheduled dynamically"
        );
        Ok(uuid)
    }

    /// Remove a job from the scheduler by database ID.
    ///
    /// If the job is not currently scheduled, this is a no-op.
    pub async fn unschedule_job(&self, job_id: i64) -> anyhow::Result<()> {
        let uuid = {
            let mut map = self.job_map.write().await;
            map.remove(&job_id)
        };

        if let Some(uuid) = uuid {
            self.scheduler.remove(&uuid).await?;
            tracing::info!(job_id = job_id, uuid = %uuid, "Job unscheduled");
        } else {
            tracing::debug!(job_id = job_id, "Job was not scheduled, nothing to remove");
        }
        Ok(())
    }

    /// Update a job (remove old + add new if enabled).
    pub async fn reschedule_job(&self, job_id: i64, job: &IngestionJob) -> anyhow::Result<()> {
        // Always remove first
        self.unschedule_job(job_id).await?;

        // Only re-add if enabled
        if job.enabled {
            self.schedule_job(job_id, job).await?;
        }
        Ok(())
    }

    /// Check if a job is currently scheduled.
    pub async fn is_scheduled(&self, job_id: i64) -> bool {
        let map = self.job_map.read().await;
        map.contains_key(&job_id)
    }

    /// Load all enabled jobs from database and schedule them.
    pub async fn load_jobs_from_db(&self) -> anyhow::Result<usize> {
        let db_jobs = self.storage.list_jobs().await?;
        tracing::info!(count = db_jobs.len(), "Loaded jobs from database");

        let mut scheduled_count = 0;
        for record in db_jobs {
            if !record.job.enabled {
                tracing::debug!(name = %record.job.name, "Skipping disabled job");
                continue;
            }

            match self.schedule_job(record.id, &record.job).await {
                Ok(_) => {
                    scheduled_count += 1;
                    tracing::info!(
                        name = %record.job.name,
                        method = %record.job.method,
                        "Ingestion job scheduled"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        name = %record.job.name,
                        error = %e,
                        "Failed to schedule job"
                    );
                }
            }
        }

        Ok(scheduled_count)
    }

    /// Start the scheduler.
    pub async fn start(&self) -> anyhow::Result<()> {
        self.scheduler.start().await?;
        tracing::info!("Scheduler started");
        Ok(())
    }

    /// Shutdown the scheduler gracefully.
    pub async fn shutdown(&self) -> anyhow::Result<()> {
        // JobScheduler::shutdown takes &mut self, but we have shared access.
        // Clone creates a new handle to the same internal scheduler.
        let mut scheduler = self.scheduler.clone();
        scheduler.shutdown().await?;
        tracing::info!("Scheduler shutdown complete");
        Ok(())
    }

    /// Get reference to client (for trigger_job).
    pub fn client(&self) -> &Arc<dyn DataSourceClient> {
        &self.client
    }

    /// Get reference to storage (for trigger_job).
    pub fn storage(&self) -> &Arc<dyn StorageBackend> {
        &self.storage
    }

    /// Get instance ID.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Add a system job to the scheduler.
    ///
    /// System jobs (cleanup, metadata refresh) are not tracked in job_map
    /// since they don't have database IDs and don't need dynamic management.
    pub async fn add_system_job(&self, job: Job) -> anyhow::Result<()> {
        self.scheduler.add(job).await?;
        Ok(())
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    fn create_ingestion_job(&self, job_config: &IngestionJob) -> anyhow::Result<Job> {
        let client = Arc::clone(&self.client);
        let storage = Arc::clone(&self.storage);
        let job_name = job_config.name.clone();
        let method = job_config.method.clone();
        let params = job_config.params.clone();
        let instance_id = self.instance_id.clone();

        let job = match &job_config.schedule {
            Schedule::Interval { interval_secs } => {
                let duration = Duration::from_secs(*interval_secs);
                Job::new_repeated_async(duration, move |_uuid, _lock| {
                    let client = Arc::clone(&client);
                    let storage = Arc::clone(&storage);
                    let job_name = job_name.clone();
                    let method = method.clone();
                    let params = params.clone();
                    let instance_id = instance_id.clone();
                    Box::pin(async move {
                        execute_ingestion_job(
                            &job_name,
                            &method,
                            params,
                            &client,
                            &storage,
                            &instance_id,
                        )
                        .await;
                    })
                })?
            }
            Schedule::Cron { cron } => {
                let cron_expr = normalize_cron(cron);
                Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
                    let client = Arc::clone(&client);
                    let storage = Arc::clone(&storage);
                    let job_name = job_name.clone();
                    let method = method.clone();
                    let params = params.clone();
                    let instance_id = instance_id.clone();
                    Box::pin(async move {
                        execute_ingestion_job(
                            &job_name,
                            &method,
                            params,
                            &client,
                            &storage,
                            &instance_id,
                        )
                        .await;
                    })
                })?
            }
        };

        Ok(job)
    }
}

// =============================================================================
// Free functions (used by scheduler jobs and trigger_job)
// =============================================================================

/// Normalize cron expression to 6-field format.
fn normalize_cron(cron: &str) -> String {
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() == 5 {
        format!("0 {}", cron)
    } else {
        cron.to_string()
    }
}

/// Execute an ingestion job (fetch data and store metrics).
///
/// This is a public function so it can be called for manual job triggers.
pub async fn execute_ingestion_job(
    job_name: &str,
    method: &str,
    params: Option<serde_json::Value>,
    client: &Arc<dyn DataSourceClient>,
    storage: &Arc<dyn StorageBackend>,
    instance_id: &str,
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
                // Record TaskFailed event
                let event = Event::new(
                    instance_id,
                    EventType::TaskFailed,
                    format!("Task '{}' failed to store metrics: {}", job_name, e),
                );
                if let Err(e) = storage.store_event(&event).await {
                    tracing::error!(error = %e, "Failed to record task failed event");
                }
            } else {
                // Record TaskExecuted event
                let event = Event::new(
                    instance_id,
                    EventType::TaskExecuted,
                    format!(
                        "Task '{}' executed successfully, {} metrics",
                        job_name,
                        metrics.len()
                    ),
                );
                if let Err(e) = storage.store_event(&event).await {
                    tracing::error!(error = %e, "Failed to record task executed event");
                }
            }
        }
        Err(e) => {
            tracing::error!(
                job = %job_name,
                error = %e,
                "Failed to fetch metrics"
            );
            // Record TaskFailed event
            let event = Event::new(
                instance_id,
                EventType::TaskFailed,
                format!("Task '{}' failed to fetch metrics: {}", job_name, e),
            );
            if let Err(e) = storage.store_event(&event).await {
                tracing::error!(error = %e, "Failed to record task failed event");
            }
        }
    }
}

/// Manually trigger a job execution without scheduling.
pub async fn trigger_job(
    job: &IngestionJob,
    client: &Arc<dyn DataSourceClient>,
    storage: &Arc<dyn StorageBackend>,
    instance_id: &str,
) {
    tracing::info!(name = %job.name, "Manually triggering job execution");
    execute_ingestion_job(
        &job.name,
        &job.method,
        job.params.clone(),
        client,
        storage,
        instance_id,
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::alternativeme::Client as AlternativeMeClient;
    use crate::storage::local::{LocalStorage, LocalStorageConfig};
    use crate::{DataSource, config::Schedule};

    async fn create_test_scheduler() -> SchedulerHandle {
        let storage = LocalStorage::new_in_memory(LocalStorageConfig::default())
            .await
            .unwrap();
        let client = Arc::new(AlternativeMeClient::new());
        SchedulerHandle::new(client, Arc::new(storage), "test-instance".to_string())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_schedule_and_unschedule_job() {
        let handle = create_test_scheduler().await;

        let job = IngestionJob {
            name: "test_job".to_string(),
            datasource: DataSource::AlternativeMe,
            method: "get_fear_and_greed".to_string(),
            schedule: Schedule::Interval {
                interval_secs: 3600,
            },
            params: None,
            retention_days: 7,
            enabled: true,
        };

        // Schedule
        let uuid = handle.schedule_job(1, &job).await.unwrap();
        assert!(handle.is_scheduled(1).await);

        // Verify UUID is stored
        {
            let map = handle.job_map.read().await;
            assert_eq!(map.get(&1), Some(&uuid));
        }

        // Unschedule
        handle.unschedule_job(1).await.unwrap();
        assert!(!handle.is_scheduled(1).await);
    }

    #[tokio::test]
    async fn test_schedule_disabled_job_fails() {
        let handle = create_test_scheduler().await;

        let job = IngestionJob {
            name: "disabled_job".to_string(),
            datasource: DataSource::AlternativeMe,
            method: "get_fear_and_greed".to_string(),
            schedule: Schedule::Interval {
                interval_secs: 3600,
            },
            params: None,
            retention_days: 7,
            enabled: false,
        };

        let result = handle.schedule_job(1, &job).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reschedule_job() {
        let handle = create_test_scheduler().await;

        let job = IngestionJob {
            name: "test_job".to_string(),
            datasource: DataSource::AlternativeMe,
            method: "get_fear_and_greed".to_string(),
            schedule: Schedule::Interval {
                interval_secs: 3600,
            },
            params: None,
            retention_days: 7,
            enabled: true,
        };

        // Schedule initially
        handle.schedule_job(1, &job).await.unwrap();
        assert!(handle.is_scheduled(1).await);

        // Reschedule with disabled
        let disabled_job = IngestionJob {
            enabled: false,
            ..job.clone()
        };
        handle.reschedule_job(1, &disabled_job).await.unwrap();
        assert!(!handle.is_scheduled(1).await);

        // Reschedule with enabled again
        handle.reschedule_job(1, &job).await.unwrap();
        assert!(handle.is_scheduled(1).await);
    }
}
