use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A group actor: a class or study circle others follow and post into.
#[derive(Debug, Clone)]
pub struct Group {
    pub username: String,
    pub name: String,
    pub summary: Option<String>,
    /// The naturalist subject under study — the thread posts hang off of.
    pub subject: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a group. `username` is the actor slug; name is the label.
#[derive(Debug, Clone)]
pub struct NewGroup {
    pub username: String,
    pub name: String,
    pub summary: Option<String>,
    pub subject: Option<String>,
}

/// A member's standing in a group. The creator is an `Admin`; everyone else a
/// `Member`. Only admins moderate and manage membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MembershipRole {
    Admin,
    Member,
}

impl MembershipRole {
    pub fn as_db(self) -> &'static str {
        match self {
            MembershipRole::Admin => "admin",
            MembershipRole::Member => "member",
        }
    }
    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(MembershipRole::Admin),
            "member" => Some(MembershipRole::Member),
            _ => None,
        }
    }
}

/// A member, joined with their account identity for display.
#[derive(Debug, Clone)]
pub struct Member {
    pub account_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub role: MembershipRole,
}
