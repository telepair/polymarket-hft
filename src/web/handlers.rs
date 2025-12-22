//! HTTP request handlers for the web dashboard.

use std::sync::Arc;

use axum::{
    Router,
    extract::{Query, State},
    http::header,
    response::IntoResponse,
    routing::get,
};
use chrono::{TimeZone, Utc};
use serde::Deserialize;

use crate::storage::StorageBackend;
use tokio::sync::RwLock;
use tracing;

use super::templates::{
    DashboardTemplate, EventView, EventsTemplate, FilterParams, LatestMetricView, MetricView,
    MetricsPartialTemplate, MetricsTemplate, SourceStat, StatusTemplate,
};

// =============================================================================
// Constants
// =============================================================================

/// Default limit for events queries.
const DEFAULT_EVENTS_LIMIT: usize = 100;

/// Format a unix timestamp to UTC string with explicit UTC suffix.
fn format_utc_time(timestamp: i64, fmt: &str) -> String {
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .map(|dt| format!("{} UTC", dt.format(fmt)))
        .unwrap_or_else(|| timestamp.to_string())
}

// =============================================================================
// Static Assets (embedded at compile time for offline use)
// =============================================================================

const HTMX_JS: &str = include_str!("../../templates/htmx.min.js");
const STYLES_CSS: &str = include_str!("../../templates/styles.css");

async fn serve_htmx() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/javascript")], HTMX_JS)
}

async fn serve_styles() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css")], STYLES_CSS)
}

// =============================================================================
// Router
// =============================================================================

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<dyn StorageBackend>,
    pub metadata_cache: Arc<RwLock<Vec<(String, String)>>>,
    pub instance_id: String,
}

/// Create the Axum router with all routes.
pub fn create_router(
    storage: Arc<dyn StorageBackend>,
    metadata_cache: Arc<RwLock<Vec<(String, String)>>>,
    instance_id: String,
) -> Router {
    let state = AppState {
        storage,
        metadata_cache,
        instance_id,
    };

    Router::new()
        // Static assets
        .route("/static/htmx.min.js", get(serve_htmx))
        .route("/static/styles.css", get(serve_styles))
        // Pages
        .route("/", get(index))
        .route("/metrics", get(metrics_page))
        .route("/status", get(status))
        .route("/events", get(events))
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

/// Index page - renders the dashboard with statistics overview.
async fn index(State(state): State<AppState>) -> impl IntoResponse {
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

    // Calculate statistics
    let total_status = available_metrics.len();

    // Get events count
    let total_events = state
        .storage
        .get_events(None, Some(1000))
        .await
        .map(|e| e.len())
        .unwrap_or(0);

    // Calculate source stats: group metrics by source
    let now = chrono::Utc::now().timestamp();
    let start_24h = now - 24 * 3600;

    let mut source_map: std::collections::HashMap<
        String,
        (usize, std::collections::HashSet<String>, i64),
    > = std::collections::HashMap::new();

    for (source, name) in &available_metrics {
        let entry =
            source_map
                .entry(source.clone())
                .or_insert((0, std::collections::HashSet::new(), 0));
        entry.1.insert(name.clone());

        // Get latest timestamp for this source
        if let Ok(Some(metric)) = state.storage.get_latest(source, name).await
            && metric.timestamp > entry.2
        {
            entry.2 = metric.timestamp;
        }
    }

    // Count metrics in last 24h for each source
    let all_metrics = state
        .storage
        .query_range(None, None, start_24h, now, Some(10000))
        .await
        .unwrap_or_default();

    let total_metrics = all_metrics.len();

    for metric in &all_metrics {
        let source = metric.source.to_string();
        if let Some(entry) = source_map.get_mut(&source) {
            entry.0 += 1;
        }
    }

    let mut source_stats: Vec<SourceStat> = source_map
        .into_iter()
        .map(|(source, (metric_count, names, last_ts))| {
            let last_updated = if last_ts > 0 {
                format_utc_time(last_ts, "%Y-%m-%d %H:%M")
            } else {
                "-".to_string()
            };
            SourceStat {
                source,
                metric_count,
                name_count: names.len(),
                last_updated,
            }
        })
        .collect();

    source_stats.sort_by(|a, b| a.source.cmp(&b.source));

    DashboardTemplate {
        title: "Dashboard".to_string(),
        total_metrics,
        total_status,
        total_events,
        source_stats,
    }
}

/// Metrics page - renders the metrics explorer with filter form.
async fn metrics_page(
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

    MetricsTemplate {
        title: "Metrics Explorer".to_string(),
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

/// Query parameters for events page.
#[derive(Debug, Deserialize, Default)]
pub struct EventsQuery {
    /// Filter by instance ID.
    #[serde(default)]
    pub instance_id: Option<String>,

    /// Maximum number of events.
    #[serde(default = "default_events_limit")]
    pub limit: usize,
}

fn default_events_limit() -> usize {
    DEFAULT_EVENTS_LIMIT
}

/// Events page - shows system events log.
async fn events(
    State(state): State<AppState>,
    Query(query): Query<EventsQuery>,
) -> impl IntoResponse {
    let filter_instance = query.instance_id.as_deref().filter(|s| !s.is_empty());

    let events = state
        .storage
        .get_events(filter_instance, Some(query.limit))
        .await
        .unwrap_or_default();

    // Get unique instance IDs using SQL DISTINCT
    let mut available_instances = state
        .storage
        .get_distinct_instance_ids()
        .await
        .unwrap_or_default();

    // Ensure current instance is in the list
    if !available_instances.contains(&state.instance_id) {
        available_instances.insert(0, state.instance_id.clone());
    }

    let event_views: Vec<EventView> = events.into_iter().map(EventView::from_event).collect();

    EventsTemplate {
        title: "System Events".to_string(),
        events: event_views,
        instance_id: state.instance_id.clone(),
        available_instances,
        filter_instance: query.instance_id,
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
            instance_id: "test-instance".to_string(),
        };

        // 3. Call index handler (now only takes State, no Query)
        // Since we can't easily inspect IntoResponse, we just verify side effects on the cache
        let _ = index(State(state.clone())).await;

        // 4. Verify cache is populated
        let cache = state.metadata_cache.read().await;
        assert!(!cache.is_empty(), "Cache should be populated from storage");
        assert_eq!(cache.len(), 1);
        assert_eq!(cache[0].0, "alternativeme");
        assert_eq!(cache[0].1, "test_fallback");
    }
}
