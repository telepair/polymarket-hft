//! Alternative.me CLI module.
//!
//! This module provides CLI commands for interacting with the Alternative.me free Crypto API.
//!
//! **Note**: No API key required.

use crate::cli::common::write_json_output;

use clap::Subcommand;
use polymarket_hft::client::alternativeme::Client;

/// Alternative.me API commands (free, no API key required)
#[allow(clippy::enum_variant_names)] // All variants are API commands with 'Get' prefix
#[derive(Subcommand)]
pub enum AlternativeMeCommands {
    /// List cryptocurrency ticker data
    ListTicker {
        /// Number of results to return (default: 100)
        #[arg(long)]
        limit: Option<i32>,
        /// Starting position for pagination
        #[arg(long)]
        start: Option<i32>,
        #[arg(long)]
        sort: Option<String>,
    },
    /// Get ticker data for a specific cryptocurrency
    GetTicker {
        /// Cryptocurrency ID or slug (e.g., "1" or "bitcoin")
        #[arg(long)]
        target: String,
    },
    /// Get global market metrics
    GetGlobal,
    /// Get Fear and Greed Index
    GetFearAndGreed {
        /// Number of results to return (0 for all, default: 1)
        #[arg(short, long)]
        limit: Option<i32>,
    },
}

pub async fn handle(command: &AlternativeMeCommands) -> anyhow::Result<()> {
    let client = Client::new();

    match command {
        AlternativeMeCommands::ListTicker { limit, start, sort } => {
            let response = client.list_ticker(*limit, *start, sort.clone()).await?;
            write_json_output(&response)?;
        }
        AlternativeMeCommands::GetTicker { target } => {
            let response = client.get_ticker(target.clone()).await?;
            write_json_output(&response)?;
        }
        AlternativeMeCommands::GetGlobal => {
            let response = client.get_global().await?;
            write_json_output(&response)?;
        }
        AlternativeMeCommands::GetFearAndGreed { limit } => {
            let response = client.get_fear_and_greed(*limit).await?;
            write_json_output(&response)?;
        }
    }

    Ok(())
}
