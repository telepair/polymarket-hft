//! Helper functions for Gamma API types.

use serde::{Deserialize, Deserializer, de::Error as DeError};
use serde_json::Value;

use crate::error::{PolymarketError, Result};

/// Deserializes a field that may be a number, string, or null into `Option<f64>`.
pub fn deserialize_option_f64<'de, D>(deserializer: D) -> std::result::Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        Some(Value::Number(num)) => num
            .as_f64()
            .ok_or_else(|| DeError::custom("expected f64-compatible number"))
            .map(Some),
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                trimmed
                    .parse::<f64>()
                    .map(Some)
                    .map_err(|e| DeError::custom(format!("invalid float string: {}", e)))
            }
        }
        Some(Value::Bool(b)) => Ok(Some(if b { 1.0 } else { 0.0 })),
        Some(Value::Null) => Ok(None),
        None => Ok(None),
        other => Err(DeError::custom(format!(
            "expected number or string for float field, got {:?}",
            other
        ))),
    }
}

/// Deserializes a field that may be a number, string, or null into `Option<u64>`.
/// Returns `None` for negative numbers instead of failing.
pub fn deserialize_option_u64<'de, D>(deserializer: D) -> std::result::Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        Some(Value::Number(num)) => Ok(num.as_u64()),
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                // Return None for negative numbers or invalid strings
                Ok(trimmed.parse::<u64>().ok())
            }
        }
        Some(Value::Bool(b)) => Ok(Some(if b { 1 } else { 0 })),
        Some(Value::Null) => Ok(None),
        None => Ok(None),
        _ => Ok(None),
    }
}

/// Deserializes a field that may be a number, string, or null into `Option<i64>`.
pub fn deserialize_option_i64<'de, D>(deserializer: D) -> std::result::Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        Some(Value::Number(num)) => num
            .as_i64()
            .ok_or_else(|| DeError::custom("expected i64-compatible number"))
            .map(Some),
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                trimmed
                    .parse::<i64>()
                    .map(Some)
                    .map_err(|e| DeError::custom(format!("invalid integer string: {}", e)))
            }
        }
        Some(Value::Bool(b)) => Ok(Some(if b { 1 } else { 0 })),
        Some(Value::Null) => Ok(None),
        None => Ok(None),
        other => Err(DeError::custom(format!(
            "expected number or string for integer field, got {:?}",
            other
        ))),
    }
}

/// Validates numeric tag IDs (all digits).
pub(crate) fn validate_tag_id(tag_id: Option<&str>) -> Result<()> {
    if let Some(id) = tag_id {
        if id.is_empty() {
            return Err(PolymarketError::bad_request("tag_id cannot be empty"));
        }

        if !id.chars().all(|c| c.is_ascii_digit()) {
            return Err(PolymarketError::bad_request(
                "tag_id must contain only digits".to_string(),
            ));
        }
    }
    Ok(())
}

const ALLOWED_COMMENT_ENTITY_TYPES: [&str; 3] = ["Event", "Series", "market"];

/// Validates the required parent entity filters for comments.
pub(crate) fn validate_comment_parent(
    entity_type: Option<&str>,
    entity_id: Option<&str>,
) -> Result<()> {
    match (entity_type, entity_id) {
        (Some(entity_type), Some(entity_id)) => {
            if entity_type.trim().is_empty() {
                return Err(PolymarketError::bad_request(
                    "parent_entity_type cannot be empty",
                ));
            }

            if !ALLOWED_COMMENT_ENTITY_TYPES.contains(&entity_type) {
                return Err(PolymarketError::bad_request(format!(
                    "parent_entity_type must be one of {:?}",
                    ALLOWED_COMMENT_ENTITY_TYPES
                )));
            }

            if entity_id.trim().is_empty() {
                return Err(PolymarketError::bad_request(
                    "parent_entity_id cannot be empty",
                ));
            }
        }
        (None, None) => {
            return Err(PolymarketError::bad_request(
                "parent_entity_type and parent_entity_id are required",
            ));
        }
        _ => {
            return Err(PolymarketError::bad_request(
                "parent_entity_type and parent_entity_id must both be provided",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_tag_id_rejects_non_digits() {
        let err = validate_tag_id(Some("abc")).unwrap_err();
        assert!(err.to_string().contains("digits"));
    }

    #[test]
    fn validate_comment_parent_requires_both_fields() {
        let missing = validate_comment_parent(None, None).unwrap_err();
        assert!(missing.to_string().contains("required"));

        let partial = validate_comment_parent(Some("Event"), None).unwrap_err();
        assert!(partial.to_string().contains("must both be provided"));
    }

    #[test]
    fn validate_comment_parent_rejects_invalid_type() {
        let err = validate_comment_parent(Some("InvalidType"), Some("id")).unwrap_err();
        assert!(
            err.to_string()
                .contains("parent_entity_type must be one of")
        );
    }

    #[test]
    fn validate_comment_parent_accepts_valid_input() {
        assert!(validate_comment_parent(Some("Event"), Some("123")).is_ok());
    }
}
