//! Data Source CLI module.
//!
//! This module aggregates all data source commands under a single `ds` subcommand.
//!
//! ## Polymarket APIs
//! - `clob` - CLOB API (order book, pricing)
//! - `clob-ws` - CLOB WebSocket (real-time updates)
//! - `data` - Data API (user positions, trades)
//! - `gamma` - Gamma API (events, markets, sports)
//! - `rtds` - RTDS (Real-Time Data Service)
//!
//! ## External APIs
//! - `cmc` - CoinMarketCap API (requires CMC_API_KEY)
//! - `cg` - CoinGecko API (requires CG_API_KEY)
//! - `alt` - Alternative.me API (free, no API key required)

use clap::Subcommand;

use super::{alternativeme, clob, clob_ws, cmc, coingecko, data, gamma, rtds};

/// Data Source commands (Polymarket and external data providers)
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
pub enum DsCommands {
    // Polymarket APIs
    /// Polymarket CLOB (Central Limit Order Book) API
    #[command(subcommand)]
    Clob(clob::ClobCommands),
    /// Polymarket CLOB WebSocket for real-time updates
    #[command(subcommand)]
    ClobWs(clob_ws::ClobWsCommands),
    /// Polymarket Data API (user positions, trades, portfolio)
    #[command(subcommand)]
    Data(data::DataCommands),
    /// Polymarket Gamma API (events, markets, sports)
    #[command(subcommand)]
    Gamma(gamma::GammaCommands),
    /// Polymarket RTDS (Real-Time Data Service)
    #[command(subcommand)]
    Rtds(rtds::RtdsCommands),

    // External APIs
    /// CoinMarketCap API commands (requires CMC_API_KEY)
    #[command(subcommand)]
    Cmc(cmc::CmcCommands),
    /// CoinGecko API commands (requires CG_API_KEY)
    #[command(subcommand)]
    Cg(coingecko::CgCommands),
    /// Alternative.me API commands (free, no API key)
    #[command(subcommand)]
    Alt(alternativeme::AlternativeMeCommands),
}

pub async fn handle(command: &DsCommands) -> anyhow::Result<()> {
    match command {
        // Polymarket APIs
        DsCommands::Clob(cmd) => clob::handle(cmd).await,
        DsCommands::ClobWs(cmd) => clob_ws::handle(cmd).await,
        DsCommands::Data(cmd) => data::handle(cmd).await,
        DsCommands::Gamma(cmd) => gamma::handle(cmd).await,
        DsCommands::Rtds(cmd) => rtds::handle(cmd).await,
        // External APIs
        DsCommands::Cmc(cmd) => cmc::handle(cmd).await,
        DsCommands::Cg(cmd) => coingecko::handle(cmd).await,
        DsCommands::Alt(cmd) => alternativeme::handle(cmd).await,
    }
}
