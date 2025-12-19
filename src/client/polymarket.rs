//! Polymarket API clients.
//!
//! This module contains all clients for interacting with Polymarket services:
//! - [`data`]: User data, positions, trades, and portfolio information
//! - [`clob`]: Central Limit Order Book (REST + WebSocket)
//! - [`gamma`]: Market discovery and metadata
//! - [`rtds`]: Real-time data streaming

pub mod clob;
pub mod data;
pub mod gamma;
pub mod rtds;
