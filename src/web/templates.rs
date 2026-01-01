//! HTML templates for the web dashboard.

use askama::Template;
use askama_web::WebTemplate;
use chrono::{TimeZone, Utc};

use crate::storage::Metric;

/// Format a unix timestamp in **seconds** to UTC string with explicit UTC suffix.
/// Used for JobRecord.created_at/updated_at which are stored in seconds.
fn format_utc_time_seconds(timestamp: i64, fmt: &str) -> String {
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .map(|dt| format!("{} UTC", dt.format(fmt)))
        .unwrap_or_else(|| timestamp.to_string())
}

/// Format a unix timestamp in **milliseconds** to UTC string with explicit UTC suffix.
/// Used for Metric.timestamp and Event.timestamp which are stored in milliseconds.
fn format_utc_time_millis(timestamp_ms: i64, fmt: &str) -> String {
    chrono::DateTime::from_timestamp_millis(timestamp_ms)
        .map(|dt| format!("{} UTC", dt.format(fmt)))
        .unwrap_or_else(|| timestamp_ms.to_string())
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
        let timestamp = format_utc_time_millis(m.timestamp, "%Y-%m-%d %H:%M:%S");

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
        let timestamp = format_utc_time_millis(event.timestamp, "%Y-%m-%d %H:%M:%S");

        let event_type_class = match event.event_type {
            crate::storage::EventType::ServiceStart => "bg-green-500/20 text-green-300",
            crate::storage::EventType::ServiceStop => "bg-red-500/20 text-red-300",
            crate::storage::EventType::TaskScheduled => "bg-blue-500/20 text-blue-300",
            crate::storage::EventType::TaskExecuted => "bg-emerald-500/20 text-emerald-300",
            crate::storage::EventType::TaskFailed | crate::storage::EventType::Error => {
                "bg-orange-500/20 text-orange-300"
            }
            crate::storage::EventType::JobCreated => "bg-purple-500/20 text-purple-300",
            crate::storage::EventType::JobUpdated => "bg-indigo-500/20 text-indigo-300",
            crate::storage::EventType::JobDeleted => "bg-pink-500/20 text-pink-300",
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

// =============================================================================
// Jobs
// =============================================================================

/// Jobs page template - shows job management interface.
/// Jobs page template - shows job management interface.
#[derive(Template, WebTemplate)]
#[template(path = "jobs.html")]
pub struct JobsTemplate {
    pub title: String,
    pub jobs: Vec<JobView>,
    pub available_datasources: Vec<String>,
    pub methods_json: String,
    pub form: JobFormDataView,
    pub error: Option<String>,
}

impl Default for JobsTemplate {
    fn default() -> Self {
        Self {
            title: "Job Management".to_string(),
            jobs: Vec::new(),
            available_datasources: vec![
                "alternativeme".to_string(),
                "coingecko".to_string(),
                "coinmarketcap".to_string(),
                "polymarket".to_string(),
            ],
            methods_json: default_methods_json(),
            form: JobFormDataView::default(),
            error: None,
        }
    }
}

/// View model for a single job.
pub struct JobView {
    pub id: i64,
    pub name: String,
    pub datasource: String,
    pub method: String,
    pub schedule: String,
    pub schedule_type: String,
    pub params: Option<String>,
    pub retention_days: u32,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl JobView {
    /// Creates a JobView from a JobRecord.
    pub fn from_record(record: crate::storage::JobRecord) -> Self {
        // JobRecord timestamps are in seconds (from SQLite strftime('%s', 'now'))
        let created_at = format_utc_time_seconds(record.created_at, "%Y-%m-%d %H:%M:%S");
        let updated_at = format_utc_time_seconds(record.updated_at, "%Y-%m-%d %H:%M:%S");

        let (schedule, schedule_type) = match &record.job.schedule {
            crate::config::Schedule::Interval { interval_secs } => {
                (format!("{}s", interval_secs), "interval".to_string())
            }
            crate::config::Schedule::Cron { cron } => (cron.clone(), "cron".to_string()),
        };

        let params = record
            .job
            .params
            .map(|p| serde_json::to_string_pretty(&p).unwrap_or_default());

        Self {
            id: record.id,
            name: record.job.name,
            datasource: record.job.datasource.to_string(),
            method: record.job.method,
            schedule,
            schedule_type,
            params,
            retention_days: record.job.retention_days,
            enabled: record.job.enabled,
            created_at,
            updated_at,
        }
    }
}

/// Job form template - partial for the job creation form.
#[derive(Template, WebTemplate)]
#[template(path = "partials/job_form.html")]
pub struct JobFormTemplate {
    pub form: JobFormDataView,
    pub available_datasources: Vec<String>,
    pub methods_json: String,
    pub error: Option<String>,
}

impl Default for JobFormTemplate {
    fn default() -> Self {
        Self {
            form: JobFormDataView::default(),
            available_datasources: vec![
                "alternativeme".to_string(),
                "coingecko".to_string(),
                "coinmarketcap".to_string(),
                "polymarket".to_string(),
            ],
            methods_json: default_methods_json(),
            error: None,
        }
    }
}

/// Returns the default JSON string mapping datasource to available methods.
fn default_methods_json() -> String {
    // Fallback hardcoded version when no client is available
    r#"{
        "alternativeme": {
            "get_fear_and_greed": {"params": []},
            "get_global": {"params": []},
            "get_ticker": {"params": [{"name": "target", "description": "Cryptocurrency ID or slug (e.g., 'bitcoin')", "required": true}]}
        },
        "coingecko": {
            "get_simple_price": {"params": []},
            "get_coins_markets": {"params": []},
            "get_trending": {"params": []},
            "get_global": {"params": []}
        },
        "coinmarketcap": {
            "get_listings_latest": {"params": []},
            "get_global_metrics_quotes_latest": {"params": []},
            "get_fear_and_greed_latest": {"params": []}
        },
        "polymarket": {
            "get_markets": {"params": []},
            "get_events": {"params": []},
            "get_series": {"params": []}
        }
    }"#
    .to_string()
}

/// Generate methods JSON from a DataSourceClient, including parameter metadata.
pub fn generate_methods_json(client: &dyn crate::client::DataSourceClient) -> String {
    use std::collections::HashMap;

    let methods = client.supported_methods();
    let mut datasource_methods: HashMap<String, serde_json::Value> = HashMap::new();

    // Group methods by datasource (for now we only have one client, so we use its name)
    // In the future, we might need to aggregate from multiple clients
    let mut method_map = serde_json::Map::new();
    for method in methods {
        let params: Vec<serde_json::Value> = method
            .params
            .iter()
            .map(|p| {
                serde_json::json!({
                    "name": p.name,
                    "description": p.description,
                    "required": p.required
                })
            })
            .collect();

        method_map.insert(
            method.method.to_string(),
            serde_json::json!({
                "params": params,
                "description": method.description
            }),
        );
    }

    // For now, detect datasource from client type name - this is a workaround
    // In production, we might want a better way to identify the datasource
    datasource_methods.insert(
        "alternativeme".to_string(),
        serde_json::Value::Object(method_map),
    );

    // Add placeholder entries for other datasources (until we implement their clients)
    datasource_methods.insert(
        "coingecko".to_string(),
        serde_json::json!({
            "get_simple_price": {"params": [], "description": "Get simple price"},
            "get_coins_markets": {"params": [], "description": "Get coins markets"},
            "get_trending": {"params": [], "description": "Get trending"},
            "get_global": {"params": [], "description": "Get global metrics"}
        }),
    );
    datasource_methods.insert(
        "coinmarketcap".to_string(),
        serde_json::json!({
            "get_listings_latest": {"params": [], "description": "Get latest listings"},
            "get_global_metrics_quotes_latest": {"params": [], "description": "Get global metrics"},
            "get_fear_and_greed_latest": {"params": [], "description": "Get fear and greed"}
        }),
    );
    datasource_methods.insert(
        "polymarket".to_string(),
        serde_json::json!({
            "get_markets": {"params": [], "description": "Get markets"},
            "get_events": {"params": [], "description": "Get events"},
            "get_series": {"params": [], "description": "Get series"}
        }),
    );

    serde_json::to_string(&datasource_methods).unwrap_or_else(|_| default_methods_json())
}

/// View model for job form data
#[derive(Debug, Default, Clone)]
pub struct JobFormDataView {
    pub name: String,
    pub datasource: String,
    pub method: String,
    pub schedule_type: String,
    pub schedule_value: String,
    pub params: Option<String>,
    pub retention_days: u32,
    pub enabled: Option<String>, // "true" or None (checkbox)
}
