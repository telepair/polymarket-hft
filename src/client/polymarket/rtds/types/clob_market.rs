//! CLOB market message types.

use serde::{Deserialize, Serialize};

use super::clob_user::Side;

/// Price changes payload (wrapper).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChanges {
    /// Market ID.
    #[serde(rename = "m", default)]
    pub market: String,

    /// Price changes array.
    #[serde(rename = "pc", default)]
    pub price_changes: Vec<PriceChange>,

    /// Timestamp.
    #[serde(rename = "t", default)]
    pub timestamp: u64,
}

/// Single price change entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChange {
    /// Asset ID.
    #[serde(rename = "a", default)]
    pub asset_id: String,

    /// Hash.
    #[serde(rename = "h", default)]
    pub hash: String,

    /// Price (e.g., "0.5").
    #[serde(rename = "p", default)]
    pub price: String,

    /// Side (BUY/SELL).
    #[serde(rename = "s", default)]
    pub side: Option<Side>,

    /// Spread index (0-100).
    #[serde(rename = "si", default)]
    pub spread_index: u32,

    /// Best ask.
    #[serde(rename = "ba", default)]
    pub best_ask: String,

    /// Best bid.
    #[serde(rename = "bb", default)]
    pub best_bid: String,
}

/// Order book level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    /// Price at this level.
    pub price: String,

    /// Total size at this level.
    pub size: String,
}

/// Aggregated orderbook payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggOrderbook {
    /// Ask levels.
    #[serde(default)]
    pub asks: Vec<OrderBookLevel>,

    /// Bid levels.
    #[serde(default)]
    pub bids: Vec<OrderBookLevel>,

    /// Asset ID.
    #[serde(default)]
    pub asset_id: String,

    /// Hash.
    #[serde(default)]
    pub hash: String,

    /// Market ID.
    #[serde(default)]
    pub market: String,

    /// Minimum order size.
    #[serde(default)]
    pub min_order_size: String,

    /// Negative risk flag.
    #[serde(default)]
    pub neg_risk: bool,

    /// Tick size.
    #[serde(default)]
    pub tick_size: String,

    /// Timestamp.
    #[serde(default)]
    pub timestamp: u64,
}

/// Last trade price payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTradePrice {
    /// Asset ID.
    #[serde(default)]
    pub asset_id: String,

    /// Fee rate in basis points.
    #[serde(default)]
    pub fee_rate_bps: String,

    /// Market ID.
    #[serde(default)]
    pub market: String,

    /// Price.
    #[serde(default)]
    pub price: String,

    /// Trade side.
    #[serde(default)]
    pub side: Option<Side>,

    /// Trade size.
    #[serde(default)]
    pub size: String,
}

/// Tick size change payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickSizeChange {
    /// Market ID.
    #[serde(default)]
    pub market: String,

    /// Asset ID (ERC1155).
    #[serde(default)]
    pub asset_id: String,

    /// Old tick size.
    #[serde(default)]
    pub old_tick_size: String,

    /// New tick size.
    #[serde(default)]
    pub new_tick_size: String,
}

/// CLOB Market payload (market_created, market_resolved).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobMarket {
    /// Market ID.
    #[serde(default)]
    pub market: String,

    /// Asset IDs (ERC1155).
    #[serde(default)]
    pub asset_ids: Vec<String>,

    /// Minimum order size.
    #[serde(default)]
    pub min_order_size: String,

    /// Tick size.
    #[serde(default)]
    pub tick_size: String,

    /// Negative risk flag.
    #[serde(default)]
    pub neg_risk: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_changes_deserialize() {
        let json = r#"{
            "m": "market-123",
            "t": 1234567890,
            "pc": [
                {"a": "asset-1", "p": "0.5", "s": "BUY", "si": 50, "ba": "0.51", "bb": "0.49", "h": "hash1"}
            ]
        }"#;

        let pc: PriceChanges = serde_json::from_str(json).unwrap();
        assert_eq!(pc.market, "market-123");
        assert_eq!(pc.price_changes.len(), 1);
        assert_eq!(pc.price_changes[0].price, "0.5");
    }

    #[test]
    fn test_agg_orderbook_deserialize() {
        let json = r#"{
            "asset_id": "0x123",
            "market": "market-456",
            "tick_size": "0.01",
            "asks": [{"price": "0.55", "size": "100"}],
            "bids": [{"price": "0.45", "size": "200"}]
        }"#;

        let ob: AggOrderbook = serde_json::from_str(json).unwrap();
        assert_eq!(ob.asset_id, "0x123");
        assert_eq!(ob.asks.len(), 1);
        assert_eq!(ob.bids.len(), 1);
    }

    #[test]
    fn test_last_trade_price_deserialize() {
        let json = r#"{
            "asset_id": "0x123",
            "market": "market-789",
            "price": "0.65",
            "side": "SELL",
            "size": "500"
        }"#;

        let ltp: LastTradePrice = serde_json::from_str(json).unwrap();
        assert_eq!(ltp.price, "0.65");
        assert_eq!(ltp.side, Some(Side::Sell));
    }

    #[test]
    fn test_clob_market_deserialize() {
        let json = r#"{
            "market": "0xabc",
            "asset_ids": ["0x123", "0x456"],
            "tick_size": "0.001",
            "min_order_size": "1",
            "neg_risk": false
        }"#;

        let market: ClobMarket = serde_json::from_str(json).unwrap();
        assert_eq!(market.asset_ids.len(), 2);
        assert_eq!(market.tick_size, "0.001");
    }
}
