use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// An instance-level publication by a school. An `Event` carries a time and
/// place; `News` is a dated write-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnouncementKind {
    Event,
    News,
}

impl AnnouncementKind {
    pub fn as_db(self) -> &'static str {
        match self {
            AnnouncementKind::Event => "event",
            AnnouncementKind::News => "news",
        }
    }
    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "event" => Some(AnnouncementKind::Event),
            "news" => Some(AnnouncementKind::News),
            _ => None,
        }
    }
    /// The ActivityStreams object type this maps to on the wire.
    pub fn as_object_type(self) -> &'static str {
        match self {
            AnnouncementKind::Event => "Event",
            AnnouncementKind::News => "Article",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Announcement {
    pub id: Uuid,
    pub kind: AnnouncementKind,
    pub author_username: String,
    pub title: String,
    pub content: String,
    pub start_time: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub object_id: String,
    pub activity: Value,
    pub published: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewAnnouncement {
    pub kind: AnnouncementKind,
    pub title: String,
    pub content: String,
    pub start_time: Option<DateTime<Utc>>,
    pub location: Option<String>,
}
