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

/// A member's standing in a group — a trust tier the admin assigns:
/// - `Guest`: may read and submit, but submissions go to the admin review queue.
/// - `Contributor`: may post without review (the default trusted member).
/// - `Admin`: all of the above, plus moderation and membership management.
///
/// The group creator is an `Admin`; new joiners start as `Guest` until an admin
/// promotes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MembershipRole {
    Admin,
    Contributor,
    Guest,
}

impl MembershipRole {
    pub fn as_db(self) -> &'static str {
        match self {
            MembershipRole::Admin => "admin",
            MembershipRole::Contributor => "contributor",
            MembershipRole::Guest => "guest",
        }
    }
    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(MembershipRole::Admin),
            "contributor" => Some(MembershipRole::Contributor),
            "guest" => Some(MembershipRole::Guest),
            // Legacy rows written before the trust tiers existed could post
            // directly, so they map to `contributor`.
            "member" => Some(MembershipRole::Contributor),
            _ => None,
        }
    }

    /// May this member publish immediately, without admin review? Admins and
    /// contributors can; guests are moderated.
    pub fn can_post_without_review(self) -> bool {
        matches!(self, MembershipRole::Admin | MembershipRole::Contributor)
    }

    /// May this member moderate the review queue and manage membership/roles?
    pub fn can_moderate(self) -> bool {
        matches!(self, MembershipRole::Admin)
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
