//! Alternative.me API client.
//!
//! This module provides a client for interacting with the Alternative.me free Crypto API.
//! No API key is required.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::client::alternativeme::{Client, GetTickerRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!
//!     // Fetch ticker data
//!     let ticker = client
//!         .get_ticker(GetTickerRequest { limit: Some(10), ..Default::default() })
//!         .await?;
//!     println!("found {} cryptocurrencies", ticker.data.len());
//!
//!     // Get Fear and Greed Index
//!     let fng = client.get_fear_and_greed(Default::default()).await?;
//!     println!("Fear & Greed: {} ({})", fng.data[0].value, fng.data[0].value_classification);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod model;

pub use client::Client;
pub use model::*;
