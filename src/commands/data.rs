use std::io::{self, Write};

use clap::Subcommand;
use polymarket_hft::data::Client;
use thiserror::Error;

/// Error type for CLI operations.
#[derive(Error, Debug)]
pub enum CliError {
    #[error("{0}")]
    Sdk(#[from] polymarket_hft::error::PolymarketError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Subcommand)]
pub enum DataCommands {
    // ========== User-related commands ==========
    /// Get current positions for a user
    GetUserPositions {
        /// User Profile Address (0x-prefixed, 40 hex chars)
        #[arg(short, long, required = true)]
        user: String,
        /// Market condition IDs to filter by (0x-prefixed, 64 hex chars each)
        #[arg(short, long)]
        market: Option<Vec<String>>,
        /// Event IDs to filter by
        #[arg(short, long)]
        event_id: Option<Vec<i64>>,
        /// Minimum position size (>= 0)
        #[arg(long)]
        size_threshold: Option<f64>,
        /// Filter for redeemable positions
        #[arg(long)]
        redeemable: Option<bool>,
        /// Filter for mergeable positions
        #[arg(long)]
        mergeable: Option<bool>,
        /// Limit results (0-500, default: 100)
        #[arg(short, long)]
        limit: Option<i32>,
        /// Offset for pagination (0-10000, default: 0)
        #[arg(short, long)]
        offset: Option<i32>,
        /// Sort field (CURRENT, INITIAL, TOKENS, CASHPNL, PERCENTPNL, TITLE, RESOLVING, PRICE, AVGPRICE)
        #[arg(long)]
        sort_by: Option<String>,
        /// Sort direction (ASC or DESC)
        #[arg(long)]
        sort_direction: Option<String>,
        /// Title filter (max 160 chars)
        #[arg(short, long)]
        title: Option<String>,
    },
    /// Get closed positions for a user
    GetUserClosedPositions {
        /// User Profile Address (0x-prefixed, 40 hex chars)
        #[arg(short, long, required = true)]
        user: String,
        /// Market condition IDs to filter by (0x-prefixed, 64 hex chars each)
        #[arg(short, long)]
        market: Option<Vec<String>>,
        /// Title filter (max 100 chars)
        #[arg(short, long)]
        title: Option<String>,
        /// Event IDs to filter by (>= 1)
        #[arg(short, long)]
        event_id: Option<Vec<i64>>,
        /// Limit results (0-50, default: 10)
        #[arg(short, long)]
        limit: Option<i32>,
        /// Offset for pagination (0-100000, default: 0)
        #[arg(short, long)]
        offset: Option<i32>,
        /// Sort field (REALIZEDPNL, TITLE, PRICE, AVGPRICE, TIMESTAMP)
        #[arg(long)]
        sort_by: Option<String>,
        /// Sort direction (ASC or DESC)
        #[arg(long)]
        sort_direction: Option<String>,
    },
    /// Get total value of a user's positions
    GetUserPortfolioValue {
        /// User Profile Address (0x-prefixed, 40 hex chars)
        #[arg(short, long, required = true)]
        user: String,
        /// Optional market IDs to filter by (0x-prefixed, 64 hex chars each)
        #[arg(short, long)]
        market: Option<Vec<String>>,
    },
    /// Get total number of markets a user has traded
    GetUserTradedMarkets {
        /// User Profile Address (0x-prefixed, 40 hex chars)
        #[arg(short, long, required = true)]
        user: String,
    },
    /// Get on-chain activity for a user
    GetUserActivity {
        /// User Profile Address (0x-prefixed, 40 hex chars)
        #[arg(short, long, required = true)]
        user: String,
        /// Limit results (0-500, default: 100)
        #[arg(short, long)]
        limit: Option<i32>,
        /// Offset for pagination (0-10000, default: 0)
        #[arg(short, long)]
        offset: Option<i32>,
        /// Market condition IDs to filter by (0x-prefixed, 64 hex chars each). Mutually exclusive with event_id.
        #[arg(short, long)]
        market: Option<Vec<String>>,
        /// Event IDs to filter by (>= 1). Mutually exclusive with market.
        #[arg(short, long)]
        event_id: Option<Vec<i64>>,
        /// Activity types to filter by (TRADE, SPLIT, MERGE, REDEEM, REWARD, CONVERSION)
        #[arg(short = 't', long = "type")]
        activity_type: Option<Vec<String>>,
        /// Start timestamp (>= 0)
        #[arg(long)]
        start: Option<i64>,
        /// End timestamp (>= 0)
        #[arg(long)]
        end: Option<i64>,
        /// Sort field (TIMESTAMP, TOKENS, CASH)
        #[arg(long)]
        sort_by: Option<String>,
        /// Sort direction (ASC or DESC)
        #[arg(long)]
        sort_direction: Option<String>,
        /// Trade side filter (BUY or SELL)
        #[arg(long)]
        side: Option<String>,
    },
    /// Get trades for a user or markets
    GetTrades {
        /// User Profile Address (0x-prefixed, 40 hex chars)
        #[arg(short, long)]
        user: Option<String>,
        /// Market condition IDs to filter by (0x-prefixed, 64 hex chars each). Mutually exclusive with event_id.
        #[arg(short, long)]
        market: Option<Vec<String>>,
        /// Event IDs to filter by (>= 1). Mutually exclusive with market.
        #[arg(short, long)]
        event_id: Option<Vec<i64>>,
        /// Limit results (0-10000, default: 100)
        #[arg(short, long)]
        limit: Option<i32>,
        /// Offset for pagination (0-10000, default: 0)
        #[arg(short, long)]
        offset: Option<i32>,
        /// Filter for taker-only trades
        #[arg(long)]
        taker_only: Option<bool>,
        /// Filter type (CASH or TOKENS). Must be provided with filter_amount.
        #[arg(long)]
        filter_type: Option<String>,
        /// Filter amount (>= 0). Must be provided with filter_type.
        #[arg(long)]
        filter_amount: Option<f64>,
        /// Trade side filter (BUY or SELL)
        #[arg(short, long)]
        side: Option<String>,
    },
    // ========== Market/System commands ==========
    /// Check API health
    Health,
    /// Get top holders for markets
    GetMarketTopHolders {
        /// Market IDs (0x-prefixed, 64 hex chars each)
        #[arg(short, long, required = true)]
        market: Vec<String>,
        /// Limit results (0-500, default: 100)
        #[arg(short, long)]
        limit: Option<i32>,
        /// Minimum balance filter (0-999999, default: 1)
        #[arg(long)]
        min_balance: Option<i32>,
    },
    /// Get open interest for markets
    GetOpenInterest {
        /// Market IDs (0x-prefixed, 64 hex chars each)
        #[arg(short, long, required = true)]
        market: Vec<String>,
    },
    /// Get live volume for an event
    GetEventLiveVolume {
        /// Event ID (must be >= 1)
        #[arg(short, long, required = true)]
        id: i64,
    },
}

/// Writes JSON output to stdout using streaming writer for better performance.
fn write_json_output<T: serde::Serialize>(value: &T) -> Result<(), CliError> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(&mut handle, value)?;
    writeln!(handle)?;
    Ok(())
}

pub async fn handle(command: &DataCommands) -> Result<(), CliError> {
    let client = Client::new();

    match command {
        // ========== User-related commands ==========
        DataCommands::GetUserPositions {
            user,
            market,
            event_id,
            size_threshold,
            redeemable,
            mergeable,
            limit,
            offset,
            sort_by,
            sort_direction,
            title,
        } => {
            let market_refs: Option<Vec<&str>> = market
                .as_ref()
                .map(|m| m.iter().map(|s| s.as_str()).collect());
            let event_ids: Option<Vec<i64>> = event_id.clone();

            // Parse sort_by if provided
            let parsed_sort_by = sort_by
                .as_ref()
                .map(|s| s.parse::<polymarket_hft::data::PositionSortBy>())
                .transpose()
                .map_err(|e| {
                    CliError::Sdk(polymarket_hft::error::PolymarketError::bad_request(
                        format!("invalid --sort-by: {}", e),
                    ))
                })?;

            // Parse sort_direction if provided
            let parsed_sort_direction = sort_direction
                .as_ref()
                .map(|s| s.parse::<polymarket_hft::data::SortDirection>())
                .transpose()
                .map_err(|e| {
                    CliError::Sdk(polymarket_hft::error::PolymarketError::bad_request(
                        format!("invalid --sort-direction: {}", e),
                    ))
                })?;

            let positions = client
                .get_user_positions(polymarket_hft::data::GetUserPositionsRequest {
                    user,
                    markets: market_refs.as_deref(),
                    event_ids: event_ids.as_deref(),
                    size_threshold: *size_threshold,
                    redeemable: *redeemable,
                    mergeable: *mergeable,
                    limit: *limit,
                    offset: *offset,
                    sort_by: parsed_sort_by,
                    sort_direction: parsed_sort_direction,
                    title: title.as_deref(),
                })
                .await?;
            write_json_output(&positions)?;
        }
        DataCommands::GetUserClosedPositions {
            user,
            market,
            title,
            event_id,
            limit,
            offset,
            sort_by,
            sort_direction,
        } => {
            let market_refs: Option<Vec<&str>> = market
                .as_ref()
                .map(|m| m.iter().map(|s| s.as_str()).collect());
            let event_ids: Option<Vec<i64>> = event_id.clone();

            // Parse sort_by if provided
            let parsed_sort_by = sort_by
                .as_ref()
                .map(|s| s.parse::<polymarket_hft::data::ClosedPositionSortBy>())
                .transpose()
                .map_err(|e| {
                    CliError::Sdk(polymarket_hft::error::PolymarketError::bad_request(
                        format!("invalid --sort-by: {}", e),
                    ))
                })?;

            // Parse sort_direction if provided
            let parsed_sort_direction = sort_direction
                .as_ref()
                .map(|s| s.parse::<polymarket_hft::data::SortDirection>())
                .transpose()
                .map_err(|e| {
                    CliError::Sdk(polymarket_hft::error::PolymarketError::bad_request(
                        format!("invalid --sort-direction: {}", e),
                    ))
                })?;

            let positions = client
                .get_user_closed_positions(polymarket_hft::data::GetUserClosedPositionsRequest {
                    user,
                    markets: market_refs.as_deref(),
                    title: title.as_deref(),
                    event_ids: event_ids.as_deref(),
                    limit: *limit,
                    offset: *offset,
                    sort_by: parsed_sort_by,
                    sort_direction: parsed_sort_direction,
                })
                .await?;
            write_json_output(&positions)?;
        }
        DataCommands::GetUserPortfolioValue { user, market } => {
            let market_refs: Option<Vec<&str>> = market
                .as_ref()
                .map(|m| m.iter().map(|s| s.as_str()).collect());
            let values = client
                .get_user_portfolio_value(user, market_refs.as_deref())
                .await?;
            write_json_output(&values)?;
        }
        DataCommands::GetUserTradedMarkets { user } => {
            let traded = client.get_user_traded_markets(user).await?;
            write_json_output(&traded)?;
        }
        DataCommands::GetUserActivity {
            user,
            limit,
            offset,
            market,
            event_id,
            activity_type,
            start,
            end,
            sort_by,
            sort_direction,
            side,
        } => {
            let market_refs: Option<Vec<&str>> = market
                .as_ref()
                .map(|m| m.iter().map(|s| s.as_str()).collect());
            let event_ids: Option<Vec<i64>> = event_id.clone();

            // Parse activity_types if provided
            let parsed_activity_types: Option<Vec<polymarket_hft::data::ActivityType>> =
                activity_type
                    .as_ref()
                    .map(|types| {
                        types
                            .iter()
                            .map(|s| s.parse::<polymarket_hft::data::ActivityType>())
                            .collect::<std::result::Result<Vec<_>, _>>()
                    })
                    .transpose()
                    .map_err(|e| {
                        CliError::Sdk(polymarket_hft::error::PolymarketError::bad_request(
                            format!("invalid --type: {}", e),
                        ))
                    })?;

            // Parse sort_by if provided
            let parsed_sort_by = sort_by
                .as_ref()
                .map(|s| s.parse::<polymarket_hft::data::ActivitySortBy>())
                .transpose()
                .map_err(|e| {
                    CliError::Sdk(polymarket_hft::error::PolymarketError::bad_request(
                        format!("invalid --sort-by: {}", e),
                    ))
                })?;

            // Parse sort_direction if provided
            let parsed_sort_direction = sort_direction
                .as_ref()
                .map(|s| s.parse::<polymarket_hft::data::SortDirection>())
                .transpose()
                .map_err(|e| {
                    CliError::Sdk(polymarket_hft::error::PolymarketError::bad_request(
                        format!("invalid --sort-direction: {}", e),
                    ))
                })?;

            // Parse side if provided
            let parsed_side = side
                .as_ref()
                .map(|s| s.parse::<polymarket_hft::data::TradeSide>())
                .transpose()
                .map_err(|e| {
                    CliError::Sdk(polymarket_hft::error::PolymarketError::bad_request(
                        format!("invalid --side: {}", e),
                    ))
                })?;

            let activity = client
                .get_user_activity(polymarket_hft::data::GetUserActivityRequest {
                    user,
                    limit: *limit,
                    offset: *offset,
                    markets: market_refs.as_deref(),
                    event_ids: event_ids.as_deref(),
                    activity_types: parsed_activity_types.as_deref(),
                    start: *start,
                    end: *end,
                    sort_by: parsed_sort_by,
                    sort_direction: parsed_sort_direction,
                    side: parsed_side,
                })
                .await?;
            write_json_output(&activity)?;
        }
        DataCommands::GetTrades {
            user,
            market,
            event_id,
            limit,
            offset,
            taker_only,
            filter_type,
            filter_amount,
            side,
        } => {
            let market_refs: Option<Vec<&str>> = market
                .as_ref()
                .map(|m| m.iter().map(|s| s.as_str()).collect());
            let event_ids: Option<Vec<i64>> = event_id.clone();

            // Parse filter_type if provided
            let parsed_filter_type = filter_type
                .as_ref()
                .map(|s| s.parse::<polymarket_hft::data::TradeFilterType>())
                .transpose()
                .map_err(|e| {
                    CliError::Sdk(polymarket_hft::error::PolymarketError::bad_request(
                        format!("invalid --filter-type: {}", e),
                    ))
                })?;

            // Parse side if provided
            let parsed_side = side
                .as_ref()
                .map(|s| s.parse::<polymarket_hft::data::TradeSide>())
                .transpose()
                .map_err(|e| {
                    CliError::Sdk(polymarket_hft::error::PolymarketError::bad_request(
                        format!("invalid --side: {}", e),
                    ))
                })?;

            let trades = client
                .get_trades(polymarket_hft::data::GetTradesRequest {
                    limit: *limit,
                    offset: *offset,
                    taker_only: *taker_only,
                    filter_type: parsed_filter_type,
                    filter_amount: *filter_amount,
                    markets: market_refs.as_deref(),
                    event_ids: event_ids.as_deref(),
                    user: user.as_deref(),
                    side: parsed_side,
                })
                .await?;
            write_json_output(&trades)?;
        }
        // ========== Market/System commands ==========
        DataCommands::Health => {
            let health = client.health().await?;
            write_json_output(&health)?;
        }
        DataCommands::GetMarketTopHolders {
            market,
            limit,
            min_balance,
        } => {
            let market_refs: Vec<&str> = market.iter().map(|s| s.as_str()).collect();
            let holders = client
                .get_market_top_holders(&market_refs, *limit, *min_balance)
                .await?;
            write_json_output(&holders)?;
        }
        DataCommands::GetOpenInterest { market } => {
            let market_refs: Vec<&str> = market.iter().map(|s| s.as_str()).collect();
            let oi_list = client.get_open_interest(&market_refs).await?;
            write_json_output(&oi_list)?;
        }
        DataCommands::GetEventLiveVolume { id } => {
            let volume = client.get_event_live_volume(*id).await?;
            write_json_output(&volume)?;
        }
    }

    Ok(())
}
