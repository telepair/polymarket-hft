//! HTTP request handlers for the web dashboard.

use std::sync::Arc;

use axum::{
    Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;

use crate::storage::StorageBackend;
use tokio::sync::RwLock;
use tracing;

use super::templates::{
    DashboardTemplate, FilterParams, LatestMetricView, MetricView, MetricsPartialTemplate,
    StatusTemplate,
};

// =============================================================================
// Router
// =============================================================================

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<dyn StorageBackend>,
    pub metadata_cache: Arc<RwLock<Vec<(String, String)>>>,
}

/// Create the Axum router with all routes.
pub fn create_router(
    storage: Arc<dyn StorageBackend>,
    metadata_cache: Arc<RwLock<Vec<(String, String)>>>,
) -> Router {
    let state = AppState {
        storage,
        metadata_cache,
    };

    Router::new()
        .route("/", get(index))
        .route("/status", get(status))
        .route("/partials/metrics", get(metrics_partial))
        .route("/api/metrics/latest", get(api_metrics_latest))
        .with_state(state)
}

// =============================================================================
// Query Parameters
// =============================================================================

/// Query parameters for filtering metrics.
#[derive(Debug, Deserialize, Default)]
pub struct MetricsQuery {
    /// Filter by data source (e.g., "alternativeme", "coingecko").
    #[serde(default)]
    pub source: Option<String>,

    /// Filter by metric name (partial match).
    #[serde(default)]
    pub name: Option<String>,

    /// Time range preset: "1h", "6h", "24h", "7d", "30d".
    #[serde(default = "default_time_range")]
    pub time_range: String,

    /// Maximum number of results.
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_time_range() -> String {
    "1h".to_string()
}

fn default_limit() -> usize {
    100
}

impl MetricsQuery {
    /// Convert time range preset to seconds.
    fn time_range_seconds(&self) -> i64 {
        match self.time_range.as_str() {
            "1h" => 3600,
            "6h" => 6 * 3600,
            "24h" => 24 * 3600,
            "7d" => 7 * 24 * 3600,
            "30d" => 30 * 24 * 3600,
            _ => 3600, // default to 1 hour
        }
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// Index page - renders the dashboard with filter form.
async fn index(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> impl IntoResponse {
    let mut available_metrics = state.metadata_cache.read().await.clone();

    // Fallback: If cache is empty, try to fetch from storage
    if available_metrics.is_empty() {
        tracing::debug!("Metadata cache empty, fetching from storage");
        if let Ok(metrics) = state.storage.get_available_metrics().await
            && !metrics.is_empty()
        {
            let mut cache = state.metadata_cache.write().await;
            *cache = metrics.clone();
            available_metrics = metrics;
            tracing::debug!(
                count = available_metrics.len(),
                "Metadata cache updated from storage"
            );
        }
    }

    let mut available_sources: Vec<String> =
        available_metrics.iter().map(|(s, _)| s.clone()).collect();
    available_sources.sort();
    available_sources.dedup();

    let mut available_names: Vec<String> =
        available_metrics.iter().map(|(_, n)| n.clone()).collect();
    available_names.sort();
    available_names.dedup();

    DashboardTemplate {
        title: "Metrics Dashboard".to_string(),
        filter_params: FilterParams {
            source: query.source.clone().unwrap_or_default(),
            name: query.name.clone().unwrap_or_default(),
            time_range: query.time_range.clone(),
        },
        available_sources,
        available_names,
    }
}

/// Status page - shows the latest value of each metric from cache.
async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let now = chrono::Utc::now();
    let available_metrics = state.metadata_cache.read().await.clone();

    let mut metrics = Vec::new();
    for (source, name) in available_metrics {
        if let Ok(Some(metric)) = state.storage.get_latest(&source, &name).await {
            let age_seconds = now.timestamp() - metric.timestamp;
            let timestamp = chrono::DateTime::from_timestamp(metric.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| metric.timestamp.to_string());

            metrics.push(LatestMetricView {
                source: metric.source.to_string(),
                name: metric.name,
                value: format!("{:.4}", metric.value),
                unit: metric.unit.to_string(),
                timestamp,
                age_seconds,
            });
        }
    }

    // Sort by source, then by name
    metrics.sort_by(|a, b| (&a.source, &a.name).cmp(&(&b.source, &b.name)));

    StatusTemplate {
        title: "System Status".to_string(),
        metrics,
        last_updated: now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    }
}

/// Partial for htmx updates - returns only the metrics table fragment.
async fn metrics_partial(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> impl IntoResponse {
    let metrics = fetch_filtered_metrics(&state, &query).await;
    MetricsPartialTemplate {
        metrics,
        filter_params: FilterParams {
            source: query.source.clone().unwrap_or_default(),
            name: query.name.clone().unwrap_or_default(),
            time_range: query.time_range.clone(),
        },
    }
}

/// JSON API endpoint for metrics with filtering.
async fn api_metrics_latest(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> axum::Json<Vec<serde_json::Value>> {
    let now = chrono::Utc::now().timestamp();
    let start = now - query.time_range_seconds();

    let source_filter = query.source.as_deref().filter(|s| !s.is_empty());
    let name_filter = query.name.as_deref().filter(|s| !s.is_empty());

    let metrics = state
        .storage
        .query_range(source_filter, name_filter, start, now, Some(query.limit))
        .await
        .unwrap_or_default();

    let json_metrics: Vec<serde_json::Value> = metrics
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "source": m.source.to_string(),
                "name": m.name,
                "value": m.value,
                "timestamp": m.timestamp,
                "labels": m.labels,
            })
        })
        .collect();

    axum::Json(json_metrics)
}

// =============================================================================
// Helpers
// =============================================================================

/// Fetch metrics from storage with filters applied.
async fn fetch_filtered_metrics(state: &AppState, query: &MetricsQuery) -> Vec<MetricView> {
    let now = chrono::Utc::now().timestamp();
    let start = now - query.time_range_seconds();

    let source_filter = query.source.as_deref().filter(|s| !s.is_empty());
    let name_filter = query.name.as_deref().filter(|s| !s.is_empty());

    state
        .storage
        .query_range(source_filter, name_filter, start, now, Some(query.limit))
        .await
        .unwrap_or_default()
        .into_iter()
        .map(MetricView::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::local::{LocalStorage, LocalStorageConfig};
    use crate::{DataSource, Metric, MetricUnit};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_index_metadata_fallback() {
        // 1. Setup storage with some data
        let storage = LocalStorage::new_in_memory(LocalStorageConfig::default())
            .await
            .unwrap();

        let metric = Metric::new(
            DataSource::AlternativeMe,
            "test_fallback",
            100.0,
            MetricUnit::Index,
        );
        storage.store(&[metric]).await.unwrap();

        // 2. Setup state with EMPTY cache
        let state = AppState {
            storage: Arc::new(storage),
            metadata_cache: Arc::new(RwLock::new(Vec::new())),
        };

        // 3. Call index handler
        let query = MetricsQuery::default();
        // Since we can't easily inspect IntoResponse, we just verify side effects on the cache
        let _ = index(State(state.clone()), Query(query)).await;

        // 4. Verify cache is populated
        let cache = state.metadata_cache.read().await;
        assert!(!cache.is_empty(), "Cache should be populated from storage");
        assert_eq!(cache.len(), 1);
        assert_eq!(cache[0].0, "alternativeme");
        assert_eq!(cache[0].1, "test_fallback");
    }
}
