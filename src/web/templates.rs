//! HTML templates for the web dashboard.

use askama::Template;
use askama_web::WebTemplate;

use crate::storage::Metric;

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

/// Dashboard page template.
#[derive(Template, WebTemplate)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub title: String,
    pub filter_params: FilterParams,
    pub available_sources: Vec<String>,
    pub available_names: Vec<String>,
}

impl Default for DashboardTemplate {
    fn default() -> Self {
        Self {
            title: "Metrics Dashboard".to_string(),
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
        let timestamp = chrono::DateTime::from_timestamp(m.timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| m.timestamp.to_string());

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
