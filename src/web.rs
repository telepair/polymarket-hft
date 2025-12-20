//! Web UI for metrics visualization and configuration.

pub mod handlers;
pub mod templates;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::storage::{redis::RedisClient, timescale::TimescaleClient};

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub timescale: Option<Arc<TimescaleClient>>,
    pub redis: Option<Arc<RedisClient>>,
}

impl AppState {
    /// Creates a new application state.
    pub fn new() -> Self {
        Self {
            timescale: None,
            redis: None,
        }
    }

    /// Sets the TimescaleDB client.
    pub fn with_timescale(mut self, client: Arc<TimescaleClient>) -> Self {
        self.timescale = Some(client);
        self
    }

    /// Sets the Redis client.
    pub fn with_redis(mut self, client: Arc<RedisClient>) -> Self {
        self.redis = Some(client);
        self
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates the web application router.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Pages
        .route("/", get(handlers::index))
        .route("/metrics", get(handlers::metrics_page))
        .route("/state", get(handlers::state_page))
        .route("/config", get(handlers::config_page))
        // API endpoints for metrics and state
        .route("/api/metrics", get(handlers::api_metrics))
        .route("/api/state", get(handlers::api_state))
        // API endpoints for config CRUD
        .route("/api/config", get(handlers::api_config_list))
        .route("/api/config", post(handlers::api_config_create))
        .route("/api/config/{id}", get(handlers::api_config_get))
        .route("/api/config/{id}", put(handlers::api_config_update))
        .route("/api/config/{id}", delete(handlers::api_config_delete))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
