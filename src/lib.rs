//! Polymarket Rust SDK
//!
//! A comprehensive Rust SDK for interacting with Polymarket APIs.
//!
//! # Features
//!
//! - **Data API**: Access market data, user information, and more.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::data::Client;
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

pub mod data;
pub mod error;

pub use error::{PolymarketError, Result};
