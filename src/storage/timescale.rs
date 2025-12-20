//! TimescaleDB client for metric and scrape job storage.

use crate::engine::{Metric, Schedule, ScrapeJob, Target};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Scrape job row from database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScrapeJobRow {
    pub id: String,
    pub source: String,
    pub endpoint: String,
    pub params: serde_json::Value,
    pub targets: Vec<String>,
    pub schedule_type: String,
    pub schedule_value: String,
    pub state_ttl_secs: Option<i64>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ScrapeJobRow> for ScrapeJob {
    fn from(row: ScrapeJobRow) -> Self {
        let targets = row
            .targets
            .iter()
            .filter_map(|t| match t.as_str() {
                "metrics" => Some(Target::Metrics),
                "state" => Some(Target::State),
                _ => None,
            })
            .collect();

        let schedule = match row.schedule_type.as_str() {
            "cron" => Schedule::Cron {
                expression: row.schedule_value,
            },
            _ => Schedule::Interval {
                interval: humantime::parse_duration(&row.schedule_value)
                    .unwrap_or(Duration::from_secs(300)),
            },
        };

        ScrapeJob {
            id: row.id,
            source: row.source,
            endpoint: row.endpoint,
            params: row.params,
            targets,
            schedule,
            state_ttl: row.state_ttl_secs.map(|s| Duration::from_secs(s as u64)),
            enabled: row.enabled,
        }
    }
}

/// TimescaleDB client for storing and querying metrics and scrape jobs.
#[derive(Clone)]
pub struct TimescaleClient {
    pool: PgPool,
}

impl TimescaleClient {
    /// Creates a new TimescaleDB client.
    ///
    /// # Arguments
    ///
    /// * `database_url` - PostgreSQL connection string.
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Creates a new client from an existing pool.
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initializes the database schema (creates tables if not exist).
    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metrics (
                time TIMESTAMPTZ NOT NULL,
                source TEXT NOT NULL,
                name TEXT NOT NULL,
                value DOUBLE PRECISION NOT NULL,
                labels JSONB DEFAULT '{}'
            );

            -- Create hypertable if TimescaleDB extension is available
            DO $$
            BEGIN
                IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
                    PERFORM create_hypertable('metrics', 'time', if_not_exists => TRUE);
                END IF;
            END $$;

            -- Create indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_metrics_source_name_time 
                ON metrics (source, name, time DESC);
            CREATE INDEX IF NOT EXISTS idx_metrics_labels 
                ON metrics USING GIN (labels);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Inserts a single metric.
    pub async fn insert_metric(&self, metric: &Metric) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO metrics (time, source, name, value, labels)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(metric.time)
        .bind(&metric.source)
        .bind(&metric.name)
        .bind(metric.value)
        .bind(serde_json::to_value(&metric.labels).unwrap_or_default())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Batch inserts metrics for better performance.
    pub async fn insert_metrics(&self, metrics: &[Metric]) -> Result<(), sqlx::Error> {
        if metrics.is_empty() {
            return Ok(());
        }

        // Use a transaction for batch insert
        let mut tx = self.pool.begin().await?;

        for metric in metrics {
            sqlx::query(
                r#"
                INSERT INTO metrics (time, source, name, value, labels)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(metric.time)
            .bind(&metric.source)
            .bind(&metric.name)
            .bind(metric.value)
            .bind(serde_json::to_value(&metric.labels).unwrap_or_default())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Queries metrics by source and name within a time range.
    pub async fn query_metrics(
        &self,
        source: &str,
        name: &str,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
        limit: Option<i64>,
    ) -> Result<Vec<Metric>, sqlx::Error> {
        let limit = limit.unwrap_or(1000);

        let rows = sqlx::query_as::<_, MetricRow>(
            r#"
            SELECT time, source, name, value, labels
            FROM metrics
            WHERE source = $1 AND name = $2 AND time >= $3 AND time <= $4
            ORDER BY time DESC
            LIMIT $5
            "#,
        )
        .bind(source)
        .bind(name)
        .bind(start)
        .bind(end)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Gets the latest metric for a given source and name.
    pub async fn get_latest_metric(
        &self,
        source: &str,
        name: &str,
    ) -> Result<Option<Metric>, sqlx::Error> {
        let row = sqlx::query_as::<_, MetricRow>(
            r#"
            SELECT time, source, name, value, labels
            FROM metrics
            WHERE source = $1 AND name = $2
            ORDER BY time DESC
            LIMIT 1
            "#,
        )
        .bind(source)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    /// Returns a reference to the underlying connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // --- Scrape Jobs CRUD ---

    /// Initializes the scrape_jobs table schema.
    pub async fn init_scrape_jobs_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scrape_jobs (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                params JSONB DEFAULT '{}',
                targets TEXT[] NOT NULL,
                schedule_type TEXT NOT NULL CHECK (schedule_type IN ('interval', 'cron')),
                schedule_value TEXT NOT NULL,
                state_ttl_secs BIGINT,
                enabled BOOLEAN DEFAULT TRUE,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_scrape_jobs_source ON scrape_jobs (source);
            CREATE INDEX IF NOT EXISTS idx_scrape_jobs_enabled ON scrape_jobs (enabled);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Gets all scrape jobs.
    pub async fn get_scrape_jobs(&self) -> Result<Vec<ScrapeJobRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScrapeJobRow>(
            r#"
            SELECT id, source, endpoint, params, targets, schedule_type, schedule_value,
                   state_ttl_secs, enabled, created_at, updated_at
            FROM scrape_jobs
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Gets a scrape job by ID.
    pub async fn get_scrape_job(&self, id: &str) -> Result<Option<ScrapeJobRow>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScrapeJobRow>(
            r#"
            SELECT id, source, endpoint, params, targets, schedule_type, schedule_value,
                   state_ttl_secs, enabled, created_at, updated_at
            FROM scrape_jobs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Inserts a scrape job if it doesn't already exist.
    /// Returns true if inserted, false if already exists.
    pub async fn insert_scrape_job_if_not_exists(
        &self,
        job: &ScrapeJob,
    ) -> Result<bool, sqlx::Error> {
        let (schedule_type, schedule_value) = match &job.schedule {
            Schedule::Interval { interval } => (
                "interval",
                humantime::format_duration(*interval).to_string(),
            ),
            Schedule::Cron { expression } => ("cron", expression.clone()),
        };

        let targets: Vec<String> = job
            .targets
            .iter()
            .map(|t| match t {
                Target::Metrics => "metrics".to_string(),
                Target::State => "state".to_string(),
            })
            .collect();

        let result = sqlx::query(
            r#"
            INSERT INTO scrape_jobs (id, source, endpoint, params, targets, schedule_type, schedule_value, state_ttl_secs, enabled)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(&job.id)
        .bind(&job.source)
        .bind(&job.endpoint)
        .bind(&job.params)
        .bind(&targets)
        .bind(schedule_type)
        .bind(&schedule_value)
        .bind(job.state_ttl.map(|d| d.as_secs() as i64))
        .bind(job.enabled)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Creates a new scrape job.
    pub async fn create_scrape_job(&self, job: &ScrapeJob) -> Result<ScrapeJobRow, sqlx::Error> {
        let (schedule_type, schedule_value) = match &job.schedule {
            Schedule::Interval { interval } => (
                "interval",
                humantime::format_duration(*interval).to_string(),
            ),
            Schedule::Cron { expression } => ("cron", expression.clone()),
        };

        let targets: Vec<String> = job
            .targets
            .iter()
            .map(|t| match t {
                Target::Metrics => "metrics".to_string(),
                Target::State => "state".to_string(),
            })
            .collect();

        let row = sqlx::query_as::<_, ScrapeJobRow>(
            r#"
            INSERT INTO scrape_jobs (id, source, endpoint, params, targets, schedule_type, schedule_value, state_ttl_secs, enabled)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, source, endpoint, params, targets, schedule_type, schedule_value, state_ttl_secs, enabled, created_at, updated_at
            "#,
        )
        .bind(&job.id)
        .bind(&job.source)
        .bind(&job.endpoint)
        .bind(&job.params)
        .bind(&targets)
        .bind(schedule_type)
        .bind(&schedule_value)
        .bind(job.state_ttl.map(|d| d.as_secs() as i64))
        .bind(job.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Updates a scrape job.
    pub async fn update_scrape_job(
        &self,
        id: &str,
        job: &ScrapeJob,
    ) -> Result<Option<ScrapeJobRow>, sqlx::Error> {
        let (schedule_type, schedule_value) = match &job.schedule {
            Schedule::Interval { interval } => (
                "interval",
                humantime::format_duration(*interval).to_string(),
            ),
            Schedule::Cron { expression } => ("cron", expression.clone()),
        };

        let targets: Vec<String> = job
            .targets
            .iter()
            .map(|t| match t {
                Target::Metrics => "metrics".to_string(),
                Target::State => "state".to_string(),
            })
            .collect();

        let row = sqlx::query_as::<_, ScrapeJobRow>(
            r#"
            UPDATE scrape_jobs
            SET source = $2, endpoint = $3, params = $4, targets = $5, 
                schedule_type = $6, schedule_value = $7, state_ttl_secs = $8, 
                enabled = $9, updated_at = NOW()
            WHERE id = $1
            RETURNING id, source, endpoint, params, targets, schedule_type, schedule_value, state_ttl_secs, enabled, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&job.source)
        .bind(&job.endpoint)
        .bind(&job.params)
        .bind(&targets)
        .bind(schedule_type)
        .bind(&schedule_value)
        .bind(job.state_ttl.map(|d| d.as_secs() as i64))
        .bind(job.enabled)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Deletes a scrape job by ID.
    /// Returns true if deleted, false if not found.
    pub async fn delete_scrape_job(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scrape_jobs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Internal row type for sqlx mapping.
#[derive(sqlx::FromRow)]
struct MetricRow {
    time: chrono::DateTime<chrono::Utc>,
    source: String,
    name: String,
    value: f64,
    labels: serde_json::Value,
}

impl From<MetricRow> for Metric {
    fn from(row: MetricRow) -> Self {
        let labels = row
            .labels
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        Metric {
            time: row.time,
            source: row.source,
            name: row.name,
            value: row.value,
            labels,
        }
    }
}
