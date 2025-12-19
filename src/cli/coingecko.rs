//! CoinGecko CLI module.
//!
//! This module provides CLI commands for interacting with the CoinGecko API.
//!
//! **Note**: Requires `CG_API_KEY` environment variable.

use crate::cli::common::write_json_output;

use clap::{Args, Subcommand};
use polymarket_hft::client::coingecko::{
    Client, CoinDetailRequest, CoinHistoryRequest, CoinsListRequest, CoinsMarketsRequest,
    ExchangesRequest, MarketChartRequest, OhlcRequest, SimplePriceRequest,
};

/// CoinGecko API commands (requires CG_API_KEY env var)
#[derive(Subcommand)]
pub enum CgCommands {
    /// Get simple price for coins
    SimplePrice {
        #[command(flatten)]
        params: SimplePriceArgs,
    },
    /// List supported vs currencies
    SupportedVsCurrencies,
    /// List all supported coins
    CoinsList {
        #[command(flatten)]
        params: CoinsListArgs,
    },
    /// Get market data for coins
    CoinsMarkets {
        #[command(flatten)]
        params: CoinsMarketsArgs,
    },
    /// Get trending coins, NFTs, and categories
    Trending,
    /// Get global cryptocurrency statistics
    Global,
    /// Get list of exchanges
    Exchanges {
        #[command(flatten)]
        params: ExchangesArgs,
    },
    /// Get coin detail by ID
    Coin {
        #[command(flatten)]
        params: CoinArgs,
    },
    /// Get historical market chart data
    MarketChart {
        #[command(flatten)]
        params: MarketChartArgs,
    },
    /// Get historical data at specific date
    History {
        #[command(flatten)]
        params: HistoryArgs,
    },
    /// Get OHLC candlestick data
    Ohlc {
        #[command(flatten)]
        params: OhlcArgs,
    },
}

#[derive(Args, Debug, Clone)]
pub struct SimplePriceArgs {
    /// Comma-separated coin IDs (e.g., "bitcoin,ethereum")
    #[arg(long)]
    pub ids: String,

    /// Comma-separated target currencies (e.g., "usd,eur")
    #[arg(long, default_value = "usd")]
    pub vs_currencies: String,

    /// Include market cap
    #[arg(long)]
    pub include_market_cap: bool,

    /// Include 24h volume
    #[arg(long)]
    pub include_24hr_vol: bool,

    /// Include 24h change
    #[arg(long)]
    pub include_24hr_change: bool,

    /// Include last updated timestamp
    #[arg(long)]
    pub include_last_updated_at: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct CoinsListArgs {
    /// Include platform contract addresses
    #[arg(long)]
    pub include_platform: bool,

    /// Limit output (for display only, all coins are fetched)
    #[arg(long)]
    pub limit: Option<usize>,
}

#[derive(Args, Debug, Clone)]
pub struct CoinsMarketsArgs {
    /// Target currency (e.g., "usd")
    #[arg(long, default_value = "usd")]
    pub vs_currency: String,

    /// Comma-separated coin IDs to filter
    #[arg(long)]
    pub ids: Option<String>,

    /// Filter by category
    #[arg(long)]
    pub category: Option<String>,

    /// Sort order (e.g., "market_cap_desc", "volume_desc")
    #[arg(long)]
    pub order: Option<String>,

    /// Results per page (max 250)
    #[arg(long)]
    pub per_page: Option<u32>,

    /// Page number
    #[arg(long)]
    pub page: Option<u32>,

    /// Include 7-day sparkline
    #[arg(long)]
    pub sparkline: bool,

    /// Price change % timeframes (e.g., "1h,24h,7d")
    #[arg(long)]
    pub price_change_percentage: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ExchangesArgs {
    /// Results per page (max 250)
    #[arg(long)]
    pub per_page: Option<u32>,

    /// Page number
    #[arg(long)]
    pub page: Option<u32>,
}

#[derive(Args, Debug, Clone)]
pub struct CoinArgs {
    /// Coin ID (e.g., "bitcoin")
    #[arg(long)]
    pub id: String,

    /// Skip localization data
    #[arg(long)]
    pub no_localization: bool,

    /// Skip ticker data
    #[arg(long)]
    pub no_tickers: bool,

    /// Skip market data
    #[arg(long)]
    pub no_market_data: bool,

    /// Skip community data
    #[arg(long)]
    pub no_community_data: bool,

    /// Skip developer data
    #[arg(long)]
    pub no_developer_data: bool,
}

#[derive(Args, Debug, Clone)]
pub struct MarketChartArgs {
    /// Coin ID (e.g., "bitcoin")
    #[arg(long)]
    pub id: String,

    /// Target currency (e.g., "usd")
    #[arg(long, default_value = "usd")]
    pub vs_currency: String,

    /// Number of days (1, 7, 14, 30, 90, 180, 365, max)
    #[arg(long, default_value = "7")]
    pub days: String,

    /// Data interval (optional)
    #[arg(long)]
    pub interval: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct HistoryArgs {
    /// Coin ID (e.g., "bitcoin")
    #[arg(long)]
    pub id: String,

    /// Date in dd-mm-yyyy format (e.g., "30-12-2022")
    #[arg(long)]
    pub date: String,

    /// Skip localization data
    #[arg(long)]
    pub no_localization: bool,
}

#[derive(Args, Debug, Clone)]
pub struct OhlcArgs {
    /// Coin ID (e.g., "bitcoin")
    #[arg(long)]
    pub id: String,

    /// Target currency (e.g., "usd")
    #[arg(long, default_value = "usd")]
    pub vs_currency: String,

    /// Number of days (1, 7, 14, 30, 90, 180, 365, max)
    #[arg(long, default_value = "7")]
    pub days: String,
}

fn get_api_key() -> anyhow::Result<String> {
    std::env::var("CG_API_KEY").map_err(|_| {
        anyhow::anyhow!(
            "CG_API_KEY environment variable not set. \
             Get your free API key at https://www.coingecko.com/en/api"
        )
    })
}

pub async fn handle(command: &CgCommands) -> anyhow::Result<()> {
    let api_key = get_api_key()?;
    let client = Client::new(api_key);

    match command {
        CgCommands::SimplePrice { params } => {
            let request = SimplePriceRequest {
                ids: params.ids.clone(),
                vs_currencies: params.vs_currencies.clone(),
                include_market_cap: Some(params.include_market_cap),
                include_24hr_vol: Some(params.include_24hr_vol),
                include_24hr_change: Some(params.include_24hr_change),
                include_last_updated_at: Some(params.include_last_updated_at),
            };
            let response = client.get_simple_price(request).await?;
            write_json_output(&response)?;
        }
        CgCommands::SupportedVsCurrencies => {
            let response = client.get_supported_vs_currencies().await?;
            write_json_output(&response)?;
        }
        CgCommands::CoinsList { params } => {
            let request = CoinsListRequest {
                include_platform: if params.include_platform {
                    Some(true)
                } else {
                    None
                },
            };
            let response = client.get_coins_list(request).await?;

            // Optionally limit output
            if let Some(limit) = params.limit {
                let limited: Vec<_> = response.into_iter().take(limit).collect();
                write_json_output(&limited)?;
            } else {
                write_json_output(&response)?;
            }
        }
        CgCommands::CoinsMarkets { params } => {
            let request = CoinsMarketsRequest {
                vs_currency: params.vs_currency.clone(),
                ids: params.ids.clone(),
                category: params.category.clone(),
                order: params.order.clone(),
                per_page: params.per_page,
                page: params.page,
                sparkline: if params.sparkline { Some(true) } else { None },
                price_change_percentage: params.price_change_percentage.clone(),
                ..Default::default()
            };
            let response = client.get_coins_markets(request).await?;
            write_json_output(&response)?;
        }
        CgCommands::Trending => {
            let response = client.get_trending().await?;
            write_json_output(&response)?;
        }
        CgCommands::Global => {
            let response = client.get_global().await?;
            write_json_output(&response)?;
        }
        CgCommands::Exchanges { params } => {
            let request = ExchangesRequest {
                per_page: params.per_page,
                page: params.page,
            };
            let response = client.get_exchanges(request).await?;
            write_json_output(&response)?;
        }
        CgCommands::Coin { params } => {
            let request = CoinDetailRequest {
                id: params.id.clone(),
                localization: if params.no_localization {
                    Some(false)
                } else {
                    None
                },
                tickers: if params.no_tickers { Some(false) } else { None },
                market_data: if params.no_market_data {
                    Some(false)
                } else {
                    None
                },
                community_data: if params.no_community_data {
                    Some(false)
                } else {
                    None
                },
                developer_data: if params.no_developer_data {
                    Some(false)
                } else {
                    None
                },
                sparkline: None,
            };
            let response = client.get_coin(request).await?;
            write_json_output(&response)?;
        }
        CgCommands::MarketChart { params } => {
            let request = MarketChartRequest {
                id: params.id.clone(),
                vs_currency: params.vs_currency.clone(),
                days: params.days.clone(),
                interval: params.interval.clone(),
            };
            let response = client.get_coin_market_chart(request).await?;
            write_json_output(&response)?;
        }
        CgCommands::History { params } => {
            let request = CoinHistoryRequest {
                id: params.id.clone(),
                date: params.date.clone(),
                localization: if params.no_localization {
                    Some(false)
                } else {
                    None
                },
            };
            let response = client.get_coin_history(request).await?;
            write_json_output(&response)?;
        }
        CgCommands::Ohlc { params } => {
            let request = OhlcRequest {
                id: params.id.clone(),
                vs_currency: params.vs_currency.clone(),
                days: params.days.clone(),
            };
            let response = client.get_coin_ohlc(request).await?;
            write_json_output(&response)?;
        }
    }

    Ok(())
}
