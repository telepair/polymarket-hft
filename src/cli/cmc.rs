//! CoinMarketCap CLI module.
//!
//! This module provides CLI commands for interacting with the CoinMarketCap API.
//!
//! **Note**: Requires `CMC_API_KEY` environment variable.

use crate::cli::common::write_json_output;

use clap::{Args, Subcommand};
use polymarket_hft::client::coinmarketcap::{
    Client, GetCryptocurrencyInfoRequest, GetCryptocurrencyMapRequest,
    GetFearAndGreedLatestRequest, GetFiatMapRequest, GetGlobalMetricsQuotesLatestRequest,
    GetListingsLatestRequest, GetQuotesLatestRequest, PriceConversionRequest,
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
    /// Get cryptocurrency ID map
    GetMap {
        #[command(flatten)]
        params: GetMapArgs,
    },
    /// Get cryptocurrency metadata (logo, description, URLs)
    GetInfo {
        #[command(flatten)]
        params: GetInfoArgs,
    },
    /// Get latest quotes for specific cryptocurrencies
    GetQuotes {
        #[command(flatten)]
        params: GetQuotesArgs,
    },
    /// Get fiat currency ID map
    GetFiatMap {
        #[command(flatten)]
        params: GetFiatMapArgs,
    },
    /// Convert amount between currencies
    PriceConvert {
        #[command(flatten)]
        params: PriceConvertArgs,
    },
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

#[derive(Args, Debug, Clone)]
pub struct GetMapArgs {
    /// Number of results to return
    #[arg(short, long, default_value_t = 100)]
    pub limit: i32,
    /// Starting position for pagination (1-based)
    #[arg(long)]
    pub start: Option<i32>,
    /// Listing status: active, inactive, untracked
    #[arg(long)]
    pub listing_status: Option<String>,
    /// Filter by symbol (comma-separated, e.g., BTC,ETH)
    #[arg(short, long)]
    pub symbol: Option<String>,
    /// Sort field: id, cmc_rank
    #[arg(long)]
    pub sort: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct GetInfoArgs {
    /// CoinMarketCap ID (comma-separated for multiple)
    #[arg(long)]
    pub id: Option<String>,
    /// Cryptocurrency slug (comma-separated for multiple)
    #[arg(long)]
    pub slug: Option<String>,
    /// Cryptocurrency symbol (comma-separated for multiple, e.g., BTC,ETH)
    #[arg(short, long)]
    pub symbol: Option<String>,
    /// Skip invalid lookups
    #[arg(long)]
    pub skip_invalid: bool,
}

#[derive(Args, Debug, Clone)]
pub struct GetQuotesArgs {
    /// CoinMarketCap ID (comma-separated for multiple)
    #[arg(long)]
    pub id: Option<String>,
    /// Cryptocurrency slug (comma-separated for multiple)
    #[arg(long)]
    pub slug: Option<String>,
    /// Cryptocurrency symbol (comma-separated for multiple, e.g., BTC,ETH)
    #[arg(short, long)]
    pub symbol: Option<String>,
    /// Currency for quotes (e.g., USD, EUR)
    #[arg(short, long)]
    pub convert: Option<String>,
    /// Skip invalid lookups
    #[arg(long)]
    pub skip_invalid: bool,
}

#[derive(Args, Debug, Clone)]
pub struct GetFiatMapArgs {
    /// Number of results to return
    #[arg(short, long, default_value_t = 100)]
    pub limit: i32,
    /// Starting position for pagination (1-based)
    #[arg(long)]
    pub start: Option<i32>,
    /// Sort field: id, name
    #[arg(long)]
    pub sort: Option<String>,
    /// Include precious metals (gold, silver, etc.)
    #[arg(long)]
    pub include_metals: bool,
}

#[derive(Args, Debug, Clone)]
pub struct PriceConvertArgs {
    /// Amount to convert
    #[arg(short, long)]
    pub amount: f64,
    /// Source currency CoinMarketCap ID
    #[arg(long)]
    pub id: Option<i32>,
    /// Source currency symbol (e.g., BTC)
    #[arg(short, long)]
    pub symbol: Option<String>,
    /// Target currency symbol(s) for conversion (comma-separated, e.g., USD,EUR)
    #[arg(short, long, default_value = "USD")]
    pub convert: String,
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
        CmcCommands::GetMap { params } => {
            let request = GetCryptocurrencyMapRequest {
                start: params.start,
                limit: Some(params.limit),
                listing_status: params.listing_status.clone(),
                symbol: params.symbol.clone(),
                sort: params.sort.clone(),
                ..Default::default()
            };
            let response = client.get_cryptocurrency_map(request).await?;
            write_json_output(&response)?;
        }
        CmcCommands::GetInfo { params } => {
            let request = GetCryptocurrencyInfoRequest {
                id: params.id.clone(),
                slug: params.slug.clone(),
                symbol: params.symbol.clone(),
                skip_invalid: if params.skip_invalid {
                    Some(true)
                } else {
                    None
                },
                ..Default::default()
            };
            let response = client.get_cryptocurrency_info(request).await?;
            write_json_output(&response)?;
        }
        CmcCommands::GetQuotes { params } => {
            let request = GetQuotesLatestRequest {
                id: params.id.clone(),
                slug: params.slug.clone(),
                symbol: params.symbol.clone(),
                convert: params.convert.clone(),
                skip_invalid: if params.skip_invalid {
                    Some(true)
                } else {
                    None
                },
                ..Default::default()
            };
            let response = client.get_quotes_latest(request).await?;
            write_json_output(&response)?;
        }
        CmcCommands::GetFiatMap { params } => {
            let request = GetFiatMapRequest {
                start: params.start,
                limit: Some(params.limit),
                sort: params.sort.clone(),
                include_metals: if params.include_metals {
                    Some(true)
                } else {
                    None
                },
            };
            let response = client.get_fiat_map(request).await?;
            write_json_output(&response)?;
        }
        CmcCommands::PriceConvert { params } => {
            let request = PriceConversionRequest {
                amount: params.amount,
                id: params.id,
                symbol: params.symbol.clone(),
                convert: Some(params.convert.clone()),
                ..Default::default()
            };
            let response = client.get_price_conversion(request).await?;
            write_json_output(&response)?;
        }
    }

    Ok(())
}
