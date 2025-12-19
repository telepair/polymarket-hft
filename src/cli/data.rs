//! Data API CLI module.
//!
//! This module provides CLI commands for interacting with the Polymarket Data API.

use crate::cli::common::write_json_output;
use clap::{Args, Subcommand};
use polymarket_hft::client::polymarket::data::Client;

// =============================================================================
// Commands
// =============================================================================

/// Data API CLI commands.
#[derive(Subcommand)]
pub enum DataCommands {
    // ========== User-related commands ==========
    /// Get current positions for a user
    GetUserPositions {
        #[command(flatten)]
        params: GetUserPositionsArgs,
    },
    /// Get closed positions for a user
    GetUserClosedPositions {
        #[command(flatten)]
        params: GetUserClosedPositionsArgs,
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
        #[command(flatten)]
        params: GetUserActivityArgs,
    },
    /// Get trades for a user or markets
    GetTrades {
        #[command(flatten)]
        params: GetTradesArgs,
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

#[derive(Args, Debug, Clone)]
pub struct GetUserPositionsArgs {
    /// User Profile Address (0x-prefixed, 40 hex chars)
    #[arg(short, long, required = true)]
    pub user: String,
    /// Market condition IDs to filter by (0x-prefixed, 64 hex chars each)
    #[arg(short, long)]
    pub market: Option<Vec<String>>,
    /// Event IDs to filter by
    #[arg(short, long)]
    pub event_id: Option<Vec<i64>>,
    /// Minimum position size (>= 0)
    #[arg(long)]
    pub size_threshold: Option<f64>,
    /// Filter for redeemable positions
    #[arg(long)]
    pub redeemable: Option<bool>,
    /// Filter for mergeable positions
    #[arg(long)]
    pub mergeable: Option<bool>,
    /// Limit results (0-500, default: 100)
    #[arg(short, long)]
    pub limit: Option<i32>,
    /// Offset for pagination (0-10000, default: 0)
    #[arg(short, long)]
    pub offset: Option<i32>,
    /// Sort field (CURRENT, INITIAL, TOKENS, CASHPNL, PERCENTPNL, TITLE, RESOLVING, PRICE, AVGPRICE)
    #[arg(long)]
    pub sort_by: Option<String>,
    /// Sort direction (ASC or DESC)
    #[arg(long)]
    pub sort_direction: Option<String>,
    /// Title filter (max 160 chars)
    #[arg(short, long)]
    pub title: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct GetUserClosedPositionsArgs {
    /// User Profile Address (0x-prefixed, 40 hex chars)
    #[arg(short, long, required = true)]
    pub user: String,
    /// Market condition IDs to filter by (0x-prefixed, 64 hex chars each)
    #[arg(short, long)]
    pub market: Option<Vec<String>>,
    /// Title filter (max 100 chars)
    #[arg(short, long)]
    pub title: Option<String>,
    /// Event IDs to filter by (>= 1)
    #[arg(short, long)]
    pub event_id: Option<Vec<i64>>,
    /// Limit results (0-50, default: 10)
    #[arg(short, long)]
    pub limit: Option<i32>,
    /// Offset for pagination (0-100000, default: 0)
    #[arg(short, long)]
    pub offset: Option<i32>,
    /// Sort field (REALIZEDPNL, TITLE, PRICE, AVGPRICE, TIMESTAMP)
    #[arg(long)]
    pub sort_by: Option<String>,
    /// Sort direction (ASC or DESC)
    #[arg(long)]
    pub sort_direction: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct GetUserActivityArgs {
    /// User Profile Address (0x-prefixed, 40 hex chars)
    #[arg(short, long, required = true)]
    pub user: String,
    /// Limit results (0-500, default: 100)
    #[arg(short, long)]
    pub limit: Option<i32>,
    /// Offset for pagination (0-10000, default: 0)
    #[arg(short, long)]
    pub offset: Option<i32>,
    /// Market condition IDs to filter by (0x-prefixed, 64 hex chars each). Mutually exclusive with event_id.
    #[arg(short, long)]
    pub market: Option<Vec<String>>,
    /// Event IDs to filter by (>= 1). Mutually exclusive with market.
    #[arg(short, long)]
    pub event_id: Option<Vec<i64>>,
    /// Activity types to filter by (TRADE, SPLIT, MERGE, REDEEM, REWARD, CONVERSION)
    #[arg(short = 't', long = "type")]
    pub activity_type: Option<Vec<String>>,
    /// Start timestamp (>= 0)
    #[arg(long)]
    pub start: Option<i64>,
    /// End timestamp (>= 0)
    #[arg(long)]
    pub end: Option<i64>,
    /// Sort field (TIMESTAMP, TOKENS, CASH)
    #[arg(long)]
    pub sort_by: Option<String>,
    /// Sort direction (ASC or DESC)
    #[arg(long)]
    pub sort_direction: Option<String>,
    /// Trade side filter (BUY or SELL)
    #[arg(long)]
    pub side: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct GetTradesArgs {
    /// User Profile Address (0x-prefixed, 40 hex chars)
    #[arg(short, long)]
    pub user: Option<String>,
    /// Market condition IDs to filter by (0x-prefixed, 64 hex chars each). Mutually exclusive with event_id.
    #[arg(short, long)]
    pub market: Option<Vec<String>>,
    /// Event IDs to filter by (>= 1). Mutually exclusive with market.
    #[arg(short, long)]
    pub event_id: Option<Vec<i64>>,
    /// Limit results (0-10000, default: 100)
    #[arg(short, long)]
    pub limit: Option<i32>,
    /// Offset for pagination (0-10000, default: 0)
    #[arg(short, long)]
    pub offset: Option<i32>,
    /// Filter for taker-only trades
    #[arg(long)]
    pub taker_only: Option<bool>,
    /// Filter type (CASH or TOKENS). Must be provided with filter_amount.
    #[arg(long)]
    pub filter_type: Option<String>,
    /// Filter amount (>= 0). Must be provided with filter_type.
    #[arg(long)]
    pub filter_amount: Option<f64>,
    /// Trade side filter (BUY or SELL)
    #[arg(short, long)]
    pub side: Option<String>,
}

// =============================================================================
// Handlers
// =============================================================================

/// Handle Data API CLI commands.
pub async fn handle(command: &DataCommands) -> anyhow::Result<()> {
    let client = Client::new();

    match command {
        // ========== User-related commands ==========
        DataCommands::GetUserPositions { params } => {
            handle_get_user_positions(&client, params).await?;
        }
        DataCommands::GetUserClosedPositions { params } => {
            handle_get_user_closed_positions(&client, params).await?;
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
        DataCommands::GetUserActivity { params } => {
            handle_get_user_activity(&client, params).await?;
        }
        DataCommands::GetTrades { params } => {
            handle_get_trades(&client, params).await?;
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

async fn handle_get_user_positions(
    client: &Client,
    params: &GetUserPositionsArgs,
) -> anyhow::Result<()> {
    let market_refs: Option<Vec<&str>> = params
        .market
        .as_ref()
        .map(|m| m.iter().map(|s| s.as_str()).collect());

    let parsed_sort_by = params
        .sort_by
        .as_ref()
        .map(|s| s.parse::<polymarket_hft::client::polymarket::data::PositionSortBy>())
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid --sort-by: {}", e))?;

    let parsed_sort_direction = params
        .sort_direction
        .as_ref()
        .map(|s| s.parse::<polymarket_hft::client::polymarket::data::SortDirection>())
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid --sort-direction: {}", e))?;

    let positions = client
        .get_user_positions(
            polymarket_hft::client::polymarket::data::GetUserPositionsRequest {
                user: params.user.as_str(),
                markets: market_refs.as_deref(),
                event_ids: params.event_id.as_deref(),
                size_threshold: params.size_threshold,
                redeemable: params.redeemable,
                mergeable: params.mergeable,
                limit: params.limit,
                offset: params.offset,
                sort_by: parsed_sort_by,
                sort_direction: parsed_sort_direction,
                title: params.title.as_deref(),
            },
        )
        .await?;
    write_json_output(&positions)?;
    Ok(())
}

async fn handle_get_user_closed_positions(
    client: &Client,
    params: &GetUserClosedPositionsArgs,
) -> anyhow::Result<()> {
    let market_refs: Option<Vec<&str>> = params
        .market
        .as_ref()
        .map(|m| m.iter().map(|s| s.as_str()).collect());

    let parsed_sort_by = params
        .sort_by
        .as_ref()
        .map(|s| s.parse::<polymarket_hft::client::polymarket::data::ClosedPositionSortBy>())
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid --sort-by: {}", e))?;

    let parsed_sort_direction = params
        .sort_direction
        .as_ref()
        .map(|s| s.parse::<polymarket_hft::client::polymarket::data::SortDirection>())
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid --sort-direction: {}", e))?;

    let positions = client
        .get_user_closed_positions(
            polymarket_hft::client::polymarket::data::GetUserClosedPositionsRequest {
                user: params.user.as_str(),
                markets: market_refs.as_deref(),
                title: params.title.as_deref(),
                event_ids: params.event_id.as_deref(),
                limit: params.limit,
                offset: params.offset,
                sort_by: parsed_sort_by,
                sort_direction: parsed_sort_direction,
            },
        )
        .await?;
    write_json_output(&positions)?;
    Ok(())
}

async fn handle_get_user_activity(
    client: &Client,
    params: &GetUserActivityArgs,
) -> anyhow::Result<()> {
    let market_refs: Option<Vec<&str>> = params
        .market
        .as_ref()
        .map(|m| m.iter().map(|s| s.as_str()).collect());

    let parsed_activity_types: Option<Vec<polymarket_hft::client::polymarket::data::ActivityType>> =
        params
            .activity_type
            .as_ref()
            .map(|types| {
                types
                    .iter()
                    .map(|s| s.parse::<polymarket_hft::client::polymarket::data::ActivityType>())
                    .collect::<std::result::Result<Vec<_>, _>>()
            })
            .transpose()
            .map_err(|e| anyhow::anyhow!("invalid --type: {}", e))?;

    let parsed_sort_by = params
        .sort_by
        .as_ref()
        .map(|s| s.parse::<polymarket_hft::client::polymarket::data::ActivitySortBy>())
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid --sort-by: {}", e))?;

    let parsed_sort_direction = params
        .sort_direction
        .as_ref()
        .map(|s| s.parse::<polymarket_hft::client::polymarket::data::SortDirection>())
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid --sort-direction: {}", e))?;

    let parsed_side = params
        .side
        .as_ref()
        .map(|s| s.parse::<polymarket_hft::client::polymarket::data::TradeSide>())
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid --side: {}", e))?;

    let activity = client
        .get_user_activity(
            polymarket_hft::client::polymarket::data::GetUserActivityRequest {
                user: params.user.as_str(),
                limit: params.limit,
                offset: params.offset,
                markets: market_refs.as_deref(),
                event_ids: params.event_id.as_deref(),
                activity_types: parsed_activity_types.as_deref(),
                start: params.start,
                end: params.end,
                sort_by: parsed_sort_by,
                sort_direction: parsed_sort_direction,
                side: parsed_side,
            },
        )
        .await?;
    write_json_output(&activity)?;
    Ok(())
}

async fn handle_get_trades(client: &Client, params: &GetTradesArgs) -> anyhow::Result<()> {
    let market_refs: Option<Vec<&str>> = params
        .market
        .as_ref()
        .map(|m| m.iter().map(|s| s.as_str()).collect());

    let parsed_filter_type = params
        .filter_type
        .as_ref()
        .map(|s| s.parse::<polymarket_hft::client::polymarket::data::TradeFilterType>())
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid --filter-type: {}", e))?;

    let parsed_side = params
        .side
        .as_ref()
        .map(|s| s.parse::<polymarket_hft::client::polymarket::data::TradeSide>())
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid --side: {}", e))?;

    let trades = client
        .get_trades(polymarket_hft::client::polymarket::data::GetTradesRequest {
            limit: params.limit,
            offset: params.offset,
            taker_only: params.taker_only,
            filter_type: parsed_filter_type,
            filter_amount: params.filter_amount,
            markets: market_refs.as_deref(),
            event_ids: params.event_id.as_deref(),
            user: params.user.as_deref(),
            side: parsed_side,
        })
        .await?;
    write_json_output(&trades)?;
    Ok(())
}
