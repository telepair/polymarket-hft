//! Dispatcher for routing scrape results to processors.
//!
//! The dispatcher receives scrape results from the scheduler and routes them
//! to the appropriate processors (Archiver for metrics, StateManager for state).

use crate::engine::scheduler::ScrapeResult;
use crate::engine::{Metric, StateEntry};
use crate::storage::{redis::RedisClient, timescale::TimescaleClient};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Dispatcher that routes scrape results to storage backends.
pub struct Dispatcher {
    result_rx: mpsc::Receiver<ScrapeResult>,
    timescale: Option<Arc<TimescaleClient>>,
    redis: Option<Arc<RedisClient>>,
}

impl Dispatcher {
    /// Creates a new dispatcher.
    pub fn new(result_rx: mpsc::Receiver<ScrapeResult>) -> Self {
        Self {
            result_rx,
            timescale: None,
            redis: None,
        }
    }

    /// Sets the TimescaleDB client for archiving metrics.
    pub fn with_timescale(mut self, client: Arc<TimescaleClient>) -> Self {
        self.timescale = Some(client);
        self
    }

    /// Sets the Redis client for state management.
    pub fn with_redis(mut self, client: Arc<RedisClient>) -> Self {
        self.redis = Some(client);
        self
    }

    /// Runs the dispatcher, processing results as they arrive.
    pub async fn run(mut self) {
        info!("Starting dispatcher");

        while let Some(result) = self.result_rx.recv().await {
            match result {
                ScrapeResult::Metrics(metrics) => {
                    self.handle_metrics(metrics).await;
                }
                ScrapeResult::State(states) => {
                    self.handle_states(states).await;
                }
            }
        }

        info!("Dispatcher shutting down");
    }

    /// Handles metrics by archiving to TimescaleDB.
    async fn handle_metrics(&self, metrics: Vec<Metric>) {
        if metrics.is_empty() {
            return;
        }

        debug!("Archiving {} metrics", metrics.len());

        if let Some(timescale) = &self.timescale {
            if let Err(e) = timescale.insert_metrics(&metrics).await {
                error!("Failed to archive metrics: {}", e);
            }
        } else {
            debug!("No TimescaleDB client configured, skipping metrics");
        }
    }

    /// Handles state entries by storing in Redis.
    async fn handle_states(&self, states: Vec<StateEntry>) {
        if states.is_empty() {
            return;
        }

        debug!("Storing {} state entries", states.len());

        if let Some(redis) = &self.redis {
            if let Err(e) = redis.set_states(&states).await {
                error!("Failed to store state: {}", e);
            }
        } else {
            debug!("No Redis client configured, skipping state");
        }
    }
}
