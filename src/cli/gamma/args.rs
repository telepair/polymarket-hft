use clap::Args;
use polymarket_hft::client::polymarket::gamma::{
    GetCommentsRequest, GetEventsRequest, GetMarketsRequest, GetSeriesRequest, GetTagsRequest,
    GetTeamsRequest, SearchRequest,
};

#[derive(Args, Debug, Clone)]
pub struct GetTeamsArgs {
    /// Limit results (1-1000)
    #[arg(short, long)]
    pub limit: Option<u32>,
    /// Offset for pagination (>= 0)
    #[arg(short, long)]
    pub offset: Option<u32>,
    /// Order expression (comma-separated)
    #[arg(long)]
    pub order: Option<String>,
    /// Sort ascending
    #[arg(long)]
    pub ascending: Option<bool>,
    /// Filter by league (repeatable)
    #[arg(long)]
    pub league: Option<Vec<String>>,
    /// Filter by name (repeatable)
    #[arg(long)]
    pub name: Option<Vec<String>>,
    /// Filter by abbreviation (repeatable)
    #[arg(long)]
    pub abbreviation: Option<Vec<String>>,
}

impl<'a> From<&'a GetTeamsArgs> for GetTeamsRequest<'a> {
    fn from(args: &'a GetTeamsArgs) -> Self {
        GetTeamsRequest {
            limit: args.limit,
            offset: args.offset,
            order: args.order.as_deref(),
            ascending: args.ascending,
            league: args.league.clone(),
            name: args.name.clone(),
            abbreviation: args.abbreviation.clone(),
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct GetTagsArgs {
    /// Limit results (1-1000)
    #[arg(short, long)]
    pub limit: Option<u32>,
    /// Offset for pagination (>= 0)
    #[arg(short, long)]
    pub offset: Option<u32>,
    /// Order expression (comma-separated)
    #[arg(long)]
    pub order: Option<String>,
    /// Sort ascending
    #[arg(long)]
    pub ascending: Option<bool>,
    /// Include template tags
    #[arg(long)]
    pub include_template: Option<bool>,
    /// Filter for carousel tags
    #[arg(long)]
    pub is_carousel: Option<bool>,
}

impl<'a> From<&'a GetTagsArgs> for GetTagsRequest<'a> {
    fn from(args: &'a GetTagsArgs) -> Self {
        GetTagsRequest {
            limit: args.limit,
            offset: args.offset,
            order: args.order.as_deref(),
            ascending: args.ascending,
            include_template: args.include_template,
            is_carousel: args.is_carousel,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct GetEventsArgs {
    /// Limit results
    #[arg(short, long)]
    pub limit: Option<u32>,
    /// Offset for pagination (>= 0)
    #[arg(short, long)]
    pub offset: Option<u32>,
    /// Order expression (per Gamma API)
    #[arg(long)]
    pub order: Option<String>,
    /// Sort direction ascending flag
    #[arg(long)]
    pub ascending: Option<bool>,
    /// Filter by event IDs (repeatable)
    #[arg(long)]
    pub id: Option<Vec<String>>,
    /// Filter by tag ID
    #[arg(long)]
    pub tag_id: Option<String>,
    /// Exclude tag IDs (repeatable)
    #[arg(long)]
    pub exclude_tag_id: Option<Vec<String>>,
    /// Filter by event slugs (repeatable)
    #[arg(long)]
    pub slug: Option<Vec<String>>,
    /// Filter by tag slug
    #[arg(long)]
    pub tag_slug: Option<String>,
    /// Include related tags
    #[arg(long)]
    pub related_tags: Option<bool>,
    /// Filter by active status
    #[arg(long)]
    pub active: Option<bool>,
    /// Filter by archived status
    #[arg(long)]
    pub archived: Option<bool>,
    /// Filter by featured flag
    #[arg(long)]
    pub featured: Option<bool>,
    /// Filter by create-your-own-markets flag
    #[arg(long)]
    pub cyom: Option<bool>,
    /// Include chat channel metadata
    #[arg(long)]
    pub include_chat: Option<bool>,
    /// Include template fields
    #[arg(long)]
    pub include_template: Option<bool>,
    /// Filter by recurrence
    #[arg(long)]
    pub recurrence: Option<String>,
    /// Filter by closed status
    #[arg(long)]
    pub closed: Option<bool>,
    /// Minimum liquidity
    #[arg(long)]
    pub liquidity_min: Option<f64>,
    /// Maximum liquidity
    #[arg(long)]
    pub liquidity_max: Option<f64>,
    /// Minimum volume
    #[arg(long)]
    pub volume_min: Option<f64>,
    /// Maximum volume
    #[arg(long)]
    pub volume_max: Option<f64>,
    /// Minimum start date (ISO-8601)
    #[arg(long)]
    pub start_date_min: Option<String>,
    /// Maximum start date (ISO-8601)
    #[arg(long)]
    pub start_date_max: Option<String>,
    /// Minimum end date (ISO-8601)
    #[arg(long)]
    pub end_date_min: Option<String>,
    /// Maximum end date (ISO-8601)
    #[arg(long)]
    pub end_date_max: Option<String>,
}

impl<'a> From<&'a GetEventsArgs> for GetEventsRequest<'a> {
    fn from(args: &'a GetEventsArgs) -> Self {
        GetEventsRequest {
            limit: args.limit,
            offset: args.offset,
            order: args.order.as_deref(),
            ascending: args.ascending,
            id: args.id.clone(),
            tag_id: args.tag_id.as_deref(),
            exclude_tag_id: args.exclude_tag_id.clone(),
            slug: args.slug.clone(),
            tag_slug: args.tag_slug.as_deref(),
            related_tags: args.related_tags,
            active: args.active,
            archived: args.archived,
            featured: args.featured,
            cyom: args.cyom,
            include_chat: args.include_chat,
            include_template: args.include_template,
            recurrence: args.recurrence.as_deref(),
            closed: args.closed,
            liquidity_min: args.liquidity_min,
            liquidity_max: args.liquidity_max,
            volume_min: args.volume_min,
            volume_max: args.volume_max,
            start_date_min: args.start_date_min.as_deref(),
            start_date_max: args.start_date_max.as_deref(),
            end_date_min: args.end_date_min.as_deref(),
            end_date_max: args.end_date_max.as_deref(),
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct GetMarketsArgs {
    /// Limit results (1-1000)
    #[arg(short, long)]
    pub limit: Option<u32>,
    /// Offset for pagination (>= 0)
    #[arg(short, long)]
    pub offset: Option<u32>,
    /// Order expression
    #[arg(long)]
    pub order: Option<String>,
    /// Sort ascending
    #[arg(long)]
    pub ascending: Option<bool>,
    /// Filter by market IDs (repeatable)
    #[arg(long)]
    pub id: Option<Vec<String>>,
    /// Filter by market slugs (repeatable)
    #[arg(long)]
    pub slug: Option<Vec<String>>,
    /// Filter by CLOB token IDs (repeatable)
    #[arg(long)]
    pub clob_token_ids: Option<Vec<String>>,
    /// Filter by condition IDs (repeatable)
    #[arg(long)]
    pub condition_ids: Option<Vec<String>>,
    /// Filter by market maker addresses (repeatable)
    #[arg(long)]
    pub market_maker_address: Option<Vec<String>>,
    /// Minimum liquidity
    #[arg(long)]
    pub liquidity_num_min: Option<f64>,
    /// Maximum liquidity
    #[arg(long)]
    pub liquidity_num_max: Option<f64>,
    /// Minimum volume
    #[arg(long)]
    pub volume_num_min: Option<f64>,
    /// Maximum volume
    #[arg(long)]
    pub volume_num_max: Option<f64>,
    /// Minimum start date (ISO-8601)
    #[arg(long)]
    pub start_date_min: Option<String>,
    /// Maximum start date (ISO-8601)
    #[arg(long)]
    pub start_date_max: Option<String>,
    /// Minimum end date (ISO-8601)
    #[arg(long)]
    pub end_date_min: Option<String>,
    /// Maximum end date (ISO-8601)
    #[arg(long)]
    pub end_date_max: Option<String>,
    /// Filter by tag ID
    #[arg(long)]
    pub tag_id: Option<String>,
    /// Include related tags
    #[arg(long)]
    pub related_tags: Option<bool>,
    /// Filter by create-your-own-market flag
    #[arg(long)]
    pub cyom: Option<bool>,
    /// Filter by UMA resolution status
    #[arg(long)]
    pub uma_resolution_status: Option<String>,
    /// Filter by game ID
    #[arg(long)]
    pub game_id: Option<String>,
    /// Filter by sports market types (repeatable)
    #[arg(long)]
    pub sports_market_types: Option<Vec<String>>,
    /// Minimum rewards size
    #[arg(long)]
    pub rewards_min_size: Option<f64>,
    /// Filter by question IDs (repeatable)
    #[arg(long)]
    pub question_ids: Option<Vec<String>>,
    /// Include tags in response
    #[arg(long)]
    pub include_tag: Option<bool>,
    /// Filter by closed status
    #[arg(long)]
    pub closed: Option<bool>,
}

impl<'a> From<&'a GetMarketsArgs> for GetMarketsRequest<'a> {
    fn from(args: &'a GetMarketsArgs) -> Self {
        GetMarketsRequest {
            limit: args.limit,
            offset: args.offset,
            order: args.order.as_deref(),
            ascending: args.ascending,
            id: args.id.clone(),
            slug: args.slug.clone(),
            clob_token_ids: args.clob_token_ids.clone(),
            condition_ids: args.condition_ids.clone(),
            market_maker_address: args.market_maker_address.clone(),
            liquidity_num_min: args.liquidity_num_min,
            liquidity_num_max: args.liquidity_num_max,
            volume_num_min: args.volume_num_min,
            volume_num_max: args.volume_num_max,
            start_date_min: args.start_date_min.as_deref(),
            start_date_max: args.start_date_max.as_deref(),
            end_date_min: args.end_date_min.as_deref(),
            end_date_max: args.end_date_max.as_deref(),
            tag_id: args.tag_id.as_deref(),
            related_tags: args.related_tags,
            cyom: args.cyom,
            uma_resolution_status: args.uma_resolution_status.as_deref(),
            game_id: args.game_id.as_deref(),
            sports_market_types: args.sports_market_types.clone(),
            rewards_min_size: args.rewards_min_size,
            question_ids: args.question_ids.clone(),
            include_tag: args.include_tag,
            closed: args.closed,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct GetSeriesArgs {
    /// Limit results (1-1000)
    #[arg(short, long)]
    pub limit: Option<u32>,
    /// Offset for pagination (>= 0)
    #[arg(short, long)]
    pub offset: Option<u32>,
    /// Order expression (per Gamma API)
    #[arg(long)]
    pub order: Option<String>,
    /// Sort ascending
    #[arg(long)]
    pub ascending: Option<bool>,
    /// Filter by slug
    #[arg(short, long)]
    pub slug: Option<String>,
    /// Filter by category IDs (repeatable)
    #[arg(long)]
    pub categories_ids: Option<Vec<String>>,
    /// Filter by category labels (repeatable)
    #[arg(long)]
    pub categories_labels: Option<Vec<String>>,
    /// Filter by closed status
    #[arg(long)]
    pub closed: Option<bool>,
    /// Include chat channel metadata
    #[arg(long)]
    pub include_chat: Option<bool>,
    /// Filter by recurrence
    #[arg(long)]
    pub recurrence: Option<String>,
}

impl<'a> From<&'a GetSeriesArgs> for GetSeriesRequest<'a> {
    fn from(args: &'a GetSeriesArgs) -> Self {
        GetSeriesRequest {
            limit: args.limit,
            offset: args.offset,
            order: args.order.as_deref(),
            ascending: args.ascending,
            slug: args.slug.as_deref(),
            categories_ids: args.categories_ids.clone(),
            categories_labels: args.categories_labels.clone(),
            closed: args.closed,
            include_chat: args.include_chat,
            recurrence: args.recurrence.as_deref(),
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct GetCommentsArgs {
    /// Parent entity type (Event, Series, market)
    #[arg(long, value_parser = ["Event", "Series", "market"])]
    pub parent_entity_type: String,
    /// Parent entity ID
    #[arg(long)]
    pub parent_entity_id: String,
    /// Limit results (1-1000)
    #[arg(short, long)]
    pub limit: Option<u32>,
    /// Offset for pagination (>= 0)
    #[arg(short, long)]
    pub offset: Option<u32>,
    /// Order expression
    #[arg(long)]
    pub order: Option<String>,
    /// Sort ascending
    #[arg(long)]
    pub ascending: Option<bool>,
    /// Include position data for commenters
    #[arg(long)]
    pub get_positions: Option<bool>,
    /// Restrict results to holders only
    #[arg(long)]
    pub holders_only: Option<bool>,
}

impl<'a> From<&'a GetCommentsArgs> for GetCommentsRequest<'a> {
    fn from(args: &'a GetCommentsArgs) -> Self {
        GetCommentsRequest {
            limit: args.limit,
            offset: args.offset,
            order: args.order.as_deref(),
            ascending: args.ascending,
            parent_entity_type: Some(args.parent_entity_type.as_str()),
            parent_entity_id: Some(args.parent_entity_id.as_str()),
            get_positions: args.get_positions,
            holders_only: args.holders_only,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct SearchArgs {
    /// Query string
    pub query: String,
    /// Use cached results when available
    #[arg(long)]
    pub cache: Option<bool>,
    /// Filter by event status
    #[arg(long)]
    pub events_status: Option<String>,
    /// Limit per result type (1-1000)
    #[arg(long)]
    pub limit_per_type: Option<u32>,
    /// Page number (>= 1)
    #[arg(long)]
    pub page: Option<u32>,
    /// Filter by event tags (repeatable)
    #[arg(long)]
    pub events_tag: Option<Vec<String>>,
    /// Include closed markets in results
    #[arg(long)]
    pub keep_closed_markets: Option<u32>,
    /// Sort expression
    #[arg(long)]
    pub sort: Option<String>,
    /// Sort ascending
    #[arg(long)]
    pub ascending: Option<bool>,
    /// Search tags
    #[arg(long)]
    pub search_tags: Option<bool>,
    /// Search profiles
    #[arg(long)]
    pub search_profiles: Option<bool>,
    /// Filter by recurrence
    #[arg(long)]
    pub recurrence: Option<String>,
    /// Exclude tag IDs (repeatable)
    #[arg(long)]
    pub exclude_tag_id: Option<Vec<String>>,
    /// Enable optimized search
    #[arg(long)]
    pub optimized: Option<bool>,
}

impl<'a> From<&'a SearchArgs> for SearchRequest<'a> {
    fn from(args: &'a SearchArgs) -> Self {
        SearchRequest {
            q: args.query.as_str(),
            cache: args.cache,
            events_status: args.events_status.as_deref(),
            limit_per_type: args.limit_per_type,
            page: args.page,
            events_tag: args.events_tag.clone(),
            keep_closed_markets: args.keep_closed_markets,
            sort: args.sort.as_deref(),
            ascending: args.ascending,
            search_tags: args.search_tags,
            search_profiles: args.search_profiles,
            recurrence: args.recurrence.as_deref(),
            exclude_tag_id: args.exclude_tag_id.clone(),
            optimized: args.optimized,
        }
    }
}
