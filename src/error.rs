//! Error types for the Polymarket SDK.
//!
//! This module defines all error types that can occur when using the SDK.

/// The main error type for the Polymarket SDK.
#[derive(Debug, thiserror::Error)]
pub enum PolymarketError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// HTTP middleware request failed.
    #[error("HTTP request failed: {0}")]
    HttpMiddleware(#[from] reqwest_middleware::Error),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_error_creation() {
        let err = PolymarketError::websocket("connection failed");
        assert!(matches!(err, PolymarketError::WebSocket(_)));
        assert!(err.to_string().contains("connection failed"));
    }

    #[test]
    fn test_api_error_creation() {
        let err = PolymarketError::api("rate limited");
        assert!(matches!(err, PolymarketError::Api(_)));
        assert!(err.to_string().contains("rate limited"));
    }

    #[test]
    fn test_bad_request_error_creation() {
        let err = PolymarketError::bad_request("invalid parameter");
        assert!(matches!(err, PolymarketError::BadRequest(_)));
        assert!(err.to_string().contains("invalid parameter"));
    }

    #[test]
    fn test_other_error_creation() {
        let err = PolymarketError::other("something went wrong");
        assert!(matches!(err, PolymarketError::Other(_)));
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn test_display_websocket() {
        let err = PolymarketError::WebSocket("test".to_string());
        assert_eq!(err.to_string(), "WebSocket error: test");
    }

    #[test]
    fn test_display_api() {
        let err = PolymarketError::Api("test".to_string());
        assert_eq!(err.to_string(), "API error: test");
    }

    #[test]
    fn test_display_bad_request() {
        let err = PolymarketError::BadRequest("test".to_string());
        assert_eq!(err.to_string(), "Bad request: test");
    }

    #[test]
    fn test_from_url_parse_error() {
        let url_err = url::Url::parse("not a url").unwrap_err();
        let err: PolymarketError = url_err.into();
        assert!(matches!(err, PolymarketError::Url(_)));
        assert!(err.to_string().contains("URL parsing error"));
    }

    #[test]
    fn test_from_serde_error() {
        let json_err = serde_json::from_str::<String>("not valid json").unwrap_err();
        let err: PolymarketError = json_err.into();
        assert!(matches!(err, PolymarketError::Serde(_)));
        assert!(err.to_string().contains("Serialization error"));
    }
}
