//! API clients for various data sources.
//!
//! # Structure
//!
//! - [`polymarket`]: Polymarket API clients (Data, CLOB, Gamma, RTDS)
//! - [`coinmarketcap`]: CoinMarketCap API client (requires API key)
//! - [`coingecko`]: CoinGecko API client (requires API key)
//! - [`alternativeme`]: Alternative.me free Crypto API client
//! - [`http`]: Shared HTTP client with retry middleware

pub mod alternativeme;
pub mod coingecko;
pub mod coinmarketcap;
pub mod http;
pub mod polymarket;
