//! Comments message types (Comment, Reaction).

use serde::{Deserialize, Serialize};

/// Comment payload for "comment_created" and "comment_removed" messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    /// Comment ID.
    pub id: u64,

    /// Comment body text.
    #[serde(default)]
    pub body: String,

    /// Parent entity type (Event or Series).
    #[serde(default)]
    pub parent_entity_type: String,

    /// Parent entity ID.
    #[serde(default)]
    pub parent_entity_id: u64,

    /// Parent comment ID (for replies).
    #[serde(default)]
    pub parent_comment_id: Option<u64>,

    /// User wallet address.
    #[serde(default)]
    pub user_address: String,

    /// Reply target address.
    #[serde(default)]
    pub reply_address: Option<String>,

    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,

    /// Last update timestamp.
    #[serde(default)]
    pub updated_at: String,
}

/// Reaction payload for "reaction_created" and "reaction_removed" messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    /// Reaction ID.
    pub id: u64,

    /// Comment ID this reaction belongs to.
    #[serde(default)]
    pub comment_id: u64,

    /// Reaction type (e.g., "like", "dislike").
    #[serde(default)]
    pub reaction_type: String,

    /// Reaction icon.
    #[serde(default)]
    pub icon: String,

    /// User wallet address.
    #[serde(default)]
    pub user_address: String,

    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_deserialize() {
        let json = r#"{
            "id": 123,
            "body": "Test comment",
            "parentEntityType": "Event",
            "parentEntityId": 456,
            "userAddress": "0x123"
        }"#;

        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, 123);
        assert_eq!(comment.body, "Test comment");
        assert_eq!(comment.parent_entity_type, "Event");
    }

    #[test]
    fn test_reaction_deserialize() {
        let json = r#"{
            "id": 1,
            "commentId": 123,
            "reactionType": "like",
            "userAddress": "0x456"
        }"#;

        let reaction: Reaction = serde_json::from_str(json).unwrap();
        assert_eq!(reaction.id, 1);
        assert_eq!(reaction.comment_id, 123);
        assert_eq!(reaction.reaction_type, "like");
    }
}
