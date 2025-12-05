//! Error types for the Polymarket SDK.
//!
//! This module defines all error types that can occur when using the SDK.

/// The main error type for the Polymarket SDK.
#[derive(Debug, thiserror::Error)]
pub enum PolymarketError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// WebSocket connection or communication error.
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// API returned an error response.
    #[error("API error: {0}")]
    Api(String),

    /// URL parsing error.
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    /// Bad request - invalid parameters or input.
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Generic error with custom message.
    #[error("{0}")]
    Other(String),
}

/// A specialized Result type for Polymarket SDK operations.
pub type Result<T> = std::result::Result<T, PolymarketError>;

impl PolymarketError {
    /// Creates a new WebSocket error.
    pub fn websocket<S: Into<String>>(msg: S) -> Self {
        Self::WebSocket(msg.into())
    }

    /// Creates a new API error.
    pub fn api<S: Into<String>>(msg: S) -> Self {
        Self::Api(msg.into())
    }

    /// Creates a new bad request error.
    pub fn bad_request<S: Into<String>>(msg: S) -> Self {
        Self::BadRequest(msg.into())
    }

    /// Creates a generic error.
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }
}
