use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod cli;

use cli::{ds, serve};

#[derive(Parser)]
#[command(name = "polymarket")]
#[command(about = "A CLI for Polymarket HFT", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Data source commands (Polymarket and external APIs)
    #[command(subcommand)]
    Ds(Box<ds::DsCommands>),

    /// Start the web server
    Serve(serve::ServeArgs),
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
        Commands::Ds(ds_cmd) => {
            ds::handle(ds_cmd).await?;
        }
        Commands::Serve(args) => {
            serve::handle(args).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cli::data;

    #[test]
    fn parses_health_command() {
        let cli = Cli::parse_from(["polymarket", "ds", "data", "health"]);
        match cli.command {
            Commands::Ds(cmd) => match *cmd {
                ds::DsCommands::Data(data::DataCommands::Health) => {}
                _ => panic!("expected health command"),
            },
            _ => panic!("expected ds command"),
        }
    }
}
