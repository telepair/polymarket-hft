//! Serve command handler logic.
//!
//! Starts the data ingestion server with configured data sources and jobs.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::client::alternativeme::Client as AlternativeMeClient;
use crate::client::http::HttpClientConfig;
use crate::config::{AppConfig, StorageBackendType, StorageConfig};
use crate::scheduler::SchedulerHandle;
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

    // Jobs are now managed entirely via web UI and stored in database
    tracing::info!("Jobs will be loaded from database (manage via /jobs page)");

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

    // Create cancellation token for coordinated shutdown
    let shutdown_token = CancellationToken::new();

    // Create shared scheduler handle (used by both TaskManager and web handlers)
    let scheduler =
        SchedulerHandle::new(client.clone(), storage.clone(), instance_id.clone()).await?;
    tracing::info!("Scheduler handle created");

    // Create task manager (handles ingestion, cleanup, and metadata refresh)
    let task_manager = TaskManager::new(
        scheduler.clone(),
        storage.clone(),
        metadata_cache.clone(),
        storage_config,
        shutdown_token.clone(),
    );

    // Create web router (uses shared scheduler for dynamic job management)
    let app = crate::web::create_router(
        storage.clone(),
        metadata_cache,
        instance_id.clone(),
        client,
        scheduler,
    );

    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!(
        address = %addr,
        "Dashboard available at http://{}",
        addr
    );

    // Bind HTTP server
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Create graceful shutdown signal for web server
    let server_shutdown_token = shutdown_token.clone();
    let shutdown_signal = async move {
        server_shutdown_token.cancelled().await;
        tracing::info!("Web server received shutdown signal");
    };

    // Spawn signal handler task
    let signal_token = shutdown_token.clone();
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            tracing::info!("Shutdown signal received (Ctrl+C)");
            signal_token.cancel();
        }
    });

    // Run web server and task manager concurrently
    // Both will stop when the shutdown_token is cancelled
    let (server_result, manager_result) = tokio::join!(
        axum::serve(listener, app).with_graceful_shutdown(shutdown_signal),
        task_manager.run()
    );

    // Log any errors from either task
    if let Err(e) = server_result {
        tracing::error!(error = %e, "Web server error");
    }
    if let Err(e) = manager_result {
        tracing::error!(error = %e, "Task manager error");
    }

    // Record ServiceStop event - guaranteed to execute
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
