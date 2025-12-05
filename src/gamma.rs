//! Polymarket Gamma Markets API client.
//!
//! This module provides a client for interacting with the Polymarket Gamma API.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::gamma::{Client, GetMarketsRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!
//!     // Fetch a few markets
//!     let markets = client
//!         .get_markets(GetMarketsRequest {
//!             limit: Some(5),
//!             closed: Some(false),
//!             ..Default::default()
//!         })
//!         .await?;
//!     println!("found {} markets", markets.len());
//!
//!     Ok(())
//! }
//! ```

mod client;
mod types;

pub use client::{Client, DEFAULT_BASE_URL};
pub use types::{
    Category, Collection, Comment, CommentProfile, Event, EventChat, EventSummary,
    GetCommentsByUserAddressRequest, GetCommentsRequest, GetEventsRequest, GetMarketsRequest,
    GetSeriesRequest, GetTagsRequest, GetTeamsRequest, Market, SearchRequest, SearchResults,
    Series, SeriesSummary, SportMetadata, Tag, TagRelationship, TagRelationshipStatus, Team,
};
