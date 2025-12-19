//! Exchange order builder for creating and signing orders.

use alloy_primitives::{Address, U256};
use alloy_signer_local::PrivateKeySigner;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

use super::constants::get_exchange_address;
use super::eip712::sign_order;
use super::types::{Order, OrderData, SignatureType, SignedOrder};
use crate::error::{PolymarketError, Result};

// =============================================================================
// Salt Generation
// =============================================================================

/// Generates a random salt for order uniqueness.
///
/// Matches the Go implementation: random(0..2^32)
pub fn generate_salt() -> U256 {
    let mut rng = rand::thread_rng();
    let salt: u32 = rng.r#gen();
    U256::from(salt)
}

/// Generates salt matching TypeScript implementation: Math.round(Math.random() * Date.now())
#[allow(dead_code)]
pub fn generate_salt_ts_style() -> U256 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let mut rng = rand::thread_rng();
    let random: f64 = rng.r#gen();

    let salt = (random * now as f64).round() as u128;
    U256::from(salt)
}

// =============================================================================
// ExchangeOrderBuilder
// =============================================================================

/// Builder for creating and signing Polymarket exchange orders.
#[derive(Debug, Clone)]
pub struct ExchangeOrderBuilder {
    /// Private key signer.
    signer: PrivateKeySigner,
    /// Chain ID (137 for Polygon, 80002 for Amoy).
    chain_id: u64,
    /// Signature type.
    signature_type: SignatureType,
    /// Optional funder address for smart contract wallets.
    funder_address: Option<Address>,
}

impl ExchangeOrderBuilder {
    /// Creates a new ExchangeOrderBuilder.
    ///
    /// # Arguments
    ///
    /// * `signer` - Private key signer for signing orders.
    /// * `chain_id` - Chain ID (137 for Polygon, 80002 for Amoy).
    /// * `signature_type` - Signature type (EOA, PolyProxy, or PolyGnosisSafe).
    /// * `funder_address` - Optional funder address for smart contract wallets.
    pub fn new(
        signer: PrivateKeySigner,
        chain_id: u64,
        signature_type: Option<SignatureType>,
        funder_address: Option<Address>,
    ) -> Self {
        Self {
            signer,
            chain_id,
            signature_type: signature_type.unwrap_or(SignatureType::Eoa),
            funder_address,
        }
    }

    /// Gets the signer address.
    pub fn signer_address(&self) -> Address {
        self.signer.address()
    }

    /// Gets the maker address (funder if set, otherwise signer).
    pub fn maker_address(&self) -> Address {
        self.funder_address.unwrap_or_else(|| self.signer.address())
    }

    /// Builds and signs an order.
    ///
    /// # Arguments
    ///
    /// * `order_data` - Input order data.
    /// * `neg_risk` - Whether this is a negative risk market.
    ///
    /// # Returns
    ///
    /// Returns a signed order ready for submission.
    pub async fn build_signed_order(
        &self,
        order_data: OrderData,
        neg_risk: bool,
    ) -> Result<SignedOrder> {
        // Build the order
        let order = self.build_order(order_data)?;

        // Get the contract address
        let contract = get_exchange_address(self.chain_id, neg_risk)?;

        // Sign the order
        let signature = sign_order(&self.signer, &order, self.chain_id, contract).await?;

        Ok(SignedOrder::new(order, signature))
    }

    /// Builds an Order from OrderData (without signing).
    pub fn build_order(&self, order_data: OrderData) -> Result<Order> {
        // Determine signer address
        let signer_address = order_data.signer.unwrap_or(order_data.maker);

        // Verify the signer matches our wallet
        if signer_address != self.signer.address() && order_data.signer.is_some() {
            return Err(PolymarketError::other(format!(
                "Signer mismatch: expected {:?}, got {:?}",
                self.signer.address(),
                signer_address
            )));
        }

        // Use provided or generate salt
        let salt = generate_salt();

        // Set defaults
        let expiration = order_data.expiration.unwrap_or(U256::ZERO);
        let signature_type = order_data.signature_type.unwrap_or(self.signature_type);

        Ok(Order {
            salt,
            maker: order_data.maker,
            signer: signer_address,
            taker: order_data.taker,
            tokenId: order_data.token_id,
            makerAmount: order_data.maker_amount,
            takerAmount: order_data.taker_amount,
            expiration,
            nonce: order_data.nonce,
            feeRateBps: order_data.fee_rate_bps,
            side: order_data.side as u8,
            signatureType: signature_type as u8,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::Side;
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        // Salt should be non-zero (highly likely)
        // Note: There's a tiny chance this could fail, but extremely unlikely
        assert!(salt1 != U256::ZERO || salt2 != U256::ZERO);

        // Salts should be different (highly likely)
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_exchange_order_builder_new() {
        let signer = PrivateKeySigner::random();
        let builder = ExchangeOrderBuilder::new(signer.clone(), 137, None, None);

        assert_eq!(builder.chain_id, 137);
        assert_eq!(builder.signature_type, SignatureType::Eoa);
        assert!(builder.funder_address.is_none());
        assert_eq!(builder.signer_address(), signer.address());
    }

    #[test]
    fn test_exchange_order_builder_with_funder() {
        let signer = PrivateKeySigner::random();
        let funder = Address::repeat_byte(1);
        let builder = ExchangeOrderBuilder::new(
            signer.clone(),
            137,
            Some(SignatureType::PolyProxy),
            Some(funder),
        );

        assert_eq!(builder.signature_type, SignatureType::PolyProxy);
        assert_eq!(builder.funder_address, Some(funder));
        assert_eq!(builder.maker_address(), funder);
    }

    #[test]
    fn test_build_order() {
        let signer = PrivateKeySigner::random();
        let builder = ExchangeOrderBuilder::new(signer.clone(), 137, None, None);

        let order_data = OrderData {
            maker: signer.address(),
            taker: Address::ZERO,
            token_id: U256::from(12345),
            maker_amount: U256::from(1_000_000),
            taker_amount: U256::from(500_000),
            side: Side::Buy,
            fee_rate_bps: U256::from(100),
            nonce: U256::ZERO,
            signer: None,
            expiration: None,
            signature_type: None,
        };

        let order = builder.build_order(order_data).unwrap();

        assert_eq!(order.maker, signer.address());
        assert_eq!(order.signer, signer.address());
        assert_eq!(order.taker, Address::ZERO);
        assert_eq!(order.tokenId, U256::from(12345));
        assert_eq!(order.makerAmount, U256::from(1_000_000));
        assert_eq!(order.takerAmount, U256::from(500_000));
        assert_eq!(order.side, 0);
        assert_eq!(order.signatureType, 0);
    }

    #[tokio::test]
    async fn test_build_signed_order() {
        let signer = PrivateKeySigner::random();
        let builder = ExchangeOrderBuilder::new(signer.clone(), 137, None, None);

        let order_data = OrderData {
            maker: signer.address(),
            taker: Address::ZERO,
            token_id: U256::from(12345),
            maker_amount: U256::from(1_000_000),
            taker_amount: U256::from(500_000),
            side: Side::Buy,
            fee_rate_bps: U256::from(100),
            nonce: U256::ZERO,
            signer: None,
            expiration: None,
            signature_type: None,
        };

        let signed_order = builder.build_signed_order(order_data, false).await.unwrap();

        assert!(!signed_order.signature.is_empty());
        assert!(signed_order.signature.starts_with("0x"));
    }
}
