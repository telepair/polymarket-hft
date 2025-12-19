//! CoinGecko API client module.
//!
//! Provides access to CoinGecko API for cryptocurrency market data.
//!
//! # Authentication
//!
//! The client uses the Demo API key which should be set via the `CG_API_KEY` environment variable.
//! Authentication is done via the `x-cg-demo-api-key` header.
//!
//! # Endpoints
//!
//! - `/simple/price` - Get simple prices for coins
//! - `/coins/list` - Get list of all supported coins
//! - `/coins/markets` - Get market data for coins
//! - `/search/trending` - Get trending coins, NFTs, and categories
//! - `/global` - Get global cryptocurrency statistics
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::client::coingecko::{Client, SimplePriceRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("your-api-key");
//!     
//!     let request = SimplePriceRequest {
//!         ids: "bitcoin,ethereum".to_string(),
//!         vs_currencies: "usd".to_string(),
//!         ..Default::default()
//!     };
//!     
//!     let prices = client.get_simple_price(request).await?;
//!     println!("Bitcoin price: {:?}", prices.get("bitcoin"));
//!     Ok(())
//! }
//! ```

mod client;
mod model;

pub use client::Client;
pub use model::*;
