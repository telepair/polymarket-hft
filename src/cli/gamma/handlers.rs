use polymarket_hft::client::polymarket::gamma::{
    Client, GetCommentsByUserAddressRequest, GetCommentsRequest, GetEventsRequest,
    GetMarketsRequest, GetSeriesRequest, GetTagsRequest, GetTeamsRequest, SearchRequest,
};

use crate::cli::common::write_json_output;

use super::commands::GammaCommands;

pub async fn handle(command: &GammaCommands) -> anyhow::Result<()> {
    let client = Client::new();

    match command {
        GammaCommands::GetTeams { params } => {
            let request = GetTeamsRequest::from(params);
            let teams = client.get_teams(request).await?;
            write_json_output(&teams)?;
        }
        GammaCommands::GetSports => {
            let sports = client.get_sports().await?;
            write_json_output(&sports)?;
        }
        GammaCommands::GetTags { params } => {
            let request = GetTagsRequest::from(params);
            let tags = client.get_tags(request).await?;
            write_json_output(&tags)?;
        }
        GammaCommands::GetTagById { id } => {
            let tag = client.get_tag_by_id(id.as_str()).await?;
            write_json_output(&tag)?;
        }
        GammaCommands::GetTagBySlug {
            slug,
            include_template,
        } => {
            let tag = client
                .get_tag_by_slug(slug.as_str(), *include_template)
                .await?;
            write_json_output(&tag)?;
        }
        GammaCommands::GetTagRelationshipsByTag {
            id,
            omit_empty,
            status,
        } => {
            let tags = client
                .get_tag_relationships_by_tag(id.as_str(), *omit_empty, *status)
                .await?;
            write_json_output(&tags)?;
        }
        GammaCommands::GetTagRelationshipsBySlug {
            slug,
            omit_empty,
            status,
        } => {
            let tags = client
                .get_tag_relationships_by_slug(slug.as_str(), *omit_empty, *status)
                .await?;
            write_json_output(&tags)?;
        }
        GammaCommands::GetTagsRelatedToTag {
            id,
            omit_empty,
            status,
        } => {
            let tags = client
                .get_tags_related_to_tag(id.as_str(), *omit_empty, *status)
                .await?;
            write_json_output(&tags)?;
        }
        GammaCommands::GetTagsRelatedToSlug {
            slug,
            omit_empty,
            status,
        } => {
            let tags = client
                .get_tags_related_to_slug(slug.as_str(), *omit_empty, *status)
                .await?;
            write_json_output(&tags)?;
        }
        GammaCommands::GetEvents { params } => {
            let request = GetEventsRequest::from(params);
            let events = client.get_events(request).await?;
            write_json_output(&events)?;
        }
        GammaCommands::GetEventById {
            id,
            include_chat,
            include_template,
        } => {
            let event = client
                .get_event_by_id(id.as_str(), *include_chat, *include_template)
                .await?;
            write_json_output(&event)?;
        }
        GammaCommands::GetEventBySlug {
            slug,
            include_chat,
            include_template,
        } => {
            let event = client
                .get_event_by_slug(slug.as_str(), *include_chat, *include_template)
                .await?;
            write_json_output(&event)?;
        }
        GammaCommands::GetEventTags { id } => {
            let tags = client.get_event_tags(id.as_str()).await?;
            write_json_output(&tags)?;
        }
        GammaCommands::GetMarkets { params } => {
            let request = GetMarketsRequest::from(params);
            let markets = client.get_markets(request).await?;
            write_json_output(&markets)?;
        }
        GammaCommands::GetMarketById { id, include_tag } => {
            let market = client.get_market_by_id(id.as_str(), *include_tag).await?;
            write_json_output(&market)?;
        }
        GammaCommands::GetMarketBySlug { slug, include_tag } => {
            let market = client
                .get_market_by_slug(slug.as_str(), *include_tag)
                .await?;
            write_json_output(&market)?;
        }
        GammaCommands::GetMarketTags { id } => {
            let tags = client.get_market_tags(id.as_str()).await?;
            write_json_output(&tags)?;
        }
        GammaCommands::GetSeries { params } => {
            let request = GetSeriesRequest::from(params);
            let series = client.get_series(request).await?;
            write_json_output(&series)?;
        }
        GammaCommands::GetSeriesById { id, include_chat } => {
            let series = client.get_series_by_id(id.as_str(), *include_chat).await?;
            write_json_output(&series)?;
        }
        GammaCommands::GetComments { params } => {
            let request = GetCommentsRequest::from(params);
            let comments = client.get_comments(request).await?;
            write_json_output(&comments)?;
        }
        GammaCommands::GetCommentById { id, get_positions } => {
            let comment = client
                .get_comment_by_id(id.as_str(), *get_positions)
                .await?;
            write_json_output(&comment)?;
        }
        GammaCommands::GetCommentsByUserAddress {
            user_address,
            limit,
            offset,
            order,
            ascending,
        } => {
            let comments = client
                .get_comments_by_user_address(GetCommentsByUserAddressRequest {
                    user_address: user_address.as_str(),
                    limit: *limit,
                    offset: *offset,
                    order: order.as_deref(),
                    ascending: *ascending,
                })
                .await?;
            write_json_output(&comments)?;
        }
        GammaCommands::Search { params } => {
            let request = SearchRequest::from(params);
            let results = client.search(request).await?;
            write_json_output(&results)?;
        }
    }

    Ok(())
}
