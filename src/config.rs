//! Application configuration.
//!
//! This module provides configuration structures for the serve command,
//! including server settings, data source configurations, and ingestion jobs.

mod job;
mod settings;

pub use job::{IngestionJob, Schedule};
pub use settings::{
    AppConfig, DataSourcesConfig, HttpClientConfigSerde, IngestionConfig, ServerConfig,
};
