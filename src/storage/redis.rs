//! Redis client for state management.

use crate::engine::StateEntry;
use redis::{AsyncCommands, Client, aio::ConnectionManager};
use std::time::Duration;

/// Default TTL for state entries (15 minutes).
const DEFAULT_TTL_SECS: u64 = 900;

/// Redis client for storing and retrieving state entries.
#[derive(Clone)]
pub struct RedisClient {
    conn: ConnectionManager,
    default_ttl: Duration,
}

impl RedisClient {
    /// Creates a new Redis client.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection string (e.g., "redis://localhost:6379").
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self {
            conn,
            default_ttl: Duration::from_secs(DEFAULT_TTL_SECS),
        })
    }

    /// Creates a new client with a custom default TTL.
    pub async fn with_ttl(
        redis_url: &str,
        default_ttl: Duration,
    ) -> Result<Self, redis::RedisError> {
        let mut client = Self::new(redis_url).await?;
        client.default_ttl = default_ttl;
        Ok(client)
    }

    /// Sets a state entry with TTL.
    pub async fn set_state(&self, entry: &StateEntry) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        let value = entry.value.to_string();
        let ttl = entry.ttl.unwrap_or(self.default_ttl);

        conn.set_ex(&entry.key, value, ttl.as_secs()).await
    }

    /// Sets multiple state entries.
    pub async fn set_states(&self, entries: &[StateEntry]) -> Result<(), redis::RedisError> {
        for entry in entries {
            self.set_state(entry).await?;
        }
        Ok(())
    }

    /// Gets a state entry by key.
    pub async fn get_state(
        &self,
        key: &str,
    ) -> Result<Option<serde_json::Value>, redis::RedisError> {
        let mut conn = self.conn.clone();
        let value: Option<String> = conn.get(key).await?;

        Ok(value.and_then(|v| serde_json::from_str(&v).ok()))
    }

    /// Gets multiple state entries by pattern.
    pub async fn get_states_by_pattern(
        &self,
        pattern: &str,
    ) -> Result<Vec<(String, serde_json::Value)>, redis::RedisError> {
        let mut conn = self.conn.clone();
        let keys: Vec<String> = conn.keys(pattern).await?;

        let mut results = Vec::new();
        for key in keys {
            if let Some(value) = self.get_state(&key).await? {
                results.push((key, value));
            }
        }

        Ok(results)
    }

    /// Deletes a state entry.
    pub async fn delete_state(&self, key: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.conn.clone();
        let deleted: i64 = conn.del(key).await?;
        Ok(deleted > 0)
    }

    /// Gets the TTL of a key in seconds.
    pub async fn get_ttl(&self, key: &str) -> Result<Option<i64>, redis::RedisError> {
        let mut conn = self.conn.clone();
        let ttl: i64 = conn.ttl(key).await?;

        // Redis returns -2 if key doesn't exist, -1 if no TTL
        if ttl < 0 { Ok(None) } else { Ok(Some(ttl)) }
    }

    /// Checks if a key exists.
    pub async fn exists(&self, key: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.conn.clone();
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    /// Gets all state keys matching a source pattern.
    pub async fn get_source_keys(&self, source: &str) -> Result<Vec<String>, redis::RedisError> {
        let pattern = format!("state:{}:*", source);
        let mut conn = self.conn.clone();
        conn.keys(pattern).await
    }
}
