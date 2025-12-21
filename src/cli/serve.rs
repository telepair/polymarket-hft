//! Serve command handler.
//!
//! Starts the data ingestion server with configured data sources and jobs.

use std::path::PathBuf;
use std::sync::Arc;

use clap::Args;

use polymarket_hft::client::alternativeme::Client as AlternativeMeClient;
use polymarket_hft::client::http::HttpClientConfig;
use polymarket_hft::config::{AppConfig, IngestionJob, StorageBackendType, StorageConfig};
use polymarket_hft::ingestor::IngestorManager;
use polymarket_hft::storage::local::LocalStorage;
use polymarket_hft::{LocalStorageConfig, StorageBackend};

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
                retention_days = job.retention_days,
                "  - Job registered"
            );
        }
        jobs
    } else {
        tracing::info!("No ingestion configuration, running without jobs");
        Vec::new()
    };

    // Create storage backend based on configuration
    let storage_config = config.storage.unwrap_or_default();
    let storage: Arc<dyn StorageBackend> = create_storage(&storage_config).await?;

    // Start cleanup task with system-level configuration
    start_cleanup_task(
        storage.clone(),
        storage_config.cleanup_interval_secs,
        storage_config.retention_days,
    );

    // Create ingestor manager
    let manager = IngestorManager::new(jobs, client, storage);

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

/// Create storage backend based on configuration.
async fn create_storage(config: &StorageConfig) -> anyhow::Result<Arc<dyn StorageBackend>> {
    match config.backend {
        StorageBackendType::Local => {
            let local_config: LocalStorageConfig = config.local.clone().unwrap_or_default().into();
            tracing::info!(
                db_path = %local_config.db_path.display(),
                cache_ttl_secs = local_config.cache_ttl.as_secs(),
                "Using local storage backend (SQLite + memory cache)"
            );
            let storage = LocalStorage::new(local_config).await?;
            Ok(Arc::new(storage))
        }
        StorageBackendType::External => {
            anyhow::bail!("External storage backend not yet implemented")
        }
    }
}

/// Start a background task to cleanup old metrics.
fn start_cleanup_task(storage: Arc<dyn StorageBackend>, interval_secs: u64, retention_days: u32) {
    tracing::info!(
        interval_secs = interval_secs,
        retention_days = retention_days,
        "Starting cleanup task"
    );

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;

            // Calculate cutoff timestamp
            let cutoff = chrono::Utc::now().timestamp() - (retention_days as i64 * 86400);

            match storage.cleanup_before(cutoff).await {
                Ok(deleted) => {
                    if deleted > 0 {
                        tracing::info!(
                            deleted_rows = deleted,
                            retention_days = retention_days,
                            "Cleaned up old metrics"
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to cleanup old metrics");
                }
            }
        }
    });
}
