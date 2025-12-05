use clap::{Parser, Subcommand};

mod commands;

use commands::data;

#[derive(Parser)]
#[command(name = "polymarket")]
#[command(about = "A CLI for Polymarket APIs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Data API commands
    #[command(subcommand)]
    Data(data::DataCommands),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Data(data_cmd) => {
            data::handle(data_cmd).await?;
        }
    }

    Ok(())
}
