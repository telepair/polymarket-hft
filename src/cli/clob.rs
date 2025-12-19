use crate::cli::common::write_json_output;

use clap::{Args, Subcommand};
use polymarket_hft::client::polymarket::clob::Client;

#[allow(clippy::enum_variant_names)] // All variants are API commands with 'Get' prefix
#[derive(Subcommand)]
pub enum ClobCommands {
    // ========== OrderBook commands ==========
    /// Get order book summary for a token
    GetOrderBook {
        /// Token ID
        #[arg(short, long, required = true)]
        token_id: String,
    },
    /// Get order book summaries for multiple tokens
    GetOrderBooks {
        /// Token IDs (can be specified multiple times)
        #[arg(short, long, required = true)]
        token_id: Vec<String>,
        /// Side (BUY or SELL)  
        #[arg(short, long)]
        side: Option<String>,
    },
    // ========== Pricing commands ==========
    /// Get market price for a token and side
    GetMarketPrice {
        /// Token ID
        #[arg(short, long, required = true)]
        token_id: String,
        /// Side (BUY or SELL)
        #[arg(short, long, required = true)]
        side: String,
    },
    /// Get all market prices
    GetMarketPrices,
    /// Get midpoint price for a token
    GetMidpointPrice {
        /// Token ID
        #[arg(short, long, required = true)]
        token_id: String,
    },
    /// Get price history for a token
    GetPriceHistory {
        #[command(flatten)]
        params: GetPriceHistoryArgs,
    },
    // ========== Spreads commands ==========
    /// Get bid-ask spreads for tokens
    GetSpreads {
        /// Token IDs (can be specified multiple times)
        #[arg(short, long, required = true)]
        token_id: Vec<String>,
    },
}

#[derive(Args, Debug, Clone)]
pub struct GetPriceHistoryArgs {
    /// The CLOB token ID (market)
    #[arg(short, long, required = true)]
    pub market: String,
    /// Start timestamp (Unix timestamp in UTC)
    #[arg(long)]
    pub start_ts: Option<i64>,
    /// End timestamp (Unix timestamp in UTC)
    #[arg(long)]
    pub end_ts: Option<i64>,
    /// Interval (1m, 1h, 6h, 1d, 1w, max). Mutually exclusive with start_ts/end_ts.
    #[arg(short, long)]
    pub interval: Option<String>,
    /// Resolution of the data, in minutes
    #[arg(short, long)]
    pub fidelity: Option<i32>,
}

pub async fn handle(command: &ClobCommands) -> anyhow::Result<()> {
    let client = Client::new();

    match command {
        // ========== OrderBook commands ==========
        ClobCommands::GetOrderBook { token_id } => {
            let order_book = client.get_order_book(token_id).await?;
            write_json_output(&order_book)?;
        }
        ClobCommands::GetOrderBooks { token_id, side } => {
            let parsed_side = side
                .as_ref()
                .map(|s| s.parse::<polymarket_hft::client::polymarket::clob::Side>())
                .transpose()
                .map_err(|e| anyhow::anyhow!("invalid --side: {}", e))?;

            let request: Vec<polymarket_hft::client::polymarket::clob::GetOrderBooksRequestItem> =
                token_id
                    .iter()
                    .map(
                        |id| polymarket_hft::client::polymarket::clob::GetOrderBooksRequestItem {
                            token_id: id.clone(),
                            side: parsed_side,
                        },
                    )
                    .collect();
            let order_books = client.get_order_books(&request).await?;
            write_json_output(&order_books)?;
        }
        // ========== Pricing commands ==========
        ClobCommands::GetMarketPrice { token_id, side } => {
            let parsed_side = side
                .parse::<polymarket_hft::client::polymarket::clob::Side>()
                .map_err(|e| anyhow::anyhow!("invalid --side: {}", e))?;
            let price = client.get_market_price(token_id, parsed_side).await?;
            write_json_output(&price)?;
        }
        ClobCommands::GetMarketPrices => {
            let prices = client.get_market_prices().await?;
            write_json_output(&prices)?;
        }
        ClobCommands::GetMidpointPrice { token_id } => {
            let midpoint = client.get_midpoint_price(token_id).await?;
            write_json_output(&midpoint)?;
        }
        ClobCommands::GetPriceHistory { params } => {
            let parsed_interval = params
                .interval
                .as_ref()
                .map(|s| {
                    s.parse::<polymarket_hft::client::polymarket::clob::PriceHistoryInterval>()
                })
                .transpose()
                .map_err(|e| anyhow::anyhow!("invalid --interval: {}", e))?;

            let history = client
                .get_price_history(
                    polymarket_hft::client::polymarket::clob::GetPriceHistoryRequest {
                        market: &params.market,
                        start_ts: params.start_ts,
                        end_ts: params.end_ts,
                        interval: parsed_interval,
                        fidelity: params.fidelity,
                    },
                )
                .await?;
            write_json_output(&history)?;
        }
        // ========== Spreads commands ==========
        ClobCommands::GetSpreads { token_id } => {
            let request: Vec<polymarket_hft::client::polymarket::clob::SpreadRequest> = token_id
                .iter()
                .map(
                    |id| polymarket_hft::client::polymarket::clob::SpreadRequest {
                        token_id: id.clone(),
                        side: None,
                    },
                )
                .collect();
            let spreads = client.get_spreads(&request).await?;
            write_json_output(&spreads)?;
        }
    }

    Ok(())
}
