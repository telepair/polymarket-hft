//! Polymarket Gamma Markets API client.
//!
//! This module provides a client for interacting with the Polymarket Gamma API.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::client::polymarket::gamma::{Client, GetMarketsRequest};
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
mod comments;
mod events;
pub(crate) mod helpers;
mod markets;
mod search;
mod series;
mod sports;
mod tags;

pub use client::{Client, DEFAULT_BASE_URL};
pub use comments::{Comment, CommentProfile, GetCommentsByUserAddressRequest, GetCommentsRequest};
pub use events::{Category, Collection, Event, EventChat, EventSummary, GetEventsRequest};
pub use markets::{GetMarketsRequest, Market};
pub use search::{SearchRequest, SearchResults};
pub use series::{GetSeriesRequest, Series, SeriesSummary};
pub use sports::{GetTeamsRequest, SportMetadata, Team};
pub use tags::{GetTagsRequest, Tag, TagRelationship, TagRelationshipStatus};
