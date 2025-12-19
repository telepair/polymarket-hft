use clap::Subcommand;
use polymarket_hft::client::polymarket::gamma::TagRelationshipStatus;

use super::args::*;

#[allow(clippy::large_enum_variant)] // CLI command payloads are only constructed once at startup
#[derive(Subcommand)]
pub enum GammaCommands {
    /// List teams
    GetTeams {
        #[command(flatten)]
        params: GetTeamsArgs,
    },
    /// List sports metadata
    GetSports,
    /// List tags
    GetTags {
        #[command(flatten)]
        params: GetTagsArgs,
    },
    /// Get a single tag by ID
    GetTagById {
        /// Tag ID
        id: String,
    },
    /// Get a single tag by slug
    GetTagBySlug {
        /// Tag slug
        slug: String,
        /// Include template tags
        #[arg(long)]
        include_template: Option<bool>,
    },
    /// Get related tags (relationships) by tag ID
    GetTagRelationshipsByTag {
        /// Tag ID
        id: String,
        /// Exclude relationships where the related tag is missing
        #[arg(long)]
        omit_empty: Option<bool>,
        /// Relationship status filter (active, closed, all)
        #[arg(long)]
        status: Option<TagRelationshipStatus>,
    },
    /// Get related tags (relationships) by tag slug
    GetTagRelationshipsBySlug {
        /// Tag slug
        slug: String,
        /// Exclude relationships where the related tag is missing
        #[arg(long)]
        omit_empty: Option<bool>,
        /// Relationship status filter (active, closed, all)
        #[arg(long)]
        status: Option<TagRelationshipStatus>,
    },
    /// Get tags related to a tag ID
    GetTagsRelatedToTag {
        /// Tag ID
        id: String,
        /// Exclude relationships where the related tag is missing
        #[arg(long)]
        omit_empty: Option<bool>,
        /// Relationship status filter (active, closed, all)
        #[arg(long)]
        status: Option<TagRelationshipStatus>,
    },
    /// Get tags related to a tag slug
    GetTagsRelatedToSlug {
        /// Tag slug
        slug: String,
        /// Exclude relationships where the related tag is missing
        #[arg(long)]
        omit_empty: Option<bool>,
        /// Relationship status filter (active, closed, all)
        #[arg(long)]
        status: Option<TagRelationshipStatus>,
    },
    /// List events
    GetEvents {
        #[command(flatten)]
        params: GetEventsArgs,
    },
    /// Get an event by ID
    GetEventById {
        /// Event ID
        id: String,
        /// Include chat channel metadata
        #[arg(long)]
        include_chat: Option<bool>,
        /// Include template fields
        #[arg(long)]
        include_template: Option<bool>,
    },
    /// Get an event by slug
    GetEventBySlug {
        /// Event slug
        slug: String,
        /// Include chat channel metadata
        #[arg(long)]
        include_chat: Option<bool>,
        /// Include template fields
        #[arg(long)]
        include_template: Option<bool>,
    },
    /// Get tags for an event
    GetEventTags {
        /// Event ID
        id: String,
    },
    /// List markets
    GetMarkets {
        #[command(flatten)]
        params: GetMarketsArgs,
    },
    /// Get a market by ID
    GetMarketById {
        /// Market ID
        id: String,
        /// Include associated tags
        #[arg(long)]
        include_tag: Option<bool>,
    },
    /// Get a market by slug
    GetMarketBySlug {
        /// Market slug
        slug: String,
        /// Include associated tags
        #[arg(long)]
        include_tag: Option<bool>,
    },
    /// Get tags for a market
    GetMarketTags {
        /// Market ID
        id: String,
    },
    /// List series
    GetSeries {
        #[command(flatten)]
        params: GetSeriesArgs,
    },
    /// Get a series by ID
    GetSeriesById {
        /// Series ID
        id: String,
        /// Include chat channel metadata
        #[arg(long)]
        include_chat: Option<bool>,
    },
    /// List comments
    GetComments {
        #[command(flatten)]
        params: GetCommentsArgs,
    },
    /// Get a comment by ID
    GetCommentById {
        /// Comment ID
        id: String,
        /// Include position data for commenters
        #[arg(long)]
        get_positions: Option<bool>,
    },
    /// Get comments by user address
    GetCommentsByUserAddress {
        /// User address
        user_address: String,
        /// Limit results (1-1000)
        #[arg(short, long)]
        limit: Option<u32>,
        /// Offset for pagination (>= 0)
        #[arg(short, long)]
        offset: Option<u32>,
        /// Order expression
        #[arg(long)]
        order: Option<String>,
        /// Sort ascending
        #[arg(long)]
        ascending: Option<bool>,
    },
    /// Search markets, events, and profiles
    Search {
        #[command(flatten)]
        params: SearchArgs,
    },
}
