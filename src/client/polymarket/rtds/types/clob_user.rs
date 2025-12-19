//! CLOB user message types (Order, Trade).

use serde::{Deserialize, Serialize};

/// Order type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Good Till Cancelled.
    Gtc,
    /// Good Till Date.
    Gtd,
    /// Fill or Kill.
    Fok,
    /// Fill and Kill.
    Fak,
}

/// Order status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    /// Order matched.
    Matched,
    /// Order placement.
    Placement,
    /// Order cancellation.
    Cancellation,
    /// Order fill.
    Fill,
}

/// CLOB outcome side.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Outcome {
    /// Yes outcome.
    Yes,
    /// No outcome.
    No,
}

/// Trade side.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    /// Buy side.
    Buy,
    /// Sell side.
    Sell,
}

/// CLOB Order payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobOrder {
    /// Asset ID (ERC1155).
    #[serde(default)]
    pub asset_id: String,

    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,

    /// Expiration timestamp.
    #[serde(default)]
    pub expiration: String,

    /// Order ID.
    #[serde(default)]
    pub id: String,

    /// Maker address.
    #[serde(default)]
    pub maker_address: String,

    /// Market ID.
    #[serde(default)]
    pub market: String,

    /// Order type (GTC, GTD, FOK, FAK).
    #[serde(default)]
    pub order_type: Option<OrderType>,

    /// Original order size.
    #[serde(default)]
    pub original_size: String,

    /// Outcome (Yes/No).
    #[serde(default)]
    pub outcome: Option<Outcome>,

    /// Owner address.
    #[serde(default)]
    pub owner: String,

    /// Price (e.g., "0.5").
    #[serde(default)]
    pub price: String,

    /// Order side (Buy/Sell).
    #[serde(default)]
    pub side: Option<Side>,

    /// Size matched so far.
    #[serde(default)]
    pub size_matched: String,

    /// Order status.
    #[serde(default)]
    pub status: Option<OrderStatus>,

    /// Event type (PLACEMENT, CANCELLATION, FILL).
    #[serde(rename = "type", default)]
    pub event_type: String,
}

/// CLOB User Trade payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobUserTrade {
    /// Asset ID (ERC1155).
    #[serde(default)]
    pub asset_id: String,

    /// Fee rate in basis points.
    #[serde(default)]
    pub fee_rate_bps: String,

    /// Trade ID.
    #[serde(default)]
    pub id: String,

    /// Last update timestamp.
    #[serde(default)]
    pub last_update: String,

    /// Maker address.
    #[serde(default)]
    pub maker_address: String,

    /// Maker orders in this trade.
    #[serde(default)]
    pub maker_orders: Vec<MakerOrder>,

    /// Market ID.
    #[serde(default)]
    pub market: String,

    /// Match timestamp.
    #[serde(default)]
    pub match_time: String,

    /// Outcome (Yes/No).
    #[serde(default)]
    pub outcome: Option<Outcome>,

    /// Owner address.
    #[serde(default)]
    pub owner: String,

    /// Trade price.
    #[serde(default)]
    pub price: String,

    /// Trade side.
    #[serde(default)]
    pub side: Option<Side>,

    /// Trade size.
    #[serde(default)]
    pub size: String,

    /// Trade status (e.g., "MINED").
    #[serde(default)]
    pub status: String,

    /// Taker order ID.
    #[serde(default)]
    pub taker_order_id: String,

    /// Transaction hash.
    #[serde(default)]
    pub transaction_hash: String,
}

/// Maker order in a trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakerOrder {
    /// Asset ID (ERC1155).
    #[serde(default)]
    pub asset_id: String,

    /// Fee rate in basis points.
    #[serde(default)]
    pub fee_rate_bps: String,

    /// Maker address.
    #[serde(default)]
    pub maker_address: String,

    /// Matched amount.
    #[serde(default)]
    pub matched_amount: String,

    /// Order ID.
    #[serde(default)]
    pub order_id: String,

    /// Outcome (Yes/No).
    #[serde(default)]
    pub outcome: Option<Outcome>,

    /// Owner address.
    #[serde(default)]
    pub owner: String,

    /// Price.
    #[serde(default)]
    pub price: String,

    /// Side.
    #[serde(default)]
    pub side: Option<Side>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clob_order_deserialize() {
        let json = r#"{
            "asset_id": "0x123",
            "id": "order-123",
            "order_type": "GTC",
            "outcome": "YES",
            "price": "0.5",
            "side": "BUY",
            "status": "MATCHED"
        }"#;

        let order: ClobOrder = serde_json::from_str(json).unwrap();
        assert_eq!(order.id, "order-123");
        assert_eq!(order.order_type, Some(OrderType::Gtc));
        assert_eq!(order.outcome, Some(Outcome::Yes));
        assert_eq!(order.side, Some(Side::Buy));
    }

    #[test]
    fn test_clob_user_trade_deserialize() {
        let json = r#"{
            "asset_id": "0x123",
            "id": "trade-456",
            "price": "0.6",
            "side": "SELL",
            "status": "MINED"
        }"#;

        let trade: ClobUserTrade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.id, "trade-456");
        assert_eq!(trade.side, Some(Side::Sell));
        assert_eq!(trade.status, "MINED");
    }
}
