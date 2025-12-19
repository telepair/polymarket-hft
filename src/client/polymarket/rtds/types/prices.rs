//! Price message types (CryptoPrice, EquityPrice).

use serde::{Deserialize, Serialize};

/// Crypto price update payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPrice {
    /// Symbol (e.g., "BTCUSDT").
    pub symbol: String,

    /// Timestamp.
    pub timestamp: u64,

    /// Price value.
    pub value: String,
}

/// Equity price update payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPrice {
    /// Symbol (e.g., "AAPL").
    pub symbol: String,

    /// Timestamp.
    pub timestamp: u64,

    /// Price value.
    pub value: String,
}

/// Historical price data sent on initial connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistorical {
    /// Symbol.
    pub symbol: String,

    /// Historical data points.
    #[serde(default)]
    pub data: Vec<PricePoint>,
}

/// A single price point in historical data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    /// Timestamp.
    pub timestamp: u64,

    /// Price value.
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_price_deserialize() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "timestamp": 1234567890,
            "value": "50000.00"
        }"#;

        let price: CryptoPrice = serde_json::from_str(json).unwrap();
        assert_eq!(price.symbol, "BTCUSDT");
        assert_eq!(price.value, "50000.00");
    }

    #[test]
    fn test_equity_price_deserialize() {
        let json = r#"{
            "symbol": "AAPL",
            "timestamp": 1234567890,
            "value": "150.00"
        }"#;

        let price: EquityPrice = serde_json::from_str(json).unwrap();
        assert_eq!(price.symbol, "AAPL");
        assert_eq!(price.value, "150.00");
    }
}
