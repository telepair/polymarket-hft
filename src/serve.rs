//! Serve command handler logic.
//!
//! Starts the data ingestion server with configured data sources and jobs.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::client::alternativeme::Client as AlternativeMeClient;
use crate::client::http::HttpClientConfig;
use crate::config::{AppConfig, IngestionJob, StorageBackendType, StorageConfig};
use crate::storage::local::LocalStorage;
use crate::storage::{Event, EventType};
use crate::task::TaskManager;
use crate::{LocalStorageConfig, StorageBackend};

/// Run the server with the given configuration file.
pub async fn run(config_path: PathBuf) -> anyhow::Result<()> {
    // Generate unique instance ID for this service run
    let instance_id = Uuid::now_v7().to_string();
    tracing::info!(instance_id = %instance_id, "Starting service instance");

    tracing::info!(config = %config_path.display(), "Loading configuration");

    // Load configuration
    let config = AppConfig::from_file(&config_path)?;
    tracing::info!(
        host = %config.server.host,
        port = config.server.port,
        "Server configuration loaded"
    );

    // Build data source client
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

    // Create storage backend
    let storage_config = config.storage.unwrap_or_default();
    let storage: Arc<dyn StorageBackend> = create_storage(&storage_config).await?;

    // Record ServiceStart event
    let start_event = Event::new(&instance_id, EventType::ServiceStart, "Service started");
    if let Err(e) = storage.store_event(&start_event).await {
        tracing::warn!(error = %e, "Failed to record service start event");
    }

    // Create shared metadata cache
    let metadata_cache = Arc::new(RwLock::new(Vec::new()));

    // Create task manager (handles ingestion, cleanup, and metadata refresh)
    let task_manager = TaskManager::new(
        jobs,
        client,
        storage.clone(),
        metadata_cache.clone(),
        storage_config,
        instance_id.clone(),
    );

    // Create web router (uses shared metadata cache and instance ID)
    let app = crate::web::create_router(storage.clone(), metadata_cache, instance_id.clone());

    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!(
        address = %addr,
        "Dashboard available at http://{}",
        addr
    );

    // Bind HTTP server
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Run web server and task manager concurrently
    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                tracing::error!(error = %e, "Web server error");
            }
        }
        result = task_manager.run() => {
            if let Err(e) = result {
                tracing::error!(error = %e, "Task manager error");
            }
        }
    }

    // Record ServiceStop event
    let stop_event = Event::new(&instance_id, EventType::ServiceStop, "Service stopped");
    if let Err(e) = storage.store_event(&stop_event).await {
        tracing::warn!(error = %e, "Failed to record service stop event");
    }

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
