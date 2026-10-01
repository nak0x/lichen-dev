use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// A published post (the read-model of a `Create`+`Note`). An observation is
/// just a post whose content describes a sighting, carries a `#species` tag and
/// — later — image attachments and a place.
#[derive(Debug, Clone)]
pub struct Post {
    pub id: Uuid,
    pub author_username: String,
    /// The group this was posted into, or `None` for a personal post.
    pub group_username: Option<String>,
    pub content: Option<String>,
    pub tags: Vec<String>,
    pub object_id: String,
    /// The canonical `Create` activity, kept for the raw JSON view and re-delivery.
    pub activity: Value,
    pub published: DateTime<Utc>,
}

/// Input for composing a post.
#[derive(Debug, Clone)]
pub struct NewPost {
    /// Target group username, or `None` to post to the author's own followers.
    pub group_username: Option<String>,
    pub content: String,
    pub tags: Vec<String>,
}
