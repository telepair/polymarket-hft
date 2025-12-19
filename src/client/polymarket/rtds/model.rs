//! Core types for RTDS WebSocket communication.

use serde::{Deserialize, Serialize};

/// API key credentials for CLOB authentication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClobAuth {
    /// API key used for authentication.
    pub key: String,

    /// API secret associated with the key.
    pub secret: String,

    /// Passphrase required for authentication.
    pub passphrase: String,
}

impl ClobAuth {
    /// Creates new CLOB authentication credentials.
    pub fn new(
        key: impl Into<String>,
        secret: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            secret: secret.into(),
            passphrase: passphrase.into(),
        }
    }
}

/// Authentication details for Gamma authentication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GammaAuth {
    /// Wallet address used for authentication.
    pub address: String,
}

impl GammaAuth {
    /// Creates new Gamma authentication.
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
        }
    }
}

/// WebSocket action types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// Subscribe to topics.
    Subscribe,
    /// Unsubscribe from topics.
    Unsubscribe,
}

/// A single subscription definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subscription {
    /// Topic to subscribe to.
    pub topic: String,

    /// Type of message within the topic (use "*" for all types).
    #[serde(rename = "type")]
    pub message_type: String,

    /// Optional filter string in JSON format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<String>,

    /// Optional CLOB authentication credentials.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clob_auth: Option<ClobAuth>,

    /// Optional Gamma authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gamma_auth: Option<GammaAuth>,
}

impl Subscription {
    /// Creates a new subscription.
    pub fn new(topic: impl Into<String>, message_type: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            message_type: message_type.into(),
            filters: None,
            clob_auth: None,
            gamma_auth: None,
        }
    }

    /// Creates a subscription for all message types in a topic.
    pub fn all(topic: impl Into<String>) -> Self {
        Self::new(topic, "*")
    }

    /// Sets the filter string.
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filters = Some(filter.into());
        self
    }

    /// Sets the CLOB authentication.
    pub fn with_clob_auth(mut self, auth: ClobAuth) -> Self {
        self.clob_auth = Some(auth);
        self
    }

    /// Sets the Gamma authentication.
    pub fn with_gamma_auth(mut self, auth: GammaAuth) -> Self {
        self.gamma_auth = Some(auth);
        self
    }
}

/// Subscription request message sent to the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    /// Action to perform.
    pub action: Action,

    /// List of subscriptions.
    pub subscriptions: Vec<Subscription>,
}

impl SubscriptionRequest {
    /// Creates a subscribe request.
    pub fn subscribe(subscriptions: Vec<Subscription>) -> Self {
        Self {
            action: Action::Subscribe,
            subscriptions,
        }
    }

    /// Creates an unsubscribe request.
    pub fn unsubscribe(subscriptions: Vec<Subscription>) -> Self {
        Self {
            action: Action::Unsubscribe,
            subscriptions,
        }
    }
}

/// Represents WebSocket connection status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Attempting to connect.
    Connecting,
    /// Successfully connected.
    Connected,
    /// Disconnected from server.
    Disconnected,
}

/// A real-time message received from the WebSocket server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Topic of the message.
    pub topic: String,

    /// Type of the message.
    #[serde(rename = "type")]
    pub message_type: String,

    /// Timestamp of when the message was sent (Unix ms).
    pub timestamp: u64,

    /// Payload containing the message data.
    pub payload: serde_json::Value,

    /// Connection ID.
    #[serde(default)]
    pub connection_id: String,
}

impl Message {
    /// Attempts to deserialize the payload into a specific type.
    pub fn parse_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }
}

/// Typed message payload for deserialization.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MessagePayload {
    /// Single object payload.
    Object(serde_json::Value),
    /// Array payload.
    Array(Vec<serde_json::Value>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_serialization() {
        let sub = Subscription::new("activity", "trades").with_filter(r#"{"event_slug":"test"}"#);

        let json = serde_json::to_string(&sub).unwrap();
        assert!(json.contains(r#""topic":"activity""#));
        assert!(json.contains(r#""type":"trades""#));
        assert!(json.contains(r#""filters":"{\"event_slug\":\"test\"}""#));
    }

    #[test]
    fn test_subscription_request_serialization() {
        let req =
            SubscriptionRequest::subscribe(vec![Subscription::new("crypto_prices", "update")]);

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""action":"subscribe""#));
    }

    #[test]
    fn test_clob_auth_serialization() {
        let sub = Subscription::new("clob_user", "*")
            .with_clob_auth(ClobAuth::new("key", "secret", "pass"));

        let json = serde_json::to_string(&sub).unwrap();
        assert!(json.contains(r#""clob_auth""#));
        assert!(json.contains(r#""key":"key""#));
    }

    #[test]
    fn test_message_deserialization() {
        let json = r#"{
            "topic": "crypto_prices",
            "type": "update",
            "timestamp": 1234567890,
            "payload": {"symbol": "BTCUSDT", "value": "50000"},
            "connection_id": "conn-123"
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.topic, "crypto_prices");
        assert_eq!(msg.message_type, "update");
        assert_eq!(msg.timestamp, 1234567890);
        assert_eq!(msg.connection_id, "conn-123");
    }

    #[test]
    fn test_message_parse_payload() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TestPayload {
            symbol: String,
            value: String,
        }

        let json = r#"{
            "topic": "test",
            "type": "test",
            "timestamp": 0,
            "payload": {"symbol": "BTC", "value": "100"}
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        let payload: TestPayload = msg.parse_payload().unwrap();
        assert_eq!(payload.symbol, "BTC");
        assert_eq!(payload.value, "100");
    }
}
