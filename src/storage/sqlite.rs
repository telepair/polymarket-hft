//! SQLite-based persistent storage for time-series metrics.
//!
//! Uses `sqlx` for async database operations with WAL mode for better concurrency.

use crate::{DataSource, Metric, MetricUnit};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

/// SQLite-based persistent storage for time-series metrics.
pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    /// Opens or creates a SQLite database with WAL mode enabled.
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file
    pub async fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path_str = path.as_ref().to_string_lossy();

        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", path_str))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .busy_timeout(std::time::Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let storage = Self { pool };
        storage.init_schema().await?;
        Ok(storage)
    }

    /// Opens an in-memory database (for testing).
    pub async fn open_in_memory() -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .max_connections(1) // In-memory requires single connection
            .connect_with(options)
            .await?;

        let storage = Self { pool };
        storage.init_schema().await?;
        Ok(storage)
    }

    async fn init_schema(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source TEXT NOT NULL,
                name TEXT NOT NULL,
                value REAL NOT NULL,
                timestamp INTEGER NOT NULL,
                unit TEXT,
                labels TEXT,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create index for efficient queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_metrics_source_name_ts
            ON metrics(source, name, timestamp DESC)
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_metrics_timestamp
            ON metrics(timestamp DESC)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Insert a batch of metrics using multi-row INSERT for better performance.
    ///
    /// Batches are split into chunks of 100 rows to stay within SQLite limits.
    pub async fn insert_batch(&self, metrics: &[Metric]) -> anyhow::Result<()> {
        if metrics.is_empty() {
            return Ok(());
        }

        // SQLite has a limit on the number of variables per query (SQLITE_MAX_VARIABLE_NUMBER)
        // Default is 999, so with 5 columns per row, we can insert ~199 rows per statement
        // We use 100 rows per batch to be safe and efficient
        const BATCH_SIZE: usize = 100;

        let mut tx = self.pool.begin().await?;

        for chunk in metrics.chunks(BATCH_SIZE) {
            // Build multi-row INSERT: INSERT INTO metrics (...) VALUES (...), (...), ...
            let placeholders: Vec<String> = chunk
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let base = i * 6;
                    format!(
                        "(${}, ${}, ${}, ${}, ${}, ${})",
                        base + 1,
                        base + 2,
                        base + 3,
                        base + 4,
                        base + 5,
                        base + 6
                    )
                })
                .collect();

            let sql = format!(
                "INSERT INTO metrics (source, name, value, timestamp, unit, labels) VALUES {}",
                placeholders.join(", ")
            );

            let mut query = sqlx::query(&sql);

            for metric in chunk {
                let labels = if metric.labels.is_empty() {
                    None
                } else {
                    Some(serde_json::to_string(&metric.labels)?)
                };

                query = query
                    .bind(metric.source.to_string())
                    .bind(&metric.name)
                    .bind(metric.value)
                    .bind(metric.timestamp)
                    .bind(metric.unit.to_string())
                    .bind(labels);
            }

            query.execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Query the latest metric for a given source and name.
    pub async fn get_latest(&self, source: &str, name: &str) -> anyhow::Result<Option<Metric>> {
        let row: Option<MetricRow> = sqlx::query_as(
            r#"
            SELECT source, name, value, timestamp, unit, labels
            FROM metrics
            WHERE source = $1 AND name = $2
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(source)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    /// Query metrics within a time range.
    pub async fn query_range(
        &self,
        source: Option<&str>,
        name: Option<&str>,
        start: i64,
        end: i64,
        limit: usize,
    ) -> anyhow::Result<Vec<Metric>> {
        // Build dynamic SQL query
        let mut sql = String::from(
            "SELECT source, name, value, timestamp, unit, labels FROM metrics WHERE timestamp >= ? AND timestamp <= ?",
        );

        if source.is_some() {
            sql.push_str(" AND source = ?");
        }
        if name.is_some() {
            sql.push_str(" AND name = ?");
        }
        sql.push_str(" ORDER BY timestamp DESC LIMIT ?");

        // Build and execute query with dynamic bindings
        let mut query = sqlx::query_as::<_, MetricRow>(&sql).bind(start).bind(end);

        if let Some(s) = source {
            query = query.bind(s);
        }
        if let Some(n) = name {
            query = query.bind(n);
        }
        query = query.bind(limit as i64);

        let rows = query.fetch_all(&self.pool).await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Delete metrics older than the specified timestamp.
    ///
    /// Returns the number of deleted rows.
    pub async fn cleanup_before(&self, cutoff_timestamp: i64) -> anyhow::Result<u64> {
        let result = sqlx::query("DELETE FROM metrics WHERE timestamp < $1")
            .bind(cutoff_timestamp)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Perform a health check.
    pub async fn health_check(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    /// Get available metrics (source, name) pairs.
    pub async fn get_available_metrics(&self) -> anyhow::Result<Vec<(String, String)>> {
        let rows = sqlx::query("SELECT DISTINCT source, name FROM metrics ORDER BY source, name")
            .fetch_all(&self.pool)
            .await?;

        let mut metrics = Vec::new();
        for row in rows {
            use sqlx::Row;
            let source: String = row.try_get("source")?;
            let name: String = row.try_get("name")?;
            metrics.push((source, name));
        }
        Ok(metrics)
    }
}

/// Internal row structure for SQLite query results.
///
/// Maps database columns to `Metric` fields via `TryFrom`.
#[derive(sqlx::FromRow)]
struct MetricRow {
    source: String,
    name: String,
    value: f64,
    timestamp: i64,
    unit: Option<String>,
    labels: Option<String>,
}

impl TryFrom<MetricRow> for Metric {
    type Error = anyhow::Error;

    fn try_from(row: MetricRow) -> Result<Self, Self::Error> {
        let source: DataSource = row.source.parse()?;
        let unit: MetricUnit = row
            .unit
            .as_deref()
            .map(|s| s.parse())
            .transpose()?
            .unwrap_or_default();
        let labels = match row.labels {
            Some(json) => serde_json::from_str(&json)?,
            None => std::collections::HashMap::new(),
        };

        Ok(Metric {
            source,
            name: row.name,
            value: row.value,
            timestamp: row.timestamp,
            unit,
            labels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_storage_crud() {
        let storage = SqliteStorage::open_in_memory().await.unwrap();
        let metric = Metric::new(DataSource::AlternativeMe, "test", 42.0, MetricUnit::Index)
            .with_timestamp(1000);

        storage
            .insert_batch(std::slice::from_ref(&metric))
            .await
            .unwrap();

        let result = storage.get_latest("alternativeme", "test").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, 42.0);
    }

    #[tokio::test]
    async fn test_sqlite_storage_query_range() {
        let storage = SqliteStorage::open_in_memory().await.unwrap();
        let metrics = vec![
            Metric::new(DataSource::AlternativeMe, "test", 1.0, MetricUnit::Index)
                .with_timestamp(100),
            Metric::new(DataSource::AlternativeMe, "test", 2.0, MetricUnit::Index)
                .with_timestamp(200),
            Metric::new(DataSource::AlternativeMe, "test", 3.0, MetricUnit::Index)
                .with_timestamp(300),
        ];

        storage.insert_batch(&metrics).await.unwrap();

        let results = storage
            .query_range(Some("alternativeme"), Some("test"), 150, 350, 10)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_sqlite_storage_cleanup() {
        let storage = SqliteStorage::open_in_memory().await.unwrap();
        let metrics = vec![
            Metric::new(DataSource::AlternativeMe, "old", 1.0, MetricUnit::Index)
                .with_timestamp(100),
            Metric::new(DataSource::AlternativeMe, "new", 2.0, MetricUnit::Index)
                .with_timestamp(1000),
        ];

        storage.insert_batch(&metrics).await.unwrap();

        let deleted = storage.cleanup_before(500).await.unwrap();
        assert_eq!(deleted, 1);

        let old = storage.get_latest("alternativeme", "old").await.unwrap();
        assert!(old.is_none());

        let new = storage.get_latest("alternativeme", "new").await.unwrap();
        assert!(new.is_some());
    }

    #[tokio::test]
    async fn test_sqlite_storage_health_check() {
        let storage = SqliteStorage::open_in_memory().await.unwrap();
        assert!(storage.health_check().await.is_ok());
    }
}
