//! Scheduler for executing scrape jobs.
//!
//! The scheduler runs configured scrape jobs according to their schedule
//! (interval or cron) and dispatches results to the appropriate handlers.

use crate::engine::{Schedule, ScrapeConfig, ScrapeJob, Target};
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};
use tracing::{debug, error, info, warn};

/// Message sent when a scrape job produces results.
#[derive(Debug)]
pub enum ScrapeResult {
    /// Metrics to be archived in TimescaleDB.
    Metrics(Vec<crate::engine::Metric>),
    /// State entries to be stored in Redis.
    State(Vec<crate::engine::StateEntry>),
}

/// Scheduler that executes scrape jobs according to their configuration.
pub struct Scheduler {
    config: ScrapeConfig,
    result_tx: mpsc::Sender<ScrapeResult>,
}

impl Scheduler {
    /// Creates a new scheduler with the given configuration.
    pub fn new(config: ScrapeConfig, result_tx: mpsc::Sender<ScrapeResult>) -> Self {
        Self { config, result_tx }
    }

    /// Starts the scheduler, spawning a task for each enabled job.
    pub async fn run(self) {
        info!(
            "Starting scheduler with {} jobs",
            self.config.scrape_jobs.len()
        );

        let mut handles = Vec::new();

        for job in self.config.scrape_jobs.into_iter() {
            if !job.enabled {
                debug!("Skipping disabled job: {}", job.id);
                continue;
            }

            let tx = self.result_tx.clone();
            let handle = tokio::spawn(async move {
                run_job(job, tx).await;
            });
            handles.push(handle);
        }

        // Wait for all jobs (they run indefinitely)
        for handle in handles {
            let _ = handle.await;
        }
    }
}

/// Runs a single scrape job according to its schedule.
async fn run_job(job: ScrapeJob, tx: mpsc::Sender<ScrapeResult>) {
    info!(
        "Starting job: {} (source: {}, endpoint: {})",
        job.id, job.source, job.endpoint
    );

    let schedule = job.schedule.clone();
    match schedule {
        Schedule::Interval { interval: dur } => {
            run_interval_job(job, dur, tx).await;
        }
        Schedule::Cron { expression } => {
            // For cron scheduling, we'd use tokio-cron-scheduler
            // For now, fall back to a default interval
            warn!(
                "Cron scheduling not yet implemented for job {}, using 1h interval. Expression: {}",
                job.id, expression
            );
            run_interval_job(job, Duration::from_secs(3600), tx).await;
        }
    }
}

/// Runs a job at fixed intervals.
async fn run_interval_job(job: ScrapeJob, dur: Duration, tx: mpsc::Sender<ScrapeResult>) {
    let mut ticker = interval(dur);

    loop {
        ticker.tick().await;
        debug!("Executing job: {}", job.id);

        match execute_scrape(&job).await {
            Ok((metrics, states)) => {
                // Send metrics if targeting TimescaleDB
                if job.targets.contains(&Target::Metrics)
                    && !metrics.is_empty()
                    && let Err(e) = tx.send(ScrapeResult::Metrics(metrics)).await
                {
                    error!("Failed to send metrics for job {}: {}", job.id, e);
                }

                // Send state if targeting Redis
                if job.targets.contains(&Target::State) && !states.is_empty() {
                    // Apply job-specific TTL if configured
                    let states = if let Some(ttl) = job.state_ttl {
                        states
                            .into_iter()
                            .map(|mut s| {
                                if s.ttl.is_none() {
                                    s.ttl = Some(ttl);
                                }
                                s
                            })
                            .collect()
                    } else {
                        states
                    };

                    if let Err(e) = tx.send(ScrapeResult::State(states)).await {
                        error!("Failed to send state for job {}: {}", job.id, e);
                    }
                }
            }
            Err(e) => {
                error!("Job {} failed: {}", job.id, e);
            }
        }
    }
}

/// Executes a scrape and returns metrics and state entries.
///
/// This is a placeholder that should be extended to dynamically dispatch
/// to the appropriate client based on source and endpoint.
async fn execute_scrape(
    job: &ScrapeJob,
) -> Result<
    (Vec<crate::engine::Metric>, Vec<crate::engine::StateEntry>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    // TODO: Implement dynamic dispatch based on job.source and job.endpoint
    // For now, return empty results as a placeholder
    debug!(
        "Scrape job {} would call {}.{} with params: {}",
        job.id, job.source, job.endpoint, job.params
    );

    Ok((Vec::new(), Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = ScrapeConfig {
            scrape_jobs: vec![],
        };
        let (tx, _rx) = mpsc::channel(10);
        let scheduler = Scheduler::new(config, tx);
        assert!(scheduler.config.scrape_jobs.is_empty());
    }
}
