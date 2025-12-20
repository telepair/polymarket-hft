//! Serve command for starting the web server.

use clap::Args;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

use polymarket_hft::engine::ScrapeConfig;
use polymarket_hft::storage::{redis::RedisClient, timescale::TimescaleClient};
use polymarket_hft::web::{AppState, create_router};

/// Arguments for the serve command.
#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Port to listen on
    #[arg(short, long, default_value = "3000", env = "PORT")]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0", env = "HOST")]
    pub host: String,

    /// TimescaleDB connection URL
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: Option<String>,

    /// Redis connection URL
    #[arg(long, env = "REDIS_URL")]
    pub redis_url: Option<String>,

    /// Path to scrape config file (YAML/JSON)
    #[arg(long, env = "SCRAPE_CONFIG_FILE")]
    pub config_file: Option<String>,

    /// Directory containing scrape config files
    #[arg(long, env = "SCRAPE_CONFIG_DIR")]
    pub config_dir: Option<String>,
}

/// Handle the serve command.
pub async fn handle(args: &ServeArgs) -> anyhow::Result<()> {
    let mut state = AppState::new();
    let mut timescale_client: Option<Arc<TimescaleClient>> = None;

    // Connect to TimescaleDB if URL provided
    if let Some(url) = &args.database_url {
        info!("Connecting to TimescaleDB...");
        match TimescaleClient::new(url).await {
            Ok(client) => {
                info!("TimescaleDB connected");
                // Initialize metrics schema
                if let Err(e) = client.init_schema().await {
                    tracing::warn!("Failed to init metrics schema (may already exist): {}", e);
                }
                // Initialize scrape_jobs schema
                if let Err(e) = client.init_scrape_jobs_schema().await {
                    tracing::warn!(
                        "Failed to init scrape_jobs schema (may already exist): {}",
                        e
                    );
                }
                let client = Arc::new(client);
                timescale_client = Some(client.clone());
                state = state.with_timescale(client);
            }
            Err(e) => {
                tracing::warn!("Failed to connect to TimescaleDB: {}", e);
            }
        }
    } else {
        info!("No DATABASE_URL set, TimescaleDB disabled");
    }

    // Connect to Redis if URL provided
    if let Some(url) = &args.redis_url {
        info!("Connecting to Redis...");
        match RedisClient::new(url).await {
            Ok(client) => {
                info!("Redis connected");
                state = state.with_redis(Arc::new(client));
            }
            Err(e) => {
                tracing::warn!("Failed to connect to Redis: {}", e);
            }
        }
    } else {
        info!("No REDIS_URL set, Redis disabled");
    }

    // Load scrape jobs from config file or directory
    if let Some(ref db) = timescale_client {
        load_scrape_configs(args, db.as_ref()).await?;
    }

    let app = create_router(state);
    let addr = format!("{}:{}", args.host, args.port);

    info!("Starting web server on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Load scrape configs from file or directory and insert into DB if not exists.
async fn load_scrape_configs(args: &ServeArgs, db: &TimescaleClient) -> anyhow::Result<()> {
    let mut configs = Vec::new();

    // Load from single config file
    if let Some(ref file_path) = args.config_file {
        info!("Loading scrape config from file: {}", file_path);
        match ScrapeConfig::from_file(file_path) {
            Ok(config) => configs.push(config),
            Err(e) => {
                tracing::warn!("Failed to load config from {}: {}", file_path, e);
            }
        }
    }

    // Load from config directory
    if let Some(ref dir_path) = args.config_dir {
        info!("Loading scrape configs from directory: {}", dir_path);
        let path = Path::new(dir_path);
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if (ext == "yaml" || ext == "yml" || ext == "json")
                        && let Some(path_str) = file_path.to_str()
                    {
                        match ScrapeConfig::from_file(path_str) {
                            Ok(config) => {
                                info!("Loaded config from: {}", path_str);
                                configs.push(config);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load config from {}: {}", path_str, e);
                            }
                        }
                    }
                }
            }
        } else {
            tracing::warn!("Config directory does not exist: {}", dir_path);
        }
    }

    // Insert jobs into DB if not already present
    let mut inserted = 0;
    let mut skipped = 0;
    for config in configs {
        for job in config.scrape_jobs {
            match db.insert_scrape_job_if_not_exists(&job).await {
                Ok(true) => {
                    info!("Inserted scrape job: {}", job.id);
                    inserted += 1;
                }
                Ok(false) => {
                    tracing::debug!("Job already exists, skipping: {}", job.id);
                    skipped += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to insert job {}: {}", job.id, e);
                }
            }
        }
    }

    if inserted > 0 || skipped > 0 {
        info!(
            "Scrape jobs loaded: {} inserted, {} already existed",
            inserted, skipped
        );
    }

    Ok(())
}
