//! API clients for various data sources.
//!
//! # Structure
//!
//! - [`polymarket`]: Polymarket API clients (Data, CLOB, Gamma, RTDS)
//! - [`coinmarketcap`]: CoinMarketCap API client (requires API key)
//! - [`coingecko`]: CoinGecko API client (requires API key)
//! - [`alternativeme`]: Alternative.me free Crypto API client
//! - [`http`]: Shared HTTP client with retry middleware

use std::future::Future;
use std::pin::Pin;

use crate::Metric;

pub mod alternativeme;
pub mod coingecko;
pub mod coinmarketcap;
pub mod http;
pub mod polymarket;

// =============================================================================
// Common Types for Data Source Clients
// =============================================================================

/// Parameter metadata for API methods.
#[derive(Debug, Clone)]
pub struct MethodParam {
    /// Parameter name.
    pub name: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Whether this parameter is required.
    pub required: bool,
}

/// Describes a single metric (name and description).
#[derive(Debug, Clone)]
pub struct MetricInfo {
    /// Unique identifier for the metric.
    pub name: &'static str,
    /// Human-readable description of the metric.
    pub description: &'static str,
}

/// Metadata for an API method and its associated metrics.
#[derive(Debug, Clone)]
pub struct MethodMetadata {
    /// The API method name.
    pub method: &'static str,
    /// Human-readable description of the method.
    pub description: &'static str,
    /// Parameters accepted by this method.
    pub params: Vec<MethodParam>,
    /// List of metrics provided by this method.
    pub metrics: Vec<MetricInfo>,
}

// =============================================================================
// DataSourceClient Trait
// =============================================================================

/// Boxed future type for async trait methods.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait for data source clients that support dynamic method invocation.
///
/// This trait enables the Ingestor to fetch metrics from any data source
/// using a unified interface, regardless of the underlying API differences.
pub trait DataSourceClient: Send + Sync {
    /// Returns metadata for all supported methods.
    fn supported_methods(&self) -> Vec<MethodMetadata>;

    /// Fetch metrics by method name with optional parameters.
    ///
    /// # Arguments
    ///
    /// * `method` - The method name (e.g., "get_fear_and_greed")
    /// * `params` - Optional JSON object containing method parameters
    ///
    /// # Returns
    ///
    /// A vector of metrics collected from the data source.
    fn fetch<'a>(
        &'a self,
        method: &'a str,
        params: Option<serde_json::Value>,
    ) -> BoxFuture<'a, anyhow::Result<Vec<Metric>>>;
}
