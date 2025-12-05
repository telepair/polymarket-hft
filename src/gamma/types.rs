//! Gamma API types and utilities.
//!
//! This module defines request builders and response types for the Gamma API.

mod comments;
mod events;
pub(crate) mod helpers;
mod markets;
mod search;
mod series;
mod sports;
mod tags;

pub use comments::{Comment, CommentProfile, GetCommentsByUserAddressRequest, GetCommentsRequest};
pub use events::{Category, Collection, Event, EventChat, EventSummary, GetEventsRequest};
pub use markets::{GetMarketsRequest, Market};
pub use search::{SearchRequest, SearchResults};
pub use series::{GetSeriesRequest, Series, SeriesSummary};
pub use sports::{GetTeamsRequest, SportMetadata, Team};
pub use tags::{GetTagsRequest, Tag, TagRelationship, TagRelationshipStatus};
