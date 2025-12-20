//! Archiver for storing collected metrics.
//!
//! This module provides the Archiver trait and implementations
//! for persisting metrics to various backends.

use crate::Metric;

/// Trait for archiving collected metrics.
///
/// Implementations of this trait handle the persistence of metrics
/// to various backends (logs, databases, etc.).
pub trait Archiver: Send + Sync {
    /// Archive a batch of metrics.
    fn archive(&self, metrics: &[Metric]);
}

/// Simple logging archiver that writes metrics to tracing logs.
///
/// This is the default archiver for initial development and debugging.
pub struct LogArchiver;

impl Archiver for LogArchiver {
    fn archive(&self, metrics: &[Metric]) {
        for metric in metrics {
            tracing::info!(
                source = %metric.source,
                name = %metric.name,
                value = metric.value,
                timestamp = metric.timestamp,
                labels = ?metric.labels,
                "Archived metric"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataSource;

    #[test]
    fn test_log_archiver_does_not_panic() {
        let archiver = LogArchiver;
        let metrics = vec![
            Metric::new(DataSource::AlternativeMe, "test_metric", 42.0),
            Metric::new(DataSource::AlternativeMe, "another_metric", 100.0),
        ];
        archiver.archive(&metrics);
    }
}
