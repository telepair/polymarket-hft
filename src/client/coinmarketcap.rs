//! CoinMarketCap API client.
//!
//! This module provides a client for interacting with the CoinMarketCap API.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::client::coinmarketcap::{Client, GetListingsLatestRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("YOUR_API_KEY");
//!
//!     // Fetch listings
//!     let listings = client
//!         .get_listings_latest(GetListingsLatestRequest::default())
//!         .await?;
//!     println!("found {} listings", listings.data.len());
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod model;

pub use client::Client;
pub use model::*;
