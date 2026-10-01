use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::vocab::ActorType;

/// What an account *is*, which decides both its ActivityPub actor type and what
/// it is allowed to do (README §5). A school administers the instance; a student
/// participates; a capture station posts observations automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    School,
    Student,
    /// An automated capture station (camera, sensor): a `Service` actor whose
    /// posts drive the "captured" feed.
    Service,
}

impl Role {
    pub fn as_db(self) -> &'static str {
        match self {
            Role::School => "school",
            Role::Student => "student",
            Role::Service => "service",
        }
    }

    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "school" => Some(Role::School),
            "student" => Some(Role::Student),
            "service" => Some(Role::Service),
            _ => None,
        }
    }

    /// The ActivityPub actor type a new account of this role is created with.
    pub fn actor_kind(self) -> ActorType {
        match self {
            Role::School => ActorType::Organization,
            Role::Student => ActorType::Person,
            Role::Service => ActorType::Service,
        }
    }

    /// Only a school administers the instance: creating accounts, publishing
    /// events and news. This is the single authorization predicate the web
    /// layer's role guard enforces.
    pub fn can_administer_instance(self) -> bool {
        matches!(self, Role::School)
    }
}

/// A local account: credentials and identity, linked 1:1 to a local actor
/// (`actors.username == accounts.username`). The password hash never leaves the
/// domain/service layer — the web layer projects a hash-free view.
#[derive(Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub username: String,
    pub role: Role,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub password_hash: String,
    /// The school that provisioned this account, if any (students are created by
    /// a school; the first school bootstraps itself and has `None`).
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Input for provisioning an account, before a keypair and hash are attached.
#[derive(Debug, Clone)]
pub struct NewAccount {
    pub username: String,
    pub password: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}
