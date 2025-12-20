//! Alternative.me API client.
//!
//! This module provides a client for interacting with the Alternative.me free Crypto API.
//! No API key is required.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::client::alternativeme::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!
//!     // Fetch ticker data
//!     let ticker = client.list_ticker(Some(10), None, None).await?;
//!     println!("found {} cryptocurrencies", ticker.data.len());
//!
//!     // Get Fear and Greed Index
//!     let fng = client.get_fear_and_greed(None).await?;
//!     println!("Fear & Greed: {} ({})", fng.data[0].value, fng.data[0].value_classification);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod metrics;
pub mod model;

pub use client::Client;
pub use model::*;
