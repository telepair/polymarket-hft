//! Validation functions for API inputs.

use crate::error::{PolymarketError, Result};

// Static error messages to avoid allocations on hot paths
const ERR_USER_MISSING_PREFIX: &str = "address must start with '0x' prefix";
const ERR_USER_INVALID_HEX: &str =
    "user must contain only hexadecimal characters after '0x' prefix";
const ERR_MARKET_MISSING_PREFIX: &str = "market ID must start with '0x' prefix";
const ERR_MARKET_INVALID_HEX: &str =
    "market ID must contain only hexadecimal characters after '0x' prefix";
const ERR_LIMIT_OUT_OF_RANGE: &str = "limit must be between 0 and 500";
const ERR_MIN_BALANCE_OUT_OF_RANGE: &str = "minBalance must be between 0 and 999999";

/// Validates the limit parameter for holders endpoint.
///
/// # Arguments
///
/// * `limit` - Optional limit value (0-500).
///
/// # Returns
///
/// Returns `Ok(())` if the limit is valid or None, or an error if validation fails.
pub(crate) fn validate_limit(limit: Option<i32>) -> Result<()> {
    if let Some(l) = limit
        && !(0..=500).contains(&l)
    {
        return Err(PolymarketError::bad_request(ERR_LIMIT_OUT_OF_RANGE));
    }
    Ok(())
}

/// Validates the minBalance parameter for holders endpoint.
///
/// # Arguments
///
/// * `min_balance` - Optional minimum balance value (0-999999).
///
/// # Returns
///
/// Returns `Ok(())` if the minBalance is valid or None, or an error if validation fails.
pub(crate) fn validate_min_balance(min_balance: Option<i32>) -> Result<()> {
    if let Some(mb) = min_balance
        && !(0..=999999).contains(&mb)
    {
        return Err(PolymarketError::bad_request(ERR_MIN_BALANCE_OUT_OF_RANGE));
    }
    Ok(())
}

/// Validates a user address.
///
/// A valid user address must:
/// - Start with "0x" prefix
/// - Be exactly 42 characters long (0x + 40 hex chars)
/// - Contain only valid hexadecimal characters after the prefix
pub(crate) fn validate_user(user: &str) -> Result<()> {
    // Check if starts with 0x
    if !user.starts_with("0x") && !user.starts_with("0X") {
        return Err(PolymarketError::bad_request(ERR_USER_MISSING_PREFIX));
    }

    // Check length: 0x + 40 hex chars = 42 chars
    if user.len() != 42 {
        return Err(PolymarketError::bad_request(format!(
            "user must be 42 characters long, got {}",
            user.len()
        )));
    }

    // Check if remaining characters are valid hex
    let hex_part = &user[2..];
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PolymarketError::bad_request(ERR_USER_INVALID_HEX));
    }

    Ok(())
}

/// Validates a market ID.
///
/// A valid market ID must:
/// - Start with "0x" prefix
/// - Be exactly 66 characters long (0x + 64 hex chars)
/// - Contain only valid hexadecimal characters after the prefix
pub(crate) fn validate_market_id(market_id: &str) -> Result<()> {
    // Check if starts with 0x
    if !market_id.starts_with("0x") && !market_id.starts_with("0X") {
        return Err(PolymarketError::bad_request(ERR_MARKET_MISSING_PREFIX));
    }

    // Check length: 0x + 64 hex chars = 66 chars
    if market_id.len() != 66 {
        return Err(PolymarketError::bad_request(format!(
            "market ID must be 66 characters long, got {}",
            market_id.len()
        )));
    }

    // Check if remaining characters are valid hex
    let hex_part = &market_id[2..];
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PolymarketError::bad_request(ERR_MARKET_INVALID_HEX));
    }

    Ok(())
}

/// Validates an event ID.
///
/// A valid event ID must be a positive integer (>= 1).
pub(crate) fn validate_event_id(event_id: i64) -> Result<()> {
    if event_id < 1 {
        return Err(PolymarketError::bad_request(format!(
            "event ID must be >= 1, got {}",
            event_id
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_user_valid() {
        assert!(validate_user("0x0123456789012345678901234567890123456789").is_ok());
        assert!(validate_user("0xABCDEF1234567890abcdef1234567890ABCDEF12").is_ok());
        assert!(validate_user("0X0123456789012345678901234567890123456789").is_ok());
    }

    #[test]
    fn test_validate_user_missing_prefix() {
        let result = validate_user("0123456789012345678901234567890123456789");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("0x"));
    }

    #[test]
    fn test_validate_user_too_short() {
        let result = validate_user("0x012345678901234567890123456789012345678");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("42"));
    }

    #[test]
    fn test_validate_user_too_long() {
        let result = validate_user("0x01234567890123456789012345678901234567890");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("42"));
    }

    #[test]
    fn test_validate_user_invalid_chars() {
        let result = validate_user("0x012345678901234567890123456789012345678z");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("hexadecimal"));
    }

    #[test]
    fn test_validate_market_id_valid() {
        assert!(
            validate_market_id(
                "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917"
            )
            .is_ok()
        );
        assert!(
            validate_market_id(
                "0XDD22472E552920B8438158EA7238BFADFA4F736AA4CEE91A6B86C39EAD110917"
            )
            .is_ok()
        );
    }

    #[test]
    fn test_validate_market_id_missing_prefix() {
        let result =
            validate_market_id("dd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("0x"));
    }

    #[test]
    fn test_validate_market_id_too_short() {
        let result =
            validate_market_id("0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead11091");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("66"));
    }

    #[test]
    fn test_validate_market_id_too_long() {
        let result = validate_market_id(
            "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead1109170",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("66"));
    }

    #[test]
    fn test_validate_market_id_invalid_chars() {
        let result = validate_market_id(
            "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead11091z",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("hexadecimal"));
    }

    #[test]
    fn test_validate_event_id_valid() {
        assert!(validate_event_id(1).is_ok());
        assert!(validate_event_id(123).is_ok());
        assert!(validate_event_id(i64::MAX).is_ok());
    }

    #[test]
    fn test_validate_event_id_zero() {
        let result = validate_event_id(0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(">= 1"));
    }

    #[test]
    fn test_validate_event_id_negative() {
        let result = validate_event_id(-1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(">= 1"));
    }

    #[test]
    fn test_validate_limit_none() {
        assert!(validate_limit(None).is_ok());
    }

    #[test]
    fn test_validate_limit_valid() {
        assert!(validate_limit(Some(0)).is_ok());
        assert!(validate_limit(Some(100)).is_ok());
        assert!(validate_limit(Some(500)).is_ok());
    }

    #[test]
    fn test_validate_limit_out_of_range() {
        let result = validate_limit(Some(-1));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("limit"));

        let result = validate_limit(Some(501));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("limit"));
    }

    #[test]
    fn test_validate_min_balance_none() {
        assert!(validate_min_balance(None).is_ok());
    }

    #[test]
    fn test_validate_min_balance_valid() {
        assert!(validate_min_balance(Some(0)).is_ok());
        assert!(validate_min_balance(Some(1000)).is_ok());
        assert!(validate_min_balance(Some(999999)).is_ok());
    }

    #[test]
    fn test_validate_min_balance_out_of_range() {
        let result = validate_min_balance(Some(-1));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("minBalance"));

        let result = validate_min_balance(Some(1000000));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("minBalance"));
    }
}
