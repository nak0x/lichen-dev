use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Metadata for one stored media object. The bytes live in the object store
/// under `object_key`; this is the durable record that maps a public id to them.
#[derive(Debug, Clone)]
pub struct MediaRef {
    pub id: Uuid,
    pub owner_username: String,
    pub object_key: String,
    pub content_type: String,
    pub byte_size: i64,
    pub created_at: DateTime<Utc>,
}
