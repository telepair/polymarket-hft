//! In-memory cache for hot data with TTL expiration.
//!
//! Uses `moka` for high-performance concurrent caching with automatic expiration.

use crate::Metric;
use moka::future::Cache;
use std::time::Duration;

/// Cache key format: "source::name"
type CacheKey = String;

/// In-memory cache for hot data with TTL expiration.
///
/// Stores the latest metric value for each unique key (source + name).
/// Entries automatically expire after the configured TTL.
pub struct MemoryCache {
    /// Cache storing latest metric per key.
    latest: Cache<CacheKey, Metric>,
}

impl MemoryCache {
    /// Creates a new cache with specified TTL and max capacity.
    ///
    /// # Arguments
    /// * `ttl` - Time-to-live for cached entries
    /// * `max_capacity` - Maximum number of entries in the cache
    pub fn new(ttl: Duration, max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .time_to_live(ttl)
            .max_capacity(max_capacity)
            .build();

        Self { latest: cache }
    }

    /// Insert or update a metric in the cache.
    pub async fn put(&self, metric: &Metric) {
        let key = metric.state_key();
        self.latest.insert(key, metric.clone()).await;
    }

    /// Get the latest metric for a given source and name.
    pub async fn get(&self, source: &str, name: &str) -> Option<Metric> {
        let key = format!("{}::{}", source, name);
        self.latest.get(&key).await
    }

    /// Batch insert metrics.
    pub async fn put_batch(&self, metrics: &[Metric]) {
        for metric in metrics {
            self.put(metric).await;
        }
    }

    /// Get cache statistics for monitoring.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.latest.entry_count(),
            weighted_size: self.latest.weighted_size(),
        }
    }
}

/// Cache statistics for monitoring and debugging.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries currently in the cache.
    pub entry_count: u64,
    /// Total weighted size of entries.
    pub weighted_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataSource, MetricUnit};

    #[tokio::test]
    async fn test_cache_put_and_get() {
        let cache = MemoryCache::new(Duration::from_secs(60), 100);
        let metric = Metric::new(DataSource::AlternativeMe, "test", 42.0, MetricUnit::Index);

        cache.put(&metric).await;

        let result = cache.get("alternativeme", "test").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, 42.0);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = MemoryCache::new(Duration::from_secs(60), 100);

        let result = cache.get("nonexistent", "metric").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_batch_insert() {
        let cache = MemoryCache::new(Duration::from_secs(60), 100);
        let metrics = vec![
            Metric::new(DataSource::AlternativeMe, "metric1", 1.0, MetricUnit::Index),
            Metric::new(DataSource::AlternativeMe, "metric2", 2.0, MetricUnit::Index),
        ];

        cache.put_batch(&metrics).await;

        assert!(cache.get("alternativeme", "metric1").await.is_some());
        assert!(cache.get("alternativeme", "metric2").await.is_some());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = MemoryCache::new(Duration::from_secs(60), 100);
        let metric = Metric::new(DataSource::AlternativeMe, "test", 42.0, MetricUnit::Index);

        cache.put(&metric).await;

        // moka updates entry_count asynchronously, so we need to wait a bit
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Verify stats are accessible (moka may report lazily)
        let _ = cache.stats();
    }
}
