//! EIP-712 signing utilities for order hashing and signing.

use alloy_primitives::Address;
use alloy_signer::Signer;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{SolStruct, eip712_domain};

use super::constants::{PROTOCOL_NAME, PROTOCOL_VERSION};
use super::types::Order;
use crate::error::{PolymarketError, Result};

// =============================================================================
// EIP-712 Domain
// =============================================================================

/// Builds the EIP-712 domain for order signing.
pub fn build_domain(chain_id: u64, verifying_contract: Address) -> alloy_sol_types::Eip712Domain {
    eip712_domain! {
        name: PROTOCOL_NAME,
        version: PROTOCOL_VERSION,
        chain_id: chain_id,
        verifying_contract: verifying_contract,
    }
}

// =============================================================================
// Order Hashing
// =============================================================================

/// Calculates the EIP-712 hash of an order.
///
/// The hash is: keccak256("\x19\x01" || domainSeparator || structHash)
pub fn hash_order(order: &Order, chain_id: u64, verifying_contract: Address) -> [u8; 32] {
    let domain = build_domain(chain_id, verifying_contract);
    order.eip712_signing_hash(&domain).into()
}

// =============================================================================
// Order Signing
// =============================================================================

/// Signs an order hash with the given signer.
///
/// Returns the signature as bytes (65 bytes: r, s, v).
pub async fn sign_order(
    signer: &PrivateKeySigner,
    order: &Order,
    chain_id: u64,
    verifying_contract: Address,
) -> Result<Vec<u8>> {
    let hash = hash_order(order, chain_id, verifying_contract);

    let signature = signer
        .sign_hash(&hash.into())
        .await
        .map_err(|e| PolymarketError::other(format!("Failed to sign order: {}", e)))?;

    // Convert signature to bytes (r || s || v)
    let mut sig_bytes = signature.as_bytes().to_vec();

    // Adjust v value: add 27 if needed (standard Ethereum signature format)
    if sig_bytes.len() == 65 && sig_bytes[64] < 27 {
        sig_bytes[64] += 27;
    }

    Ok(sig_bytes)
}

/// Validates that a signature was produced by the expected signer.
#[allow(dead_code)]
pub fn validate_signature(
    expected_signer: Address,
    order: &Order,
    signature: &[u8],
    chain_id: u64,
    verifying_contract: Address,
) -> Result<bool> {
    if signature.len() != 65 {
        return Err(PolymarketError::other("Invalid signature length"));
    }

    let hash = hash_order(order, chain_id, verifying_contract);

    // Adjust v if needed
    let mut sig_copy = signature.to_vec();
    if sig_copy[64] >= 27 {
        sig_copy[64] -= 27;
    }

    // Recover the signer address from the signature
    let signature = alloy_primitives::PrimitiveSignature::try_from(sig_copy.as_slice())
        .map_err(|e| PolymarketError::other(format!("Invalid signature: {}", e)))?;

    let recovered = signature
        .recover_address_from_prehash(&hash.into())
        .map_err(|e| PolymarketError::other(format!("Failed to recover address: {}", e)))?;

    Ok(recovered == expected_signer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{U256, address};

    #[test]
    fn test_build_domain() {
        let contract = address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E");
        let domain = build_domain(137, contract);

        assert_eq!(
            domain.name.as_ref().map(|s| s.as_ref()),
            Some(PROTOCOL_NAME)
        );
        assert_eq!(
            domain.version.as_ref().map(|s| s.as_ref()),
            Some(PROTOCOL_VERSION)
        );
        assert_eq!(domain.chain_id, Some(U256::from(137)));
        assert_eq!(domain.verifying_contract, Some(contract));
    }

    #[test]
    fn test_hash_order() {
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

        let contract = address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E");
        let hash = hash_order(&order, 137, contract);

        // Hash should be 32 bytes
        assert_eq!(hash.len(), 32);
        // Hash should be non-zero
        assert!(hash.iter().any(|&b| b != 0));
    }

    #[tokio::test]
    async fn test_sign_order() {
        let signer = PrivateKeySigner::random();
        let order = Order {
            salt: U256::from(12345),
            maker: signer.address(),
            signer: signer.address(),
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

        let contract = address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E");
        let signature = sign_order(&signer, &order, 137, contract).await.unwrap();

        // Signature should be 65 bytes (r, s, v)
        assert_eq!(signature.len(), 65);
    }
}
