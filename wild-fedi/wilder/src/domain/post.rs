use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Where a post sits in the moderation workflow:
/// - `Published`: live — in feeds, group views and federated (the default, and
///   what contributors/admins produce directly).
/// - `Pending`: a guest's submission awaiting an admin's decision; invisible
///   everywhere except the review queue, and not yet federated.
/// - `Rejected`: an admin declined it; kept so the author can be told, but never
///   surfaced publicly nor federated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PostStatus {
    Published,
    Pending,
    Rejected,
}

impl PostStatus {
    pub fn as_db(self) -> &'static str {
        match self {
            PostStatus::Published => "published",
            PostStatus::Pending => "pending",
            PostStatus::Rejected => "rejected",
        }
    }
    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "published" => Some(PostStatus::Published),
            "pending" => Some(PostStatus::Pending),
            "rejected" => Some(PostStatus::Rejected),
            _ => None,
        }
    }
}

/// A post (the read-model of a `Create`+`Note`). An observation is just a post
/// whose content describes a sighting, carries a `#species` tag and — later —
/// image attachments and a place.
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
    /// Moderation state — gates visibility and federation.
    pub status: PostStatus,
    pub published: DateTime<Utc>,
}

/// Input for composing a post.
#[derive(Debug, Clone)]
pub struct NewPost {
    /// Target group username, or `None` to post to the author's own followers.
    pub group_username: Option<String>,
    pub content: String,
    pub tags: Vec<String>,
    /// Ids of previously-uploaded images to attach (the picture space).
    pub media: Vec<uuid::Uuid>,
}
