use clap::Args;
use std::path::PathBuf;

/// Arguments for the serve command.
#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Path to the configuration file (YAML).
    #[arg(short, long)]
    pub config: PathBuf,
}

/// Handle the serve command.
pub async fn handle(args: &ServeArgs) -> anyhow::Result<()> {
    polymarket_hft::serve::run(args.config.clone()).await
}
