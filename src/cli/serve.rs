//! Serve command handler.
//!
//! Starts the data ingestion server with configured data sources and jobs.

use std::path::PathBuf;
use std::sync::Arc;

use clap::Args;

use polymarket_hft::client::alternativeme::Client as AlternativeMeClient;
use polymarket_hft::client::http::HttpClientConfig;
use polymarket_hft::config::{AppConfig, IngestionJob};
use polymarket_hft::ingestor::IngestorManager;
use polymarket_hft::storage::archiver::LogArchiver;

/// Arguments for the serve command.
#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Path to the configuration file (YAML).
    #[arg(short, long)]
    pub config: PathBuf,
}

/// Handle the serve command.
pub async fn handle(args: &ServeArgs) -> anyhow::Result<()> {
    tracing::info!(config = %args.config.display(), "Loading configuration");

    // Load configuration
    let config = AppConfig::from_file(&args.config)?;
    tracing::info!(
        host = %config.server.host,
        port = config.server.port,
        "Server configuration loaded"
    );

    // Build data source client (only alternativeme for now)
    // Use datasource-specific config with fallback to common config
    let http_config = config
        .datasources
        .alternativeme
        .map(HttpClientConfig::from)
        .or_else(|| config.datasources.common.map(HttpClientConfig::from))
        .unwrap_or_default();
    let client = Arc::new(AlternativeMeClient::with_config(http_config));
    tracing::info!("Alternative.me client initialized");

    // Load ingestion jobs
    let jobs = if let Some(ingestion) = &config.ingestion {
        let jobs = IngestionJob::load_from_dir(&ingestion.jobs_dir)?;
        tracing::info!(
            count = jobs.len(),
            dir = %ingestion.jobs_dir.display(),
            "Loaded ingestion jobs"
        );
        for job in &jobs {
            tracing::info!(
                name = %job.name,
                method = %job.method,
                "  - Job registered"
            );
        }
        jobs
    } else {
        tracing::info!("No ingestion configuration, running without jobs");
        Vec::new()
    };

    // Create archiver
    let archiver = Arc::new(LogArchiver);

    // Create ingestor manager
    let manager = IngestorManager::new(jobs, client, archiver);

    tracing::info!(
        host = %config.server.host,
        port = config.server.port,
        "Server starting (press Ctrl+C to stop)"
    );

    // Run the ingestor manager (handles shutdown internally)
    manager.run().await?;

    tracing::info!("Server stopped");
    Ok(())
}
