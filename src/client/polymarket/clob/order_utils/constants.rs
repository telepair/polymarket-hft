//! Constants for order utilities.
//!
//! Contains contract addresses, EIP-712 protocol constants, and token decimals.

use alloy_primitives::Address;
use std::str::FromStr;

use crate::error::{PolymarketError, Result};

// =============================================================================
// EIP-712 Protocol Constants
// =============================================================================

/// Protocol name for EIP-712 domain.
pub const PROTOCOL_NAME: &str = "Polymarket CTF Exchange";

/// Protocol version for EIP-712 domain.
pub const PROTOCOL_VERSION: &str = "1";

/// Order type string for EIP-712 struct hash.
pub const ORDER_TYPE: &str = "Order(uint256 salt,address maker,address signer,address taker,uint256 tokenId,uint256 makerAmount,uint256 takerAmount,uint256 expiration,uint256 nonce,uint256 feeRateBps,uint8 side,uint8 signatureType)";

// =============================================================================
// Token Decimals
// =============================================================================

/// Collateral token decimals (USDC).
pub const COLLATERAL_TOKEN_DECIMALS: u8 = 6;

/// Conditional token decimals.
pub const CONDITIONAL_TOKEN_DECIMALS: u8 = 6;

// =============================================================================
// Contract Addresses (as strings for const initialization)
// =============================================================================

// Polygon mainnet (chain ID 137)
const POLYGON_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";
const POLYGON_NEG_RISK_EXCHANGE: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";
const POLYGON_NEG_RISK_ADAPTER: &str = "0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296";
const POLYGON_COLLATERAL: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
const POLYGON_CONDITIONAL_TOKENS: &str = "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045";

// Amoy testnet (chain ID 80002)
const AMOY_EXCHANGE: &str = "0xdFE02Eb6733538f8Ea35D585af8DE5958AD99E40";
const AMOY_NEG_RISK_EXCHANGE: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";
const AMOY_NEG_RISK_ADAPTER: &str = "0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296";
const AMOY_COLLATERAL: &str = "0x9c4e1703476e875070ee25b56a58b008cfb8fa78";
const AMOY_CONDITIONAL_TOKENS: &str = "0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB";

// =============================================================================
// Contract Configuration
// =============================================================================

/// Contract configuration for a specific chain.
#[derive(Debug, Clone)]
pub struct ContractConfig {
    /// CTF Exchange contract address.
    pub exchange: Address,
    /// Negative risk CTF Exchange contract address.
    pub neg_risk_exchange: Address,
    /// Negative risk adapter contract address.
    pub neg_risk_adapter: Address,
    /// Collateral token (USDC) contract address.
    pub collateral: Address,
    /// Conditional tokens contract address.
    pub conditional_tokens: Address,
}

impl ContractConfig {
    fn polygon() -> Self {
        Self {
            exchange: Address::from_str(POLYGON_EXCHANGE).unwrap(),
            neg_risk_exchange: Address::from_str(POLYGON_NEG_RISK_EXCHANGE).unwrap(),
            neg_risk_adapter: Address::from_str(POLYGON_NEG_RISK_ADAPTER).unwrap(),
            collateral: Address::from_str(POLYGON_COLLATERAL).unwrap(),
            conditional_tokens: Address::from_str(POLYGON_CONDITIONAL_TOKENS).unwrap(),
        }
    }

    fn amoy() -> Self {
        Self {
            exchange: Address::from_str(AMOY_EXCHANGE).unwrap(),
            neg_risk_exchange: Address::from_str(AMOY_NEG_RISK_EXCHANGE).unwrap(),
            neg_risk_adapter: Address::from_str(AMOY_NEG_RISK_ADAPTER).unwrap(),
            collateral: Address::from_str(AMOY_COLLATERAL).unwrap(),
            conditional_tokens: Address::from_str(AMOY_CONDITIONAL_TOKENS).unwrap(),
        }
    }
}

/// Gets contract configuration for a given chain ID.
///
/// # Arguments
///
/// * `chain_id` - The chain ID (137 for Polygon, 80002 for Amoy).
///
/// # Returns
///
/// Returns the contract configuration for the chain, or an error if invalid.
pub fn get_contract_config(chain_id: u64) -> Result<ContractConfig> {
    match chain_id {
        137 => Ok(ContractConfig::polygon()),
        80002 => Ok(ContractConfig::amoy()),
        _ => Err(PolymarketError::other(format!(
            "Invalid chain ID: {}. Supported: 137 (Polygon), 80002 (Amoy)",
            chain_id
        ))),
    }
}

/// Gets the exchange contract address for the given chain and neg_risk flag.
pub fn get_exchange_address(chain_id: u64, neg_risk: bool) -> Result<Address> {
    let config = get_contract_config(chain_id)?;
    Ok(if neg_risk {
        config.neg_risk_exchange
    } else {
        config.exchange
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_contract_config_polygon() {
        let config = get_contract_config(137).unwrap();
        assert_eq!(
            format!("{:?}", config.exchange).to_lowercase(),
            "0x4bfb41d5b3570defd03c39a9a4d8de6bd8b8982e"
        );
    }

    #[test]
    fn test_get_contract_config_amoy() {
        let config = get_contract_config(80002).unwrap();
        assert_eq!(
            format!("{:?}", config.exchange).to_lowercase(),
            "0xdfe02eb6733538f8ea35d585af8de5958ad99e40"
        );
    }

    #[test]
    fn test_get_contract_config_invalid() {
        assert!(get_contract_config(1).is_err());
    }

    #[test]
    fn test_get_exchange_address() {
        let addr = get_exchange_address(137, false).unwrap();
        assert_eq!(
            format!("{:?}", addr).to_lowercase(),
            "0x4bfb41d5b3570defd03c39a9a4d8de6bd8b8982e"
        );

        let neg_addr = get_exchange_address(137, true).unwrap();
        assert_eq!(
            format!("{:?}", neg_addr).to_lowercase(),
            "0xc5d563a36ae78145c45a50134d48a1215220f80a"
        );
    }
}
