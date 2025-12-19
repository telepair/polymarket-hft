//! Polymarket HFT System
//!
//! A high-frequency trading system for Polymarket with built-in API clients.
//!
//! # Features
//!
//! - **Data API**: Access market data, user information, and more.
//! - **Gamma Markets API**: Market discovery and metadata.
//! - **CLOB API**: Central Limit Order Book for pricing and order books.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::client::polymarket::data::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!     
//!     // Check API health
//!     let health = client.health().await?;
//!     println!("API status: {}", health.data);
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;

pub use error::{PolymarketError, Result};
