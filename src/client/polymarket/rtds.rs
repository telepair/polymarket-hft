//! Polymarket Real-Time Data Service (RTDS) WebSocket client.
//!
//! This module provides a WebSocket client for receiving real-time data from Polymarket,
//! including market activity, comments, crypto/equity prices, and CLOB updates.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_hft::client::polymarket::rtds::{RtdsClient, Subscription};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = RtdsClient::builder()
//!         .auto_reconnect(true)
//!         .build();
//!
//!     client.connect().await?;
//!
//!     // Subscribe to crypto prices
//!     client.subscribe(vec![
//!         Subscription::new("crypto_prices", "update")
//!             .with_filter(r#"{"symbol":"BTCUSDT"}"#)
//!     ]).await?;
//!
//!     // Process messages
//!     while let Some(message) = client.next_message().await {
//!         println!("Received: {:?}", message);
//!     }
//!
//!     Ok(())
//! }
//! ```

mod client;
mod model;
pub mod types;

pub use client::{DEFAULT_HOST, DEFAULT_PING_INTERVAL, RtdsClient, RtdsClientBuilder};
pub use model::{
    Action, ClobAuth, ConnectionStatus, GammaAuth, Message, MessagePayload, Subscription,
    SubscriptionRequest,
};
