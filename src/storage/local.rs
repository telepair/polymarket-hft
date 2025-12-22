//! Combined local storage with memory cache and SQLite persistence.
//!
//! Implements write-through caching: writes go to both cache and SQLite.
//! Reads prioritize cache, falling back to SQLite on cache miss.

use super::cache::MemoryCache;
use super::sqlite::SqliteStorage;
use super::{BoxFuture, StorageBackend};
use crate::Metric;
use std::path::PathBuf;
use std::sync::Arc;

use std::time::Duration;

/// Combined local storage with memory cache and SQLite persistence.
///
/// Write strategy: Write-through (writes to both cache and SQLite)
/// Read strategy: Cache-first (cache hit returns immediately, cache miss queries SQLite)
pub struct LocalStorage {
    cache: Arc<MemoryCache>,
    sqlite: Arc<SqliteStorage>,
}

impl LocalStorage {
    /// Creates a new LocalStorage instance.
    pub async fn new(config: LocalStorageConfig) -> anyhow::Result<Self> {
        let cache = Arc::new(MemoryCache::new(
            config.cache_ttl,
            config.cache_max_capacity,
        ));
        let sqlite = Arc::new(SqliteStorage::open(&config.db_path).await?);
        Ok(Self { cache, sqlite })
    }

    /// Creates a LocalStorage with in-memory SQLite (for testing).
    pub async fn new_in_memory(config: LocalStorageConfig) -> anyhow::Result<Self> {
        let cache = Arc::new(MemoryCache::new(
            config.cache_ttl,
            config.cache_max_capacity,
        ));
        let sqlite = Arc::new(SqliteStorage::open_in_memory().await?);
        Ok(Self { cache, sqlite })
    }

    /// Access the underlying SQLite storage for cleanup operations.
    pub fn sqlite(&self) -> &SqliteStorage {
        &self.sqlite
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> super::cache::CacheStats {
        self.cache.stats()
    }
}

impl StorageBackend for LocalStorage {
    fn store(&self, metrics: &[Metric]) -> BoxFuture<'_, anyhow::Result<()>> {
        // Clone metrics to move into async block
        let metrics = metrics.to_vec();
        Box::pin(async move {
            // Write-through: update cache and persist to SQLite
            self.cache.put_batch(&metrics).await;
            self.sqlite.insert_batch(&metrics).await?;
            Ok(())
        })
    }

    fn get_latest(
        &self,
        source: &str,
        name: &str,
    ) -> BoxFuture<'_, anyhow::Result<Option<Metric>>> {
        let source = source.to_string();
        let name = name.to_string();
        Box::pin(async move {
            // Cache-first strategy
            if let Some(metric) = self.cache.get(&source, &name).await {
                return Ok(Some(metric));
            }
            // Fallback to SQLite
            self.sqlite.get_latest(&source, &name).await
        })
    }

    fn query_range(
        &self,
        source: Option<&str>,
        name: Option<&str>,
        start: i64,
        end: i64,
        limit: Option<usize>,
    ) -> BoxFuture<'_, anyhow::Result<Vec<Metric>>> {
        let source = source.map(|s| s.to_string());
        let name = name.map(|s| s.to_string());
        Box::pin(async move {
            // Range queries always go to SQLite (cache only stores latest)
            self.sqlite
                .query_range(
                    source.as_deref(),
                    name.as_deref(),
                    start,
                    end,
                    limit.unwrap_or(1000),
                )
                .await
        })
    }

    fn health_check(&self) -> BoxFuture<'_, anyhow::Result<()>> {
        Box::pin(async move { self.sqlite.health_check().await })
    }

    fn cleanup_before(&self, cutoff_timestamp: i64) -> BoxFuture<'_, anyhow::Result<u64>> {
        Box::pin(async move { self.sqlite.cleanup_before(cutoff_timestamp).await })
    }

    fn get_available_metrics(&self) -> BoxFuture<'_, anyhow::Result<Vec<(String, String)>>> {
        Box::pin(async move { self.sqlite.get_available_metrics().await })
    }
}

/// Configuration for LocalStorage.
#[derive(Debug, Clone)]
pub struct LocalStorageConfig {
    /// Path to the SQLite database file.
    pub db_path: PathBuf,
    /// TTL for cached entries.
    pub cache_ttl: Duration,
    /// Maximum number of entries in the cache.
    pub cache_max_capacity: u64,
}

impl Default for LocalStorageConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("data/metrics.db"),
            cache_ttl: Duration::from_secs(900), // 15 minutes
            cache_max_capacity: 100_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataSource, MetricUnit};

    #[tokio::test]
    async fn test_local_storage_write_through() {
        let config = LocalStorageConfig {
            cache_ttl: Duration::from_secs(60),
            cache_max_capacity: 100,
            ..Default::default()
        };
        let storage = LocalStorage::new_in_memory(config).await.unwrap();

        let metric = Metric::new(DataSource::AlternativeMe, "test", 42.0, MetricUnit::Index);
        storage.store(&[metric]).await.unwrap();

        // Verify cache hit
        let cached = storage.get_latest("alternativeme", "test").await.unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().value, 42.0);
    }

    #[tokio::test]
    async fn test_local_storage_cache_miss_fallback() {
        let config = LocalStorageConfig {
            cache_ttl: Duration::from_millis(1), // Very short TTL
            cache_max_capacity: 100,
            ..Default::default()
        };
        let storage = LocalStorage::new_in_memory(config).await.unwrap();

        let metric = Metric::new(DataSource::AlternativeMe, "test", 42.0, MetricUnit::Index);
        storage.store(&[metric]).await.unwrap();

        // Wait for cache expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should still find in SQLite
        let result = storage.get_latest("alternativeme", "test").await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_local_storage_health_check() {
        let config = LocalStorageConfig::default();
        let storage = LocalStorage::new_in_memory(config).await.unwrap();
        assert!(storage.health_check().await.is_ok());
    }
}
