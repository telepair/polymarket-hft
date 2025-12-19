use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod cli;

use cli::{clob, clob_ws, cmc, data, gamma, rtds};

#[derive(Parser)]
#[command(name = "polymarket")]
#[command(about = "A CLI for Polymarket APIs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[allow(clippy::large_enum_variant)] // Clap subcommands hold substantial payloads; parsed once
#[derive(Subcommand)]
enum Commands {
    /// CLOB API commands
    #[command(subcommand)]
    Clob(clob::ClobCommands),
    /// CLOB WebSocket commands
    #[command(subcommand)]
    ClobWs(clob_ws::ClobWsCommands),
    /// Data API commands
    #[command(subcommand)]
    Data(data::DataCommands),
    /// Gamma API commands
    #[command(subcommand)]
    Gamma(gamma::GammaCommands),
    /// RTDS (Real-Time Data Service) commands
    #[command(subcommand)]
    Rtds(rtds::RtdsCommands),
    /// CoinMarketCap API commands (requires CMC_API_KEY)
    #[command(subcommand)]
    Cmc(cmc::CmcCommands),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber with env-filter support
    // Set RUST_LOG=trace to see HTTP request/response logs
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Clob(clob_cmd) => {
            clob::handle(clob_cmd).await?;
        }
        Commands::ClobWs(clob_ws_cmd) => {
            clob_ws::handle(clob_ws_cmd).await?;
        }
        Commands::Data(data_cmd) => {
            data::handle(data_cmd).await?;
        }
        Commands::Gamma(gamma_cmd) => {
            gamma::handle(gamma_cmd).await?;
        }
        Commands::Rtds(rtds_cmd) => {
            rtds::handle(rtds_cmd).await?;
        }
        Commands::Cmc(cmc_cmd) => {
            cmc::handle(cmc_cmd).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_health_command() {
        let cli = Cli::parse_from(["polymarket", "data", "health"]);
        match cli.command {
            Commands::Data(data::DataCommands::Health) => {}
            _ => panic!("expected health command"),
        }
    }
}
