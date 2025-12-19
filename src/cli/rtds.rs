//! RTDS (Real-Time Data Service) CLI commands.

use std::time::Duration;

use crate::cli::common::write_json_output;
use clap::{Args, Subcommand};
use polymarket_hft::client::polymarket::rtds::{ClobAuth, RtdsClient, Subscription};

#[derive(Subcommand)]
pub enum RtdsCommands {
    /// Subscribe to real-time data streams and print messages
    Subscribe {
        #[command(flatten)]
        params: SubscribeArgs,
    },
}

#[derive(Args, Debug, Clone)]
pub struct SubscribeArgs {
    /// Topic to subscribe to (activity, comments, rfq, crypto_prices,
    /// crypto_prices_chainlink, equity_prices, clob_user, clob_market)
    #[arg(short, long, required = true)]
    pub topic: String,

    /// Message type (use "*" for all types)
    #[arg(short = 'T', long, default_value = "*")]
    pub message_type: String,

    /// Optional filter in JSON format (e.g., '{"symbol":"BTCUSDT"}')
    #[arg(short, long)]
    pub filter: Option<String>,

    /// CLOB API key (for clob_user topic)
    #[arg(long, env = "POLY_API_KEY")]
    pub clob_key: Option<String>,

    /// CLOB API secret (for clob_user topic)
    #[arg(long, env = "POLY_API_SECRET")]
    pub clob_secret: Option<String>,

    /// CLOB API passphrase (for clob_user topic)
    #[arg(long, env = "POLY_PASSPHRASE")]
    pub clob_passphrase: Option<String>,

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

pub async fn handle(command: &RtdsCommands) -> anyhow::Result<()> {
    match command {
        RtdsCommands::Subscribe { params } => {
            subscribe(params).await?;
        }
    }

    Ok(())
}

async fn subscribe(params: &SubscribeArgs) -> anyhow::Result<()> {
    let mut client = RtdsClient::builder().auto_reconnect(true).build();

    eprintln!("Connecting to RTDS...");
    client.connect().await?;
    eprintln!("Connected!");

    // Build subscription
    let mut subscription = Subscription::new(&params.topic, &params.message_type);

    if let Some(filter) = &params.filter {
        subscription = subscription.with_filter(filter);
    }

    // Add CLOB auth if provided
    if let (Some(key), Some(secret), Some(passphrase)) = (
        &params.clob_key,
        &params.clob_secret,
        &params.clob_passphrase,
    ) {
        subscription = subscription.with_clob_auth(ClobAuth::new(key, secret, passphrase));
    }

    eprintln!(
        "Subscribing to topic={}, type={}...",
        params.topic, params.message_type
    );
    client.subscribe(vec![subscription]).await?;
    eprintln!("Subscribed! Waiting for messages...\n");

    // Set up timeout
    let timeout = if params.timeout > 0 {
        Some(Duration::from_secs(params.timeout))
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
                eprintln!("\nTimeout reached after {} seconds", params.timeout);
                break;
            }
        }

        // Check message limit
        if params.max_messages > 0 && count >= params.max_messages {
            eprintln!("\nReached {} messages limit", params.max_messages);
            break;
        }

        // Wait for message with a timeout
        let msg_future = client.next_message();
        let result = tokio::time::timeout(Duration::from_secs(5), msg_future).await;

        match result {
            Ok(Some(msg)) => {
                count += 1;

                if params.output == "compact" {
                    println!(
                        "[{}] {}/{}: {}",
                        msg.timestamp,
                        msg.topic,
                        msg.message_type,
                        serde_json::to_string(&msg.payload)?
                    );
                } else {
                    write_json_output(&msg)?;
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
