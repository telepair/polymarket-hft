//! Storage layer module.
//!
//! This module provides data models and storage backends for metrics persistence.
//! Supports both local (SQLite + memory cache) and external (Redis + TimescaleDB) storage.

pub mod cache;
pub mod local;
pub mod model;
pub mod sqlite;

// Re-export BoxFuture from client module to avoid duplication
pub use crate::client::BoxFuture;

/// Storage backend trait for metrics persistence.
///
/// Implementations handle both hot data caching and cold data persistence.
/// The trait is designed to be object-safe and async-compatible.
pub trait StorageBackend: Send + Sync {
    /// Store a batch of metrics.
    ///
    /// Implementations should handle both caching (for hot data) and
    /// persistence (for cold data) as appropriate.
    fn store(&self, metrics: &[model::Metric]) -> BoxFuture<'_, anyhow::Result<()>>;

    /// Get the latest value for a metric by source and name.
    ///
    /// Returns `None` if the metric is not found.
    fn get_latest(
        &self,
        source: &str,
        name: &str,
    ) -> BoxFuture<'_, anyhow::Result<Option<model::Metric>>>;

    /// Query metrics within a time range.
    ///
    /// # Arguments
    /// * `source` - Optional filter by data source
    /// * `name` - Optional filter by metric name
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (inclusive)
    /// * `limit` - Maximum number of results
    fn query_range(
        &self,
        source: Option<&str>,
        name: Option<&str>,
        start: i64,
        end: i64,
        limit: Option<usize>,
    ) -> BoxFuture<'_, anyhow::Result<Vec<model::Metric>>>;

    /// Perform a health check on the storage backend.
    fn health_check(&self) -> BoxFuture<'_, anyhow::Result<()>>;

    /// Delete metrics older than the specified timestamp.
    ///
    /// Returns the number of deleted rows.
    fn cleanup_before(&self, cutoff_timestamp: i64) -> BoxFuture<'_, anyhow::Result<u64>>;

    /// Get available metrics (source, name) pairs.
    fn get_available_metrics(&self) -> BoxFuture<'_, anyhow::Result<Vec<(String, String)>>>;

    /// Store a system event.
    fn store_event(&self, event: &model::Event) -> BoxFuture<'_, anyhow::Result<()>>;

    /// Get events with optional instance ID filter.
    ///
    /// # Arguments
    /// * `instance_id` - Optional filter by instance ID
    /// * `limit` - Maximum number of events to return
    fn get_events(
        &self,
        instance_id: Option<&str>,
        limit: Option<usize>,
    ) -> BoxFuture<'_, anyhow::Result<Vec<model::Event>>>;

    /// Get distinct instance IDs from events.
    fn get_distinct_instance_ids(&self) -> BoxFuture<'_, anyhow::Result<Vec<String>>>;

    // =========================================================================
    // Job Management
    // =========================================================================

    /// Store a new job in the database.
    ///
    /// Returns the ID of the newly created job.
    fn store_job(&self, job: &crate::config::IngestionJob) -> BoxFuture<'_, anyhow::Result<i64>>;

    /// Update an existing job by ID.
    fn update_job(
        &self,
        id: i64,
        job: &crate::config::IngestionJob,
    ) -> BoxFuture<'_, anyhow::Result<()>>;

    /// Delete a job by ID.
    fn delete_job(&self, id: i64) -> BoxFuture<'_, anyhow::Result<()>>;

    /// Get a job by ID.
    fn get_job(&self, id: i64) -> BoxFuture<'_, anyhow::Result<Option<model::JobRecord>>>;

    /// List all jobs from the database.
    fn list_jobs(&self) -> BoxFuture<'_, anyhow::Result<Vec<model::JobRecord>>>;
}

// ============================================================================
// Re-exports
// ============================================================================

pub use local::{LocalStorage, LocalStorageConfig};
pub use model::{DataSource, Event, EventType, JobRecord, Metric, MetricUnit};
