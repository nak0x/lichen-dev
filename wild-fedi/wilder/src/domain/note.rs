use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// What a personal document is: a short self-note or a longer research write-up.
/// Both are private to their owner; the kind only shapes how the client presents
/// them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteKind {
    Note,
    Research,
}

impl NoteKind {
    pub fn as_db(self) -> &'static str {
        match self {
            NoteKind::Note => "note",
            NoteKind::Research => "research",
        }
    }
    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "note" => Some(NoteKind::Note),
            "research" => Some(NoteKind::Research),
            _ => None,
        }
    }
}

/// A private personal document.
#[derive(Debug, Clone)]
pub struct Note {
    pub id: Uuid,
    pub owner_username: String,
    pub kind: NoteKind,
    pub title: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewNote {
    pub kind: NoteKind,
    pub title: Option<String>,
    pub content: String,
}
