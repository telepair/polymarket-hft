//! RFQ (Request for Quote) message types.

use serde::{Deserialize, Serialize};

/// RFQ side: BUY or SELL.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RfqSide {
    /// Buy side.
    Buy,
    /// Sell side.
    Sell,
}

/// Request state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RequestState {
    /// Active request.
    Active,
    /// Canceled request.
    Canceled,
    /// Expired request.
    Expired,
    /// Filled request.
    Filled,
}

/// Quote state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum QuoteState {
    /// Active quote.
    Active,
    /// Canceled quote.
    Canceled,
    /// Expired quote.
    Expired,
    /// Filled quote.
    Filled,
}

/// RFQ Request payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    /// Request ID.
    #[serde(default)]
    pub request_id: String,

    /// Proxy wallet address.
    #[serde(default)]
    pub proxy_address: String,

    /// Market identifier.
    #[serde(default)]
    pub market: String,

    /// Token (ERC1155).
    #[serde(default)]
    pub token: String,

    /// Complement token (ERC1155).
    #[serde(default)]
    pub complement: String,

    /// Request state.
    #[serde(default)]
    pub state: Option<RequestState>,

    /// Request side.
    #[serde(default)]
    pub side: Option<RfqSide>,

    /// Size in.
    #[serde(default)]
    pub size_in: String,

    /// Size out.
    #[serde(default)]
    pub size_out: String,

    /// Price.
    #[serde(default)]
    pub price: String,

    /// Expiry timestamp.
    #[serde(default)]
    pub expiry: u64,
}

/// RFQ Quote payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    /// Quote ID.
    #[serde(default)]
    pub quote_id: String,

    /// Request ID this quote responds to.
    #[serde(default)]
    pub request_id: String,

    /// Proxy wallet address.
    #[serde(default)]
    pub proxy_address: String,

    /// Token (ERC1155).
    #[serde(default)]
    pub token: String,

    /// Quote state.
    #[serde(default)]
    pub state: Option<QuoteState>,

    /// Quote side.
    #[serde(default)]
    pub side: Option<RfqSide>,

    /// Size in.
    #[serde(default)]
    pub size_in: String,

    /// Size out.
    #[serde(default)]
    pub size_out: String,

    /// Condition.
    #[serde(default)]
    pub condition: String,

    /// Complement token (ERC1155).
    #[serde(default)]
    pub complement: String,

    /// Expiry timestamp.
    #[serde(default)]
    pub expiry: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_deserialize() {
        let json = r#"{
            "requestId": "req-123",
            "market": "0xabc",
            "side": "BUY",
            "state": "ACTIVE",
            "price": "0.5"
        }"#;

        let req: Request = serde_json::from_str(json).unwrap();
        assert_eq!(req.request_id, "req-123");
        assert_eq!(req.side, Some(RfqSide::Buy));
        assert_eq!(req.state, Some(RequestState::Active));
    }

    #[test]
    fn test_quote_deserialize() {
        let json = r#"{
            "quoteId": "quote-456",
            "requestId": "req-123",
            "state": "FILLED"
        }"#;

        let quote: Quote = serde_json::from_str(json).unwrap();
        assert_eq!(quote.quote_id, "quote-456");
        assert_eq!(quote.state, Some(QuoteState::Filled));
    }
}
