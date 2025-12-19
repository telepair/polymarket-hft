//! CoinMarketCap CLI module.
//!
//! This module provides CLI commands for interacting with the CoinMarketCap API.
//!
//! **Note**: Requires `CMC_API_KEY` environment variable.

use crate::cli::common::write_json_output;

use clap::{Args, Subcommand};
use polymarket_hft::client::coinmarketcap::{
    Client, GetFearAndGreedLatestRequest, GetGlobalMetricsQuotesLatestRequest,
    GetListingsLatestRequest,
};

/// CoinMarketCap API commands (requires CMC_API_KEY env var)
#[allow(clippy::enum_variant_names)] // All variants are API commands with 'Get' prefix
#[derive(Subcommand)]
pub enum CmcCommands {
    /// Get latest cryptocurrency listings
    GetListings {
        #[command(flatten)]
        params: GetListingsArgs,
    },
    /// Get global market metrics (total market cap, BTC dominance, etc.)
    GetGlobalMetrics {
        /// Currency for quotes (e.g., USD, EUR)
        #[arg(short, long)]
        convert: Option<String>,
    },
    /// Get Fear and Greed Index
    GetFearAndGreed,
    /// Get API key usage information
    GetKeyInfo,
}

#[derive(Args, Debug, Clone)]
pub struct GetListingsArgs {
    /// Number of results to return (max: 5000)
    #[arg(short, long, default_value_t = 10)]
    pub limit: i32,
    /// Starting position for pagination (1-based)
    #[arg(long)]
    pub start: Option<i32>,
    /// Minimum price filter
    #[arg(long)]
    pub price_min: Option<f64>,
    /// Maximum price filter
    #[arg(long)]
    pub price_max: Option<f64>,
    /// Minimum market cap filter
    #[arg(long)]
    pub market_cap_min: Option<f64>,
    /// Maximum market cap filter
    #[arg(long)]
    pub market_cap_max: Option<f64>,
    /// Currency for quotes (e.g., USD, EUR)
    #[arg(short, long)]
    pub convert: Option<String>,
    /// Sort field (market_cap, name, price, volume_24h)
    #[arg(long)]
    pub sort: Option<String>,
    /// Sort direction (asc, desc)
    #[arg(long)]
    pub sort_dir: Option<String>,
    /// Cryptocurrency type filter (all, coins, tokens)
    #[arg(long)]
    pub cryptocurrency_type: Option<String>,
    /// Tag filter (defi, filesharing, etc.)
    #[arg(long)]
    pub tag: Option<String>,
}

fn get_api_key() -> anyhow::Result<String> {
    std::env::var("CMC_API_KEY").map_err(|_| {
        anyhow::anyhow!(
            "CMC_API_KEY environment variable not set.\n\
             Get a free API key at: https://coinmarketcap.com/api/"
        )
    })
}

pub async fn handle(command: &CmcCommands) -> anyhow::Result<()> {
    let api_key = get_api_key()?;
    let client = Client::new(api_key);

    match command {
        CmcCommands::GetListings { params } => {
            let request = GetListingsLatestRequest {
                start: params.start,
                limit: Some(params.limit),
                price_min: params.price_min,
                price_max: params.price_max,
                market_cap_min: params.market_cap_min,
                market_cap_max: params.market_cap_max,
                convert: params.convert.clone(),
                sort: params.sort.clone(),
                sort_dir: params.sort_dir.clone(),
                cryptocurrency_type: params.cryptocurrency_type.clone(),
                tag: params.tag.clone(),
                ..Default::default()
            };
            let response = client.get_listings_latest(request).await?;
            write_json_output(&response)?;
        }
        CmcCommands::GetGlobalMetrics { convert } => {
            let request = GetGlobalMetricsQuotesLatestRequest {
                convert: convert.clone(),
                ..Default::default()
            };
            let response = client.get_global_metrics_quotes_latest(request).await?;
            write_json_output(&response)?;
        }
        CmcCommands::GetFearAndGreed => {
            let response = client
                .get_fear_and_greed_latest(GetFearAndGreedLatestRequest::default())
                .await?;
            write_json_output(&response)?;
        }
        CmcCommands::GetKeyInfo => {
            let response = client.get_key_info().await?;
            write_json_output(&response)?;
        }
    }

    Ok(())
}
