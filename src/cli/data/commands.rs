//! Data API CLI command definitions.

use clap::{Args, Subcommand};

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
