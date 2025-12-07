//! Activity message types (trades, orders_matched).

use serde::{Deserialize, Serialize};

/// Trade side: BUY or SELL.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TradeSide {
    /// Buy side.
    Buy,
    /// Sell side.
    Sell,
}

/// Activity trade payload for "trades" and "orders_matched" messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityTrade {
    /// Asset identifier.
    #[serde(default)]
    pub asset: String,

    /// User bio.
    #[serde(default)]
    pub bio: String,

    /// Condition ID.
    #[serde(default)]
    pub condition_id: String,

    /// Event slug.
    #[serde(default)]
    pub event_slug: String,

    /// Icon URL.
    #[serde(default)]
    pub icon: String,

    /// User name.
    #[serde(default)]
    pub name: String,

    /// Outcome name (e.g., "Yes", "No").
    #[serde(default)]
    pub outcome: String,

    /// Outcome index (0 or 1).
    #[serde(default)]
    pub outcome_index: u32,

    /// Trade price.
    #[serde(default)]
    pub price: String,

    /// Profile image URL.
    #[serde(default)]
    pub profile_image: String,

    /// Proxy wallet address.
    #[serde(default)]
    pub proxy_wallet: String,

    /// User pseudonym.
    #[serde(default)]
    pub pseudonym: String,

    /// Trade side.
    #[serde(default)]
    pub side: Option<TradeSide>,

    /// Trade size.
    #[serde(default)]
    pub size: String,

    /// Market slug.
    #[serde(default)]
    pub slug: String,

    /// Timestamp.
    #[serde(default)]
    pub timestamp: u64,

    /// Market title.
    #[serde(default)]
    pub title: String,

    /// Transaction hash.
    #[serde(default)]
    pub transaction_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_trade_deserialize() {
        let json = r#"{
            "asset": "0x123",
            "eventSlug": "test-event",
            "outcome": "Yes",
            "outcomeIndex": 0,
            "price": "0.5",
            "side": "BUY",
            "size": "100"
        }"#;

        let trade: ActivityTrade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.event_slug, "test-event");
        assert_eq!(trade.outcome, "Yes");
        assert_eq!(trade.side, Some(TradeSide::Buy));
    }
}
