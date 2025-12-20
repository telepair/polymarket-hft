//! Route handlers for the web UI.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::AppState;
use super::templates;
use crate::engine::{Schedule, ScrapeJob, Target};
use crate::storage::timescale::ScrapeJobRow;

/// Redirect root to metrics page.
pub async fn index() -> Redirect {
    Redirect::to("/metrics")
}

/// Render the metrics dashboard page.
pub async fn metrics_page() -> Html<String> {
    Html(templates::render_metrics_page())
}

/// Render the state viewer page.
pub async fn state_page() -> Html<String> {
    Html(templates::render_state_page())
}

/// Render the configuration page.
pub async fn config_page() -> Html<String> {
    Html(templates::render_config_page())
}

/// Query parameters for metrics API.
#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub source: Option<String>,
    pub name: Option<String>,
    pub hours: Option<i64>,
}

/// API response for metrics.
#[derive(Debug, Serialize)]
pub struct MetricResponse {
    pub time: String,
    pub source: String,
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

/// API endpoint for fetching metrics.
pub async fn api_metrics(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> Json<Vec<MetricResponse>> {
    let hours = query.hours.unwrap_or(24);
    let start = Utc::now() - Duration::hours(hours);
    let end = Utc::now();

    if let Some(timescale) = &state.timescale
        && let (Some(source), Some(name)) = (&query.source, &query.name)
    {
        match timescale
            .query_metrics(source, name, start, end, Some(100))
            .await
        {
            Ok(metrics) => {
                let response: Vec<MetricResponse> = metrics
                    .into_iter()
                    .map(|m| MetricResponse {
                        time: m.time.to_rfc3339(),
                        source: m.source,
                        name: m.name,
                        value: m.value,
                        labels: m.labels,
                    })
                    .collect();
                return Json(response);
            }
            Err(e) => {
                tracing::error!("Failed to query metrics: {}", e);
            }
        }
    }

    Json(vec![])
}

/// API response for state entries.
#[derive(Debug, Serialize)]
pub struct StateResponse {
    pub key: String,
    pub value: serde_json::Value,
    pub ttl: Option<i64>,
}

/// API endpoint for fetching current state.
pub async fn api_state(State(state): State<AppState>) -> Json<Vec<StateResponse>> {
    if let Some(redis) = &state.redis {
        match redis.get_states_by_pattern("state:*").await {
            Ok(states) => {
                let mut response = Vec::new();
                for (key, value) in states {
                    let ttl = redis.get_ttl(&key).await.ok().flatten();
                    response.push(StateResponse { key, value, ttl });
                }
                return Json(response);
            }
            Err(e) => {
                tracing::error!("Failed to query state: {}", e);
            }
        }
    }

    Json(vec![])
}

// --- Scrape Job Configuration API ---

/// Request to create or update a scrape job.
#[derive(Debug, Deserialize)]
pub struct ScrapeJobRequest {
    pub id: String,
    pub source: String,
    pub endpoint: String,
    #[serde(default)]
    pub params: serde_json::Value,
    pub targets: Vec<String>,
    pub schedule_type: String,
    pub schedule_value: String,
    pub state_ttl_secs: Option<i64>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl From<ScrapeJobRequest> for ScrapeJob {
    fn from(req: ScrapeJobRequest) -> Self {
        let targets = req
            .targets
            .iter()
            .filter_map(|t| match t.as_str() {
                "metrics" => Some(Target::Metrics),
                "state" => Some(Target::State),
                _ => None,
            })
            .collect();

        let schedule = match req.schedule_type.as_str() {
            "cron" => Schedule::Cron {
                expression: req.schedule_value,
            },
            _ => Schedule::Interval {
                interval: humantime::parse_duration(&req.schedule_value)
                    .unwrap_or(std::time::Duration::from_secs(300)),
            },
        };

        ScrapeJob {
            id: req.id,
            source: req.source,
            endpoint: req.endpoint,
            params: req.params,
            targets,
            schedule,
            state_ttl: req
                .state_ttl_secs
                .map(|s| std::time::Duration::from_secs(s as u64)),
            enabled: req.enabled,
        }
    }
}

/// API response for scrape jobs.
#[derive(Debug, Serialize)]
pub struct ScrapeJobResponse {
    pub id: String,
    pub source: String,
    pub endpoint: String,
    pub params: serde_json::Value,
    pub targets: Vec<String>,
    pub schedule_type: String,
    pub schedule_value: String,
    pub state_ttl_secs: Option<i64>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ScrapeJobRow> for ScrapeJobResponse {
    fn from(row: ScrapeJobRow) -> Self {
        Self {
            id: row.id,
            source: row.source,
            endpoint: row.endpoint,
            params: row.params,
            targets: row.targets,
            schedule_type: row.schedule_type,
            schedule_value: row.schedule_value,
            state_ttl_secs: row.state_ttl_secs,
            enabled: row.enabled,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

/// API endpoint for listing scrape configurations.
pub async fn api_config_list(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(timescale) = &state.timescale {
        match timescale.get_scrape_jobs().await {
            Ok(jobs) => {
                let response: Vec<ScrapeJobResponse> = jobs.into_iter().map(Into::into).collect();
                return (StatusCode::OK, Json(response)).into_response();
            }
            Err(e) => {
                tracing::error!("Failed to list scrape jobs: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        }
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Database not configured"})),
    )
        .into_response()
}

/// API endpoint for creating a scrape configuration.
pub async fn api_config_create(
    State(state): State<AppState>,
    Json(req): Json<ScrapeJobRequest>,
) -> impl IntoResponse {
    if let Some(timescale) = &state.timescale {
        let job: ScrapeJob = req.into();
        match timescale.create_scrape_job(&job).await {
            Ok(row) => {
                let response: ScrapeJobResponse = row.into();
                return (StatusCode::CREATED, Json(response)).into_response();
            }
            Err(e) => {
                tracing::error!("Failed to create scrape job: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        }
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Database not configured"})),
    )
        .into_response()
}

/// API endpoint for getting a single scrape configuration.
pub async fn api_config_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Some(timescale) = &state.timescale {
        match timescale.get_scrape_job(&id).await {
            Ok(Some(row)) => {
                let response: ScrapeJobResponse = row.into();
                return (StatusCode::OK, Json(response)).into_response();
            }
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Job not found"})),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!("Failed to get scrape job: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        }
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Database not configured"})),
    )
        .into_response()
}

/// API endpoint for updating a scrape configuration.
pub async fn api_config_update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ScrapeJobRequest>,
) -> impl IntoResponse {
    if let Some(timescale) = &state.timescale {
        let job: ScrapeJob = req.into();
        match timescale.update_scrape_job(&id, &job).await {
            Ok(Some(row)) => {
                let response: ScrapeJobResponse = row.into();
                return (StatusCode::OK, Json(response)).into_response();
            }
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Job not found"})),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!("Failed to update scrape job: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        }
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Database not configured"})),
    )
        .into_response()
}

/// API endpoint for deleting a scrape configuration.
pub async fn api_config_delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Some(timescale) = &state.timescale {
        match timescale.delete_scrape_job(&id).await {
            Ok(true) => {
                return StatusCode::NO_CONTENT.into_response();
            }
            Ok(false) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Job not found"})),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!("Failed to delete scrape job: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        }
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Database not configured"})),
    )
        .into_response()
}
