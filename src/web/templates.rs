//! HTML templates for the web dashboard.

use askama::Template;
use askama_web::WebTemplate;
use chrono::{TimeZone, Utc};

use crate::storage::Metric;

/// Format a unix timestamp to UTC string with explicit UTC suffix.
fn format_utc_time(timestamp: i64, fmt: &str) -> String {
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .map(|dt| format!("{} UTC", dt.format(fmt)))
        .unwrap_or_else(|| timestamp.to_string())
}

// =============================================================================
// Filter Parameters
// =============================================================================

/// Filter parameters for templates.
#[derive(Debug, Clone, Default)]
pub struct FilterParams {
    pub source: String,
    pub name: String,
    pub time_range: String,
}

impl FilterParams {
    /// Check if a time range option is selected.
    pub fn is_time_range(&self, value: &str) -> bool {
        self.time_range == value || (self.time_range.is_empty() && value == "1h")
    }
}

// =============================================================================
// Templates
// =============================================================================

/// Dashboard page template - displays statistics overview.
#[derive(Template, WebTemplate)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub title: String,
    pub total_metrics: usize,
    pub total_status: usize,
    pub total_events: usize,
    pub source_stats: Vec<SourceStat>,
}

/// Statistics for a data source.
pub struct SourceStat {
    pub source: String,
    pub metric_count: usize,
    pub name_count: usize,
    pub last_updated: String,
}

impl Default for DashboardTemplate {
    fn default() -> Self {
        Self {
            title: "Dashboard".to_string(),
            total_metrics: 0,
            total_status: 0,
            total_events: 0,
            source_stats: Vec::new(),
        }
    }
}

/// Metrics page template - displays metrics list with filters.
#[derive(Template, WebTemplate)]
#[template(path = "metrics.html")]
pub struct MetricsTemplate {
    pub title: String,
    pub filter_params: FilterParams,
    pub available_sources: Vec<String>,
    pub available_names: Vec<String>,
}

impl Default for MetricsTemplate {
    fn default() -> Self {
        Self {
            title: "Metrics Explorer".to_string(),
            filter_params: FilterParams::default(),
            available_sources: Vec::new(),
            available_names: Vec::new(),
        }
    }
}

/// Metrics partial template for htmx updates.
#[derive(Template, WebTemplate)]
#[template(path = "partials/metrics.html")]
pub struct MetricsPartialTemplate {
    pub metrics: Vec<MetricView>,
    pub filter_params: FilterParams,
}

/// Status page template - shows latest value of each metric.
#[derive(Template, WebTemplate)]
#[template(path = "status.html")]
pub struct StatusTemplate {
    pub title: String,
    pub metrics: Vec<LatestMetricView>,
    pub last_updated: String,
}

/// View model for the latest metric value.
pub struct LatestMetricView {
    pub source: String,
    pub name: String,
    pub value: String,
    pub unit: String,
    pub timestamp: String,
    pub age_seconds: i64,
}

// =============================================================================
// View Models
// =============================================================================

/// View model for a single metric.
pub struct MetricView {
    pub source: String,
    pub name: String,
    pub value: String,
    pub unit: String,
    pub timestamp: String,
    pub timestamp_raw: i64,
    pub labels: Vec<(String, String)>,
}

impl From<Metric> for MetricView {
    fn from(m: Metric) -> Self {
        let timestamp = format_utc_time(m.timestamp, "%Y-%m-%d %H:%M:%S");

        Self {
            source: m.source.to_string(),
            name: m.name,
            value: format!("{:.4}", m.value),
            unit: m.unit.to_string(),
            timestamp,
            timestamp_raw: m.timestamp,
            labels: m.labels.into_iter().collect(),
        }
    }
}

// =============================================================================
// Events
// =============================================================================

/// Events page template - shows system events log.
#[derive(Template, WebTemplate)]
#[template(path = "events.html")]
pub struct EventsTemplate {
    pub title: String,
    pub events: Vec<EventView>,
    pub instance_id: String,
    pub available_instances: Vec<String>,
    pub filter_instance: Option<String>,
}

/// View model for a single event.
pub struct EventView {
    pub id: i64,
    pub instance_id: String,
    pub event_type: String,
    pub event_type_class: String,
    pub message: String,
    pub payload: Option<String>,
    pub timestamp: String,
}

impl EventView {
    /// Creates an EventView from an Event.
    pub fn from_event(event: crate::storage::Event) -> Self {
        let timestamp = format_utc_time(event.timestamp, "%Y-%m-%d %H:%M:%S");

        let event_type_class = match event.event_type {
            crate::storage::EventType::ServiceStart => "bg-green-500/20 text-green-300",
            crate::storage::EventType::ServiceStop => "bg-red-500/20 text-red-300",
            crate::storage::EventType::TaskScheduled => "bg-blue-500/20 text-blue-300",
            crate::storage::EventType::TaskExecuted => "bg-emerald-500/20 text-emerald-300",
            crate::storage::EventType::TaskFailed | crate::storage::EventType::Error => {
                "bg-orange-500/20 text-orange-300"
            }
        };

        let payload = event
            .payload
            .map(|p| serde_json::to_string_pretty(&p).unwrap_or_default());

        Self {
            id: event.id.unwrap_or(0),
            instance_id: event.instance_id,
            event_type: event.event_type.to_string(),
            event_type_class: event_type_class.to_string(),
            message: event.message,
            payload,
            timestamp,
        }
    }
}
