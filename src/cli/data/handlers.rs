//! Data API CLI command handlers.

use polymarket_hft::client::polymarket::data::Client;

use super::commands::{
    DataCommands, GetTradesArgs, GetUserActivityArgs, GetUserClosedPositionsArgs,
    GetUserPositionsArgs,
};
use crate::cli::common::write_json_output;

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
