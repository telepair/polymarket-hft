//! CLOB WebSocket CLI commands.

use std::time::Duration;

use crate::cli::common::write_json_output;
use clap::{Args, Subcommand};
use polymarket_hft::client::polymarket::clob::ws::{ClobWsClient, WsAuth, WsMessage};

#[derive(Subcommand)]
pub enum ClobWsCommands {
    /// Subscribe to market channel for order book and price updates
    Market {
        #[command(flatten)]
        params: MarketArgs,
    },
    /// Subscribe to user channel for order and trade updates (requires auth)
    User {
        #[command(flatten)]
        params: UserArgs,
    },
}

#[derive(Args, Debug, Clone)]
pub struct MarketArgs {
    /// Asset IDs (token IDs) to subscribe to, comma-separated
    #[arg(short, long, required = true, value_delimiter = ',')]
    pub asset_ids: Vec<String>,

    /// Maximum number of messages to receive (0 for unlimited)
    #[arg(short = 'n', long, default_value = "10")]
    pub max_messages: usize,

    /// Timeout in seconds (0 for unlimited)
    #[arg(long, default_value = "60")]
    pub timeout: u64,

    /// Output format: json (default) or compact
    #[arg(short, long, default_value = "json")]
    pub output: String,
}

#[derive(Args, Debug, Clone)]
pub struct UserArgs {
    /// Market IDs (condition IDs) to subscribe to, comma-separated
    #[arg(short, long, required = true, value_delimiter = ',')]
    pub market_ids: Vec<String>,

    /// API key (reads from POLY_API_KEY env var if not provided)
    #[arg(long, env = "POLY_API_KEY")]
    pub api_key: String,

    /// API secret (reads from POLY_API_SECRET env var if not provided)
    #[arg(long, env = "POLY_API_SECRET")]
    pub api_secret: String,

    /// API passphrase (reads from POLY_PASSPHRASE env var if not provided)
    #[arg(long, env = "POLY_PASSPHRASE")]
    pub passphrase: String,

    /// Maximum number of messages to receive (0 for unlimited)
    #[arg(short = 'n', long, default_value = "10")]
    pub max_messages: usize,

    /// Timeout in seconds (0 for unlimited)
    #[arg(long, default_value = "60")]
    pub timeout: u64,

    /// Output format: json (default) or compact
    #[arg(short, long, default_value = "json")]
    pub output: String,
}

pub async fn handle(command: &ClobWsCommands) -> anyhow::Result<()> {
    match command {
        ClobWsCommands::Market { params } => {
            subscribe_market(params).await?;
        }
        ClobWsCommands::User { params } => {
            subscribe_user(params).await?;
        }
    }

    Ok(())
}

async fn subscribe_market(params: &MarketArgs) -> anyhow::Result<()> {
    let mut client = ClobWsClient::builder().auto_reconnect(true).build();

    eprintln!("Connecting to CLOB WebSocket (market channel)...");
    client.subscribe_market(params.asset_ids.clone()).await?;
    eprintln!("Connected and subscribed! Waiting for messages...\n");

    receive_messages(
        &mut client,
        params.max_messages,
        params.timeout,
        &params.output,
    )
    .await
}

async fn subscribe_user(params: &UserArgs) -> anyhow::Result<()> {
    let auth = WsAuth::new(&params.api_key, &params.api_secret, &params.passphrase);

    let mut client = ClobWsClient::builder().auto_reconnect(true).build();

    eprintln!("Connecting to CLOB WebSocket (user channel)...");
    client
        .subscribe_user(params.market_ids.clone(), auth)
        .await?;
    eprintln!("Connected and subscribed! Waiting for messages...\n");

    receive_messages(
        &mut client,
        params.max_messages,
        params.timeout,
        &params.output,
    )
    .await
}

async fn receive_messages(
    client: &mut ClobWsClient,
    max_messages: usize,
    timeout_secs: u64,
    output: &str,
) -> anyhow::Result<()> {
    let timeout = if timeout_secs > 0 {
        Some(Duration::from_secs(timeout_secs))
    } else {
        None
    };

    let start = std::time::Instant::now();
    let mut count = 0;

    loop {
        // Check timeout
        #[allow(clippy::collapsible_if)]
        if let Some(t) = timeout {
            if start.elapsed() > t {
                eprintln!("\nTimeout reached after {} seconds", timeout_secs);
                break;
            }
        }

        // Check message limit
        if max_messages > 0 && count >= max_messages {
            eprintln!("\nReached {} messages limit", max_messages);
            break;
        }

        // Wait for message with a timeout
        let msg_future = client.next_message();
        let result = tokio::time::timeout(Duration::from_secs(5), msg_future).await;

        match result {
            Ok(Some(msg)) => {
                count += 1;

                if output == "compact" {
                    print_compact(&msg)?;
                } else {
                    print_json(&msg)?;
                }
            }
            Ok(None) => {
                eprintln!("Connection closed");
                break;
            }
            Err(_) => {
                // Timeout on individual message, continue waiting
                continue;
            }
        }
    }

    eprintln!("\nReceived {} messages total", count);
    client.disconnect().await;

    Ok(())
}

fn print_compact(msg: &WsMessage) -> anyhow::Result<()> {
    match msg {
        WsMessage::Book(book) => {
            println!(
                "[book] asset={} bids={} asks={}",
                book.asset_id,
                book.bids.len(),
                book.asks.len()
            );
        }
        WsMessage::PriceChange(pc) => {
            println!(
                "[price_change] market={} changes={}",
                pc.market,
                pc.price_changes.len()
            );
        }
        WsMessage::TickSizeChange(tsc) => {
            println!(
                "[tick_size_change] asset={} {} -> {}",
                tsc.asset_id, tsc.old_tick_size, tsc.new_tick_size
            );
        }
        WsMessage::LastTradePrice(ltp) => {
            println!(
                "[last_trade_price] asset={} price={} size={}",
                ltp.asset_id, ltp.price, ltp.size
            );
        }
        WsMessage::Trade(trade) => {
            println!(
                "[trade] id={} status={:?} size={} price={}",
                trade.id, trade.status, trade.size, trade.price
            );
        }
        WsMessage::Order(order) => {
            println!(
                "[order] id={} type={:?} size_matched={}",
                order.id, order.order_type, order.size_matched
            );
        }
        WsMessage::Unknown(value) => {
            println!("[unknown] {}", serde_json::to_string(value)?);
        }
    }
    Ok(())
}

fn print_json(msg: &WsMessage) -> anyhow::Result<()> {
    match msg {
        WsMessage::Book(book) => write_json_output(book),
        WsMessage::PriceChange(pc) => write_json_output(pc),
        WsMessage::TickSizeChange(tsc) => write_json_output(tsc),
        WsMessage::LastTradePrice(ltp) => write_json_output(ltp),
        WsMessage::Trade(trade) => write_json_output(trade),
        WsMessage::Order(order) => write_json_output(order),
        WsMessage::Unknown(value) => write_json_output(value),
    }
}
