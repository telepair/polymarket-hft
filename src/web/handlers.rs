//! HTTP request handlers for the web dashboard.

use std::sync::Arc;

use axum::{
    Form, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};

use serde::Deserialize;

use crate::client::DataSourceClient;
use crate::scheduler::{self, SchedulerHandle};
use crate::storage::{Event, EventType, StorageBackend};
use tokio::sync::RwLock;
use tracing;

use super::templates::{
    DashboardTemplate, EventView, EventsTemplate, FilterParams, JobFormDataView, JobFormTemplate,
    JobView, JobsTemplate, LatestMetricView, MetricView, MetricsPartialTemplate, MetricsTemplate,
    SourceStat, StatusTemplate,
};

// =============================================================================
// Constants
// =============================================================================

/// Default limit for events queries.
const DEFAULT_EVENTS_LIMIT: usize = 100;

/// Format a unix timestamp in milliseconds to UTC string with explicit UTC suffix.
fn format_utc_time_millis(timestamp_ms: i64, fmt: &str) -> String {
    chrono::DateTime::from_timestamp_millis(timestamp_ms)
        .map(|dt| format!("{} UTC", dt.format(fmt)))
        .unwrap_or_else(|| timestamp_ms.to_string())
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
    pub client: Arc<dyn DataSourceClient>,
    pub scheduler: SchedulerHandle,
}

/// Create the Axum router with all routes.
pub fn create_router(
    storage: Arc<dyn StorageBackend>,
    metadata_cache: Arc<RwLock<Vec<(String, String)>>>,
    instance_id: String,
    client: Arc<dyn DataSourceClient>,
    scheduler: SchedulerHandle,
) -> Router {
    let state = AppState {
        storage,
        metadata_cache,
        instance_id,
        client,
        scheduler,
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
        .route("/jobs", get(jobs_page))
        .route("/partials/metrics", get(metrics_partial))
        // API routes
        .route("/api/metrics/latest", get(api_metrics_latest))
        .route("/api/jobs", post(api_create_job))
        .route(
            "/api/jobs/{id}",
            get(api_get_job).put(api_update_job).delete(api_delete_job),
        )
        .route("/api/jobs/{id}/trigger", post(api_trigger_job))
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

    // Calculate source stats: group metrics by source (timestamps in milliseconds)
    let now = chrono::Utc::now().timestamp_millis();
    let start_24h = now - 24 * 3600 * 1000;

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
    let all_metrics = match state
        .storage
        .query_range(None, None, start_24h, now, Some(10000))
        .await
    {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to query metrics for dashboard stats");
            Vec::new()
        }
    };

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
                format_utc_time_millis(last_ts, "%Y-%m-%d %H:%M")
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

    let events = match state
        .storage
        .get_events(filter_instance, Some(query.limit))
        .await
    {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to load events");
            Vec::new()
        }
    };

    // Get unique instance IDs using SQL DISTINCT
    let mut available_instances = match state.storage.get_distinct_instance_ids().await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to get distinct instance IDs");
            Vec::new()
        }
    };

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
    let now = chrono::Utc::now().timestamp_millis();
    let start = now - query.time_range_seconds() * 1000;

    let source_filter = query.source.as_deref().filter(|s| !s.is_empty());
    let name_filter = query.name.as_deref().filter(|s| !s.is_empty());

    let metrics = match state
        .storage
        .query_range(source_filter, name_filter, start, now, Some(query.limit))
        .await
    {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(error = %e, "Failed to query metrics for API");
            return axum::Json(vec![serde_json::json!({
                "error": "Database query failed",
                "message": e.to_string()
            })]);
        }
    };

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
// Jobs Handlers
// =============================================================================

/// Jobs page - renders the job management interface.
async fn jobs_page(State(state): State<AppState>) -> impl IntoResponse {
    let jobs = match state.storage.list_jobs().await {
        Ok(j) => j.into_iter().map(JobView::from_record).collect(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to load jobs");
            Vec::new()
        }
    };

    // Generate methods JSON dynamically from the client
    let methods_json = super::templates::generate_methods_json(state.client.as_ref());

    JobsTemplate {
        title: "Job Management".to_string(),
        jobs,
        methods_json,
        ..JobsTemplate::default()
    }
}

/// Form data for creating a new job.
#[derive(Debug, Deserialize)]
pub struct JobFormData {
    pub name: String,
    pub datasource: String,
    pub method: String,
    pub schedule_type: String,
    pub schedule_value: String,
    #[serde(default)]
    pub params: Option<String>,
    #[serde(default = "default_retention")]
    pub retention_days: u32,
    #[serde(default)]
    pub enabled: Option<String>,
}

fn default_retention() -> u32 {
    7
}

/// API endpoint to create a new job.
async fn api_create_job(
    State(state): State<AppState>,
    Form(form): Form<JobFormData>,
) -> impl IntoResponse {
    // Helper to create the error response (just the error div HTML)
    let render_error = |error_msg: String| {
        axum::response::Html(format!(
            r#"<div class="p-3 mb-4 text-sm text-red-200 bg-red-500/20 rounded-lg border border-red-500/30">{}</div>"#,
            error_msg
        ))
    };

    // Parse datasource
    let datasource: crate::DataSource = match form.datasource.parse() {
        Ok(ds) => ds,
        Err(_) => return render_error("Invalid datasource".to_string()).into_response(),
    };

    // Parse schedule
    let schedule = if form.schedule_type == "cron" {
        crate::config::Schedule::Cron {
            cron: form.schedule_value.clone(),
        }
    } else {
        match form.schedule_value.parse::<u64>() {
            Ok(secs) => crate::config::Schedule::Interval {
                interval_secs: secs,
            },
            Err(_) => {
                return render_error("Invalid interval value. Must be a number.".to_string())
                    .into_response();
            }
        }
    };

    // Parse params
    let params: Option<serde_json::Value> = match form.params.as_ref().filter(|s| !s.is_empty()) {
        Some(p) => match serde_json::from_str(p) {
            Ok(v) => Some(v),
            Err(e) => return render_error(format!("Invalid JSON params: {}", e)).into_response(),
        },
        None => None,
    };

    let job = crate::config::IngestionJob {
        name: form.name.clone(),
        datasource,
        method: form.method.clone(),
        schedule,
        params,
        retention_days: form.retention_days,
        enabled: form.enabled.is_some(),
    };

    // Validate job configuration before storing
    if let Err(e) = job.validate() {
        return render_error(format!("Invalid job configuration: {}", e)).into_response();
    }

    match state.storage.store_job(&job).await {
        Ok(id) => {
            tracing::info!(job_name = %job.name, job_id = id, "Created new job");

            // Schedule the job if enabled
            if job.enabled
                && let Err(e) = state.scheduler.schedule_job(id, &job).await
            {
                tracing::error!(
                    error = %e,
                    job_id = id,
                    "Failed to schedule job - will be scheduled on next restart"
                );
            }

            // Record JobCreated event
            let event = Event::new(
                &state.instance_id,
                EventType::JobCreated,
                format!("Job '{}' created", job.name),
            )
            .with_payload(serde_json::json!({
                "job_id": id,
                "job_name": job.name,
                "datasource": job.datasource.to_string(),
                "method": job.method,
            }));
            if let Err(e) = state.storage.store_event(&event).await {
                tracing::warn!(error = %e, "Failed to record job created event");
            }

            // Success response with HX-Trigger to refresh the table and close modal
            (StatusCode::OK, [("HX-Trigger", "jobCreated")], "").into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create job");
            render_error(format!("Database error: {}", e)).into_response()
        }
    }
}

/// API endpoint to delete a job by ID.
async fn api_delete_job(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    // Get job info before deletion for event logging
    let job_name = match state.storage.get_job(id).await {
        Ok(Some(record)) => Some(record.job.name),
        _ => None,
    };

    // Unschedule the job first
    if let Err(e) = state.scheduler.unschedule_job(id).await {
        tracing::warn!(
            error = %e,
            job_id = id,
            "Failed to unschedule job (may not have been scheduled)"
        );
    }

    match state.storage.delete_job(id).await {
        Ok(()) => {
            tracing::info!(job_id = id, "Deleted job");

            // Record JobDeleted event
            let message = match &job_name {
                Some(name) => format!("Job '{}' deleted", name),
                None => format!("Job {} deleted", id),
            };
            let event = Event::new(&state.instance_id, EventType::JobDeleted, message)
                .with_payload(serde_json::json!({
                    "job_id": id,
                    "job_name": job_name,
                }));
            if let Err(e) = state.storage.store_event(&event).await {
                tracing::warn!(error = %e, "Failed to record job deleted event");
            }

            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::error!(job_id = id, error = %e, "Failed to delete job");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to delete job: {}", e),
            )
                .into_response()
        }
    }
}

/// API endpoint to get a job by ID (JSON response for edit form).
async fn api_get_job(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    match state.storage.get_job(id).await {
        Ok(Some(record)) => {
            let (schedule_type, schedule_value) = match &record.job.schedule {
                crate::config::Schedule::Interval { interval_secs } => {
                    ("interval".to_string(), interval_secs.to_string())
                }
                crate::config::Schedule::Cron { cron } => ("cron".to_string(), cron.clone()),
            };

            let params = record
                .job
                .params
                .map(|p| serde_json::to_string(&p).unwrap_or_default());

            let job_data = serde_json::json!({
                "id": record.id,
                "name": record.job.name,
                "datasource": record.job.datasource.to_string(),
                "method": record.job.method,
                "schedule_type": schedule_type,
                "schedule_value": schedule_value,
                "params": params,
                "retention_days": record.job.retention_days,
                "enabled": record.job.enabled,
            });

            axum::Json(job_data).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Job not found").into_response(),
        Err(e) => {
            tracing::error!(job_id = id, error = %e, "Failed to get job");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get job: {}", e),
            )
                .into_response()
        }
    }
}

/// API endpoint to update an existing job.
async fn api_update_job(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<JobFormData>,
) -> impl IntoResponse {
    // Helper to create the error response
    let render_error = |error_msg: String| {
        let form_view = JobFormDataView {
            name: form.name.clone(),
            datasource: form.datasource.clone(),
            method: form.method.clone(),
            schedule_type: form.schedule_type.clone(),
            schedule_value: form.schedule_value.clone(),
            params: form.params.clone(),
            retention_days: form.retention_days,
            enabled: form.enabled.clone(),
        };

        JobFormTemplate {
            form: form_view,
            error: Some(error_msg),
            ..Default::default()
        }
    };

    // Parse datasource
    let datasource: crate::DataSource = match form.datasource.parse() {
        Ok(ds) => ds,
        Err(_) => return render_error("Invalid datasource".to_string()).into_response(),
    };

    // Parse schedule
    let schedule = if form.schedule_type == "cron" {
        crate::config::Schedule::Cron {
            cron: form.schedule_value.clone(),
        }
    } else {
        match form.schedule_value.parse::<u64>() {
            Ok(secs) => crate::config::Schedule::Interval {
                interval_secs: secs,
            },
            Err(_) => {
                return render_error("Invalid interval value. Must be a number.".to_string())
                    .into_response();
            }
        }
    };

    // Parse params
    let params: Option<serde_json::Value> = match form.params.as_ref().filter(|s| !s.is_empty()) {
        Some(p) => match serde_json::from_str(p) {
            Ok(v) => Some(v),
            Err(e) => return render_error(format!("Invalid JSON params: {}", e)).into_response(),
        },
        None => None,
    };

    let job = crate::config::IngestionJob {
        name: form.name.clone(),
        datasource,
        method: form.method.clone(),
        schedule,
        params,
        retention_days: form.retention_days,
        enabled: form.enabled.is_some(),
    };

    // Validate job configuration before updating
    if let Err(e) = job.validate() {
        return render_error(format!("Invalid job configuration: {}", e)).into_response();
    }

    match state.storage.update_job(id, &job).await {
        Ok(()) => {
            tracing::info!(job_name = %job.name, job_id = id, "Updated job");

            // Reschedule the job (remove old + add new if enabled)
            if let Err(e) = state.scheduler.reschedule_job(id, &job).await {
                tracing::error!(
                    error = %e,
                    job_id = id,
                    "Failed to reschedule job - changes will apply on next restart"
                );
            }

            // Record JobUpdated event
            let event = Event::new(
                &state.instance_id,
                EventType::JobUpdated,
                format!("Job '{}' updated", job.name),
            )
            .with_payload(serde_json::json!({
                "job_id": id,
                "job_name": job.name,
                "datasource": job.datasource.to_string(),
                "method": job.method,
                "enabled": job.enabled,
            }));
            if let Err(e) = state.storage.store_event(&event).await {
                tracing::warn!(error = %e, "Failed to record job updated event");
            }

            // Success response with HX-Trigger to refresh the table and close modal
            (StatusCode::OK, [("HX-Trigger", "jobUpdated")], "").into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to update job");
            render_error(format!("Database error: {}", e)).into_response()
        }
    }
}

/// API endpoint to manually trigger a job execution.
async fn api_trigger_job(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    // Get job from database
    let job_record = match state.storage.get_job(id).await {
        Ok(Some(record)) => record,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Job not found".to_string()).into_response();
        }
        Err(e) => {
            tracing::error!(job_id = id, error = %e, "Failed to get job");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get job: {}", e),
            )
                .into_response();
        }
    };

    // Execute job immediately
    scheduler::trigger_job(
        &job_record.job,
        state.scheduler.client(),
        state.scheduler.storage(),
        state.scheduler.instance_id(),
    )
    .await;

    tracing::info!(job_id = id, job_name = %job_record.job.name, "Triggered job execution");

    // Redirect to jobs page
    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/jobs")],
        String::new(),
    )
        .into_response()
}

// =============================================================================
// Helpers
// =============================================================================

/// Fetch metrics from storage with filters applied.
async fn fetch_filtered_metrics(state: &AppState, query: &MetricsQuery) -> Vec<MetricView> {
    let now = chrono::Utc::now().timestamp_millis();
    let start = now - query.time_range_seconds() * 1000;

    let source_filter = query.source.as_deref().filter(|s| !s.is_empty());
    let name_filter = query.name.as_deref().filter(|s| !s.is_empty());

    match state
        .storage
        .query_range(source_filter, name_filter, start, now, Some(query.limit))
        .await
    {
        Ok(metrics) => metrics.into_iter().map(MetricView::from).collect(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to fetch filtered metrics");
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::alternativeme::Client as AlternativeMeClient;
    use crate::storage::local::{LocalStorage, LocalStorageConfig};
    use crate::{DataSource, Metric, MetricUnit, SchedulerHandle};
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
        let client = Arc::new(AlternativeMeClient::new());
        let storage_arc = Arc::new(storage);
        let scheduler = SchedulerHandle::new(
            client.clone(),
            storage_arc.clone(),
            "test-instance".to_string(),
        )
        .await
        .unwrap();

        let state = AppState {
            storage: storage_arc,
            metadata_cache: Arc::new(RwLock::new(Vec::new())),
            instance_id: "test-instance".to_string(),
            client,
            scheduler,
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
