//! Persistence boundaries. Services and handlers depend on these *traits*, never
//! on a concrete store — the Dependency-Inversion pin that lets us swap the
//! in-memory store for Postgres/SQLite without touching anything upstream.

// The in-memory store is a test double: it lets services be unit-tested against
// the repository traits without a live Postgres. Production wires `PgStore`.
#[cfg(test)]
pub mod memory;
pub mod postgres;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::{Account, Group, Member, MediaRef, MembershipRole, Post};
use crate::vocab::ActorType;

/// A locally-hosted actor: the parts we persist. A richer server would attach
/// profile media, capabilities, etc.; a bare one keeps only what the seven
/// ActivityPub contracts require.
#[derive(Debug, Clone)]
pub struct LocalActor {
    pub username: String,
    pub kind: ActorType,
    pub display_name: Option<String>,
    pub summary: Option<String>,
    pub private_key_pem: String,
    pub public_key_pem: String,
    /// Avatar URL (ActivityPub `icon`), set once the actor uploads one.
    pub icon_url: Option<String>,
}

/// A remote follower we deliver to. We cache the resolved `inbox` so we don't
/// re-fetch the actor document on every delivery.
#[derive(Debug, Clone)]
pub struct Follower {
    pub actor_id: String,
    pub inbox: String,
}

// The three federation-substrate traits now return `anyhow::Result`: a real
// database can fail, and silently dropping a write (a published post that never
// persisted) is not acceptable. Errors flow through `?` into `AppError::Internal`
// at the web edge. The in-memory impl is infallible and simply wraps in `Ok`.

#[async_trait]
pub trait ActorRepository: Send + Sync {
    async fn get(&self, username: &str) -> anyhow::Result<Option<LocalActor>>;
    async fn insert(&self, actor: LocalActor) -> anyhow::Result<()>;
    async fn count(&self) -> anyhow::Result<usize>;
    /// Set the actor's avatar URL (profile avatar upload).
    async fn set_icon(&self, username: &str, icon_url: &str) -> anyhow::Result<()>;
    /// Update editable profile fields (display name, bio). `None` leaves a field
    /// unchanged.
    async fn update_bio(
        &self,
        username: &str,
        display_name: Option<&str>,
        summary: Option<&str>,
    ) -> anyhow::Result<()>;
}

#[async_trait]
pub trait FollowerRepository: Send + Sync {
    async fn add(&self, username: &str, follower: Follower) -> anyhow::Result<()>;
    async fn remove(&self, username: &str, follower_actor_id: &str) -> anyhow::Result<()>;
    async fn list(&self, username: &str) -> anyhow::Result<Vec<Follower>>;
}

/// Stores activities we publish (outbox) and ones we accept (inbox history).
#[async_trait]
pub trait ActivityRepository: Send + Sync {
    async fn append_outbox(&self, username: &str, activity: Value) -> anyhow::Result<()>;
    async fn outbox(&self, username: &str) -> anyhow::Result<Vec<Value>>;
    async fn record_inbox(&self, username: &str, activity: Value) -> anyhow::Result<()>;
}

/// Accounts: credentials, roles, and who provisioned whom.
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn by_username(&self, username: &str) -> anyhow::Result<Option<Account>>;
    async fn by_id(&self, id: Uuid) -> anyhow::Result<Option<Account>>;
    async fn insert(&self, account: &Account) -> anyhow::Result<()>;
    /// Total account count — used to gate first-run school bootstrap.
    async fn count(&self) -> anyhow::Result<usize>;
    /// Accounts a given school created (for its admin roster view).
    async fn list_created_by(&self, school_id: Uuid) -> anyhow::Result<Vec<Account>>;
}

/// Groups and their memberships. Kept in one trait because the two are always
/// manipulated together (creating a group seeds its first membership).
#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn insert(&self, group: &Group) -> anyhow::Result<()>;
    async fn by_username(&self, username: &str) -> anyhow::Result<Option<Group>>;
    async fn all(&self) -> anyhow::Result<Vec<Group>>;
    /// Groups an account belongs to.
    async fn for_account(&self, account_id: Uuid) -> anyhow::Result<Vec<Group>>;

    async fn add_member(
        &self,
        group_username: &str,
        account_id: Uuid,
        role: MembershipRole,
    ) -> anyhow::Result<()>;
    async fn remove_member(&self, group_username: &str, account_id: Uuid) -> anyhow::Result<()>;
    async fn members(&self, group_username: &str) -> anyhow::Result<Vec<Member>>;
    async fn role_of(
        &self,
        group_username: &str,
        account_id: Uuid,
    ) -> anyhow::Result<Option<MembershipRole>>;
}

/// Posts: the queryable read-model powering feeds, filtering and group views.
#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn insert(&self, post: &Post) -> anyhow::Result<()>;
    /// Posts in a group, optionally filtered to a single tag, newest first.
    async fn in_group(
        &self,
        group_username: &str,
        tag: Option<&str>,
        limit: i64,
    ) -> anyhow::Result<Vec<Post>>;
    /// Group posts since a timestamp, across several groups — the personal feed.
    async fn in_groups_since(
        &self,
        group_usernames: &[String],
        since: DateTime<Utc>,
        limit: i64,
    ) -> anyhow::Result<Vec<Post>>;
    /// Posts whose author is a `Service` actor — the "captured" feed. Scoped to a
    /// group when given, else instance-wide.
    async fn captured(&self, group_username: Option<&str>, limit: i64)
    -> anyhow::Result<Vec<Post>>;
}

/// Media metadata (the bytes live in the `MediaStore`).
#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn insert(&self, media: &MediaRef) -> anyhow::Result<()>;
    async fn by_id(&self, id: Uuid) -> anyhow::Result<Option<MediaRef>>;
}
