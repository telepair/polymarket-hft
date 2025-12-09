//! Order types for the Polymarket CTF Exchange.
//!
//! Defines the EIP-712 Order struct and related types for order signing.

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};

// =============================================================================
// Side
// =============================================================================

/// Order side (BUY or SELL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Side {
    /// Buy order (0).
    #[default]
    Buy = 0,
    /// Sell order (1).
    Sell = 1,
}

impl From<Side> for u8 {
    fn from(side: Side) -> Self {
        side as u8
    }
}

impl From<Side> for U256 {
    fn from(side: Side) -> Self {
        U256::from(side as u8)
    }
}

// =============================================================================
// SignatureType
// =============================================================================

/// Signature type used by the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SignatureType {
    /// ECDSA EIP-712 signatures signed by EOAs.
    #[default]
    Eoa = 0,
    /// EIP-712 signatures signed by EOAs that own Polymarket Proxy wallets.
    PolyProxy = 1,
    /// EIP-712 signatures signed by EOAs that own Polymarket Gnosis safes.
    PolyGnosisSafe = 2,
}

impl From<SignatureType> for u8 {
    fn from(sig_type: SignatureType) -> Self {
        sig_type as u8
    }
}

impl From<SignatureType> for U256 {
    fn from(sig_type: SignatureType) -> Self {
        U256::from(sig_type as u8)
    }
}

// =============================================================================
// OrderData (Input for order creation)
// =============================================================================

/// Input data for creating an order (before signing).
#[derive(Debug, Clone)]
pub struct OrderData {
    /// Maker of the order (source of funds).
    pub maker: Address,

    /// Address of the order taker. Zero address = public order.
    pub taker: Address,

    /// Token ID of the CTF ERC1155 asset to trade.
    pub token_id: U256,

    /// Maker amount (max tokens to be sold).
    pub maker_amount: U256,

    /// Taker amount (min tokens to be received).
    pub taker_amount: U256,

    /// Order side (BUY or SELL).
    pub side: Side,

    /// Fee rate in basis points.
    pub fee_rate_bps: U256,

    /// Nonce for onchain cancellations.
    pub nonce: U256,

    /// Signer address (defaults to maker if None).
    pub signer: Option<Address>,

    /// Expiration timestamp (0 = no expiration).
    pub expiration: Option<U256>,

    /// Signature type (defaults to EOA if None).
    pub signature_type: Option<SignatureType>,
}

// =============================================================================
// EIP-712 Order Struct
// =============================================================================

sol! {
    /// EIP-712 Order struct for the Polymarket CTF Exchange.
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Order {
        /// Unique salt for entropy.
        uint256 salt;
        /// Maker address (source of funds).
        address maker;
        /// Signer address.
        address signer;
        /// Taker address (zero = public order).
        address taker;
        /// Token ID of the CTF ERC1155 asset.
        uint256 tokenId;
        /// Maker amount (max tokens to sell).
        uint256 makerAmount;
        /// Taker amount (min tokens to receive).
        uint256 takerAmount;
        /// Expiration timestamp.
        uint256 expiration;
        /// Nonce for onchain cancellations.
        uint256 nonce;
        /// Fee rate in basis points.
        uint256 feeRateBps;
        /// Order side (0=BUY, 1=SELL).
        uint8 side;
        /// Signature type.
        uint8 signatureType;
    }
}

// =============================================================================
// SignedOrder
// =============================================================================

/// Signed order with signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedOrder {
    /// The order details.
    #[serde(flatten)]
    pub order: Order,

    /// The order signature (hex string with 0x prefix).
    pub signature: String,
}

impl SignedOrder {
    /// Creates a new signed order.
    pub fn new(order: Order, signature: Vec<u8>) -> Self {
        Self {
            order,
            signature: format!("0x{}", hex::encode(signature)),
        }
    }

    /// Converts the signed order to JSON value for API submission.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "salt": self.order.salt.to_string(),
            "maker": format!("{:?}", self.order.maker),
            "signer": format!("{:?}", self.order.signer),
            "taker": format!("{:?}", self.order.taker),
            "tokenId": self.order.tokenId.to_string(),
            "makerAmount": self.order.makerAmount.to_string(),
            "takerAmount": self.order.takerAmount.to_string(),
            "expiration": self.order.expiration.to_string(),
            "nonce": self.order.nonce.to_string(),
            "feeRateBps": self.order.feeRateBps.to_string(),
            "side": self.order.side.to_string(),
            "signatureType": self.order.signatureType.to_string(),
            "signature": self.signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_values() {
        assert_eq!(u8::from(Side::Buy), 0);
        assert_eq!(u8::from(Side::Sell), 1);
    }

    #[test]
    fn test_signature_type_values() {
        assert_eq!(u8::from(SignatureType::Eoa), 0);
        assert_eq!(u8::from(SignatureType::PolyProxy), 1);
        assert_eq!(u8::from(SignatureType::PolyGnosisSafe), 2);
    }

    #[test]
    fn test_signed_order_to_json() {
        let order = Order {
            salt: U256::from(12345),
            maker: Address::ZERO,
            signer: Address::ZERO,
            taker: Address::ZERO,
            tokenId: U256::from(1),
            makerAmount: U256::from(1000000),
            takerAmount: U256::from(500000),
            expiration: U256::ZERO,
            nonce: U256::ZERO,
            feeRateBps: U256::from(100),
            side: 0,
            signatureType: 0,
        };
        let signed = SignedOrder::new(order, vec![0u8; 65]);
        let json = signed.to_json();
        assert!(json.get("salt").is_some());
        assert!(json.get("signature").is_some());
    }
}
