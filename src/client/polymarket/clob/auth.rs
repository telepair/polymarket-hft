//! Authentication module for CLOB API.
//!
//! Provides L1 (EIP-712) and L2 (HMAC-SHA256) authentication header generation.

use std::time::{SystemTime, UNIX_EPOCH};

use alloy_primitives::Address;
use alloy_signer::Signer;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{SolStruct, sol};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::types::{ApiKeyCreds, L1PolyHeader, L2PolyHeader};
use crate::error::{PolymarketError, Result};

// =============================================================================
// EIP-712 Domain and Message Types
// =============================================================================

sol! {
    /// EIP-712 domain for Polymarket CLOB.
    #[derive(Debug)]
    struct ClobAuthDomain {
        string name;
        string version;
        uint256 chainId;
    }

    /// EIP-712 message for CLOB authentication.
    #[derive(Debug)]
    struct ClobAuthMessage {
        string message;
        uint256 timestamp;
        uint256 nonce;
    }
}

/// Domain name for EIP-712 signing.
#[allow(dead_code)]
const DOMAIN_NAME: &str = "ClobAuthDomain";

/// Domain version for EIP-712 signing.
#[allow(dead_code)]
const DOMAIN_VERSION: &str = "1";

/// Message content for EIP-712 signing.
#[allow(dead_code)]
const AUTH_MESSAGE: &str = "This message attests that I control the given wallet";

// =============================================================================
// L1 Header Generation (EIP-712)
// =============================================================================

/// Creates L1 authentication headers using EIP-712 signature.
///
/// # Arguments
///
/// * `wallet` - The wallet to sign with.
/// * `chain_id` - The chain ID.
/// * `nonce` - Optional nonce (uses 0 if not provided).
/// * `timestamp` - Optional timestamp (uses current time if not provided).
///
/// # Returns
///
/// L1 authentication headers.
#[allow(dead_code)]
pub async fn create_l1_headers(
    wallet: &PrivateKeySigner,
    chain_id: u64,
    nonce: Option<u64>,
    timestamp: Option<String>,
) -> Result<L1PolyHeader> {
    let ts = timestamp.unwrap_or_else(|| get_current_timestamp().to_string());
    let nonce_val = nonce.unwrap_or(0);

    // Create EIP-712 typed data
    let domain = alloy_sol_types::eip712_domain! {
        name: DOMAIN_NAME,
        version: DOMAIN_VERSION,
        chain_id: chain_id,
    };

    let message = ClobAuthMessage {
        message: AUTH_MESSAGE.to_string(),
        timestamp: alloy_primitives::U256::from(ts.parse::<u64>().unwrap_or(0)),
        nonce: alloy_primitives::U256::from(nonce_val),
    };

    // Sign the typed data
    let signing_hash = message.eip712_signing_hash(&domain);
    let signature = wallet
        .sign_hash(&signing_hash)
        .await
        .map_err(|e| PolymarketError::other(format!("failed to sign L1 message: {}", e)))?;

    Ok(L1PolyHeader {
        poly_address: format!("{:?}", wallet.address()),
        poly_signature: format!("0x{}", hex::encode(signature.as_bytes())),
        poly_timestamp: ts,
        poly_nonce: nonce_val.to_string(),
    })
}

// =============================================================================
// L2 Header Generation (HMAC-SHA256)
// =============================================================================

/// Creates L2 authentication headers using HMAC-SHA256.
///
/// # Arguments
///
/// * `wallet` - The wallet (for address).
/// * `creds` - API credentials.
/// * `method` - HTTP method (GET, POST, DELETE, etc.).
/// * `path` - Request path.
/// * `body` - Optional request body.
/// * `timestamp` - Optional timestamp (uses current time if not provided).
///
/// # Returns
///
/// L2 authentication headers.
pub async fn create_l2_headers(
    wallet: &PrivateKeySigner,
    creds: &ApiKeyCreds,
    method: &str,
    path: &str,
    body: Option<&str>,
    timestamp: Option<String>,
) -> Result<L2PolyHeader> {
    let ts = timestamp.unwrap_or_else(|| get_current_timestamp().to_string());

    // Build message to sign: timestamp + method + path + body
    let body_str = body.unwrap_or("");
    let message = format!("{}{}{}{}", ts, method, path, body_str);

    // HMAC-SHA256 sign
    let secret_bytes = BASE64_STANDARD
        .decode(&creds.secret)
        .map_err(|e| PolymarketError::other(format!("failed to decode API secret: {}", e)))?;

    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_bytes)
        .map_err(|e| PolymarketError::other(format!("failed to create HMAC: {}", e)))?;
    mac.update(message.as_bytes());

    let signature = BASE64_STANDARD.encode(mac.finalize().into_bytes());

    Ok(L2PolyHeader {
        poly_address: format!("{:?}", wallet.address()),
        poly_signature: signature,
        poly_timestamp: ts,
        poly_api_key: creds.key.clone(),
        poly_passphrase: creds.passphrase.clone(),
    })
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Gets the current timestamp in seconds since Unix epoch.
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs()
}

/// Parses an Ethereum address from a string.
#[allow(dead_code)]
pub fn parse_address(addr: &str) -> Result<Address> {
    addr.parse::<Address>()
        .map_err(|e| PolymarketError::other(format!("invalid address '{}': {}", addr, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_timestamp() {
        let ts = get_current_timestamp();
        // Should be a reasonable Unix timestamp (after 2020)
        assert!(ts > 1577836800);
    }

    #[test]
    fn test_parse_address() {
        let valid = "0x0000000000000000000000000000000000000001";
        assert!(parse_address(valid).is_ok());

        let invalid = "not-an-address";
        assert!(parse_address(invalid).is_err());
    }
}
