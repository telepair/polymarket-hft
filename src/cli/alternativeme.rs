//! Alternative.me CLI module.
//!
//! This module provides CLI commands for interacting with the Alternative.me free Crypto API.
//!
//! **Note**: No API key required.

use crate::cli::common::write_json_output;

use clap::{Args, Subcommand};
use polymarket_hft::client::alternativeme::{
    Client, GetFearAndGreedRequest, GetGlobalRequest, GetTickerByIdRequest, GetTickerRequest,
};

/// Alternative.me API commands (free, no API key required)
#[allow(clippy::enum_variant_names)] // All variants are API commands with 'Get' prefix
#[derive(Subcommand)]
pub enum AlternativeMeCommands {
    /// Get cryptocurrency ticker data
    GetTicker {
        #[command(flatten)]
        params: GetTickerArgs,
    },
    /// Get ticker data for a specific cryptocurrency
    GetTickerById {
        /// Cryptocurrency ID or slug (e.g., "1" or "bitcoin")
        id: String,
        /// Currency for quotes (e.g., USD, EUR, BTC)
        #[arg(short, long)]
        convert: Option<String>,
    },
    /// Get global market metrics
    GetGlobal {
        /// Currency for quotes (e.g., USD, EUR, BTC)
        #[arg(short, long)]
        convert: Option<String>,
    },
    /// Get Fear and Greed Index
    GetFearAndGreed {
        #[command(flatten)]
        params: GetFearAndGreedArgs,
    },
}

#[derive(Args, Debug, Clone)]
pub struct GetTickerArgs {
    /// Number of results to return (0 for all, default: 100)
    #[arg(short, long)]
    pub limit: Option<i32>,
    /// Starting position for pagination
    #[arg(long)]
    pub start: Option<i32>,
    /// Currency for quotes (e.g., USD, EUR, BTC)
    #[arg(short, long)]
    pub convert: Option<String>,
    /// Sort field: id, rank, volume_24h, percent_change_24h, price, etc.
    #[arg(long)]
    pub sort: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct GetFearAndGreedArgs {
    /// Number of results to return (0 for all historical data, default: 1)
    #[arg(short, long)]
    pub limit: Option<i32>,
    /// Date format: us, cn, kr, world
    #[arg(long)]
    pub date_format: Option<String>,
}

pub async fn handle(command: &AlternativeMeCommands) -> anyhow::Result<()> {
    let client = Client::new();

    match command {
        AlternativeMeCommands::GetTicker { params } => {
            let request = GetTickerRequest {
                limit: params.limit,
                start: params.start,
                convert: params.convert.clone(),
                sort: params.sort.clone(),
                ..Default::default()
            };
            let response = client.get_ticker(request).await?;
            write_json_output(&response)?;
        }
        AlternativeMeCommands::GetTickerById { id, convert } => {
            let request = GetTickerByIdRequest {
                convert: convert.clone(),
                ..Default::default()
            };
            let response = client.get_ticker_by_id(id, request).await?;
            write_json_output(&response)?;
        }
        AlternativeMeCommands::GetGlobal { convert } => {
            let request = GetGlobalRequest {
                convert: convert.clone(),
            };
            let response = client.get_global(request).await?;
            write_json_output(&response)?;
        }
        AlternativeMeCommands::GetFearAndGreed { params } => {
            let request = GetFearAndGreedRequest {
                limit: params.limit,
                date_format: params.date_format.clone(),
                ..Default::default()
            };
            let response = client.get_fear_and_greed(request).await?;
            write_json_output(&response)?;
        }
    }

    Ok(())
}
