//! Holders API types.

use serde::{Deserialize, Serialize};

/// A holder's position information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holder {
    /// Proxy wallet address.
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    /// User bio.
    pub bio: String,
    /// Asset identifier.
    pub asset: String,
    /// User pseudonym.
    pub pseudonym: String,
    /// Position amount.
    pub amount: f64,
    /// Whether username is public.
    #[serde(rename = "displayUsernamePublic")]
    pub display_username_public: bool,
    /// Outcome index.
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    /// User name.
    pub name: String,
    /// Profile image URL.
    #[serde(rename = "profileImage")]
    pub profile_image: String,
    /// Optimized profile image URL.
    #[serde(rename = "profileImageOptimized")]
    pub profile_image_optimized: String,
}

/// Response from the holders endpoint.
///
/// Contains the token identifier and list of holders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTopHolders {
    /// Token identifier.
    pub token: String,
    /// List of holders for this token.
    pub holders: Vec<Holder>,
}
