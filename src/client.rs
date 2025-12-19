//! API clients for various data sources.
//!
//! # Structure
//!
//! - [`polymarket`]: Polymarket API clients (Data, CLOB, Gamma, RTDS)
//! - [`http`]: Shared HTTP client with retry middleware

pub mod http;
pub mod polymarket;
