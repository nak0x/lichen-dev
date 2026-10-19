//! The client-facing REST API (`/api/v1/*`), distinct from the server-to-server
//! ActivityPub surface. It is what `wilder-web` talks to: JSON in, JSON out,
//! authenticated by the session cookie. Federation still happens over the AP
//! endpoints; this is the human/app side.

mod accounts;
mod announcements;
mod auth;
mod determination;
mod feeds;
mod groups;
mod measurements;
mod media;
mod me;
mod notes;
mod posts;

use axum::Router;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Account, Group, Member, Post, Role};
use crate::state::AppState;

/// Aggregate every `/api/v1` sub-router. Mounted by `web::router`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(auth::routes())
        .merge(accounts::routes())
        .merge(groups::routes())
        .merge(posts::routes())
        .merge(feeds::routes())
        .merge(media::routes())
        .merge(me::routes())
        .merge(measurements::routes())
        .merge(notes::routes())
        .merge(determination::routes())
        .merge(announcements::routes())
}

/// Public projection of an account — everything the client may see, and nothing
/// it may not (the password hash never crosses this boundary).
#[derive(Serialize)]
pub struct AccountView {
    pub id: Uuid,
    pub username: String,
    pub role: Role,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub created_at: String,
}

impl From<Account> for AccountView {
    fn from(a: Account) -> Self {
        Self {
            id: a.id,
            username: a.username,
            role: a.role,
            display_name: a.display_name,
            email: a.email,
            created_at: a.created_at.to_rfc3339(),
        }
    }
}

/// Input for creating an account (school bootstrap or student provisioning).
#[derive(Deserialize)]
pub struct NewAccountInput {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

/// Public projection of a group, with the caller's relationship to it.
#[derive(Serialize)]
pub struct GroupView {
    pub username: String,
    pub name: String,
    pub summary: Option<String>,
    pub subject: Option<String>,
    pub created_at: String,
    pub is_member: bool,
    pub is_admin: bool,
    /// The caller's trust tier in the group (`guest`/`contributor`/`admin`), or
    /// `None` if they are not a member — lets the UI tailor the composer (e.g.
    /// warn a guest that their post will be reviewed).
    pub role: Option<crate::domain::MembershipRole>,
}

impl GroupView {
    pub fn from(g: Group, role: Option<crate::domain::MembershipRole>) -> Self {
        use crate::domain::MembershipRole;
        Self {
            username: g.username,
            name: g.name,
            summary: g.summary,
            subject: g.subject,
            created_at: g.created_at.to_rfc3339(),
            is_member: role.is_some(),
            is_admin: role == Some(MembershipRole::Admin),
            role,
        }
    }
}

/// A group member, for the admin roster. `account_id` lets an admin target a
/// member for removal.
#[derive(Serialize)]
pub struct MemberView {
    pub account_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub role: crate::domain::MembershipRole,
}

impl From<Member> for MemberView {
    fn from(m: Member) -> Self {
        Self {
            account_id: m.account_id,
            username: m.username,
            display_name: m.display_name,
            role: m.role,
        }
    }
}

/// Public projection of a post. `activity` carries the raw `Create` for the
/// "read the wire" view the client offers.
#[derive(Serialize)]
pub struct PostView {
    pub id: Uuid,
    pub author: String,
    pub group: Option<String>,
    pub content: Option<String>,
    pub tags: Vec<String>,
    pub object_id: String,
    /// Moderation state (`published`/`pending`/`rejected`) — so the composer can
    /// tell a guest their observation is awaiting review.
    pub status: crate::domain::PostStatus,
    pub published: String,
    pub activity: serde_json::Value,
}

impl From<Post> for PostView {
    fn from(p: Post) -> Self {
        Self {
            id: p.id,
            author: p.author_username,
            group: p.group_username,
            content: p.content,
            tags: p.tags,
            object_id: p.object_id,
            status: p.status,
            published: p.published.to_rfc3339(),
            activity: p.activity,
        }
    }
}
