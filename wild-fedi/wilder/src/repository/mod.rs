//! Persistence boundaries. Services and handlers depend on these *traits*, never
//! on a concrete store — the Dependency-Inversion pin that lets us swap the
//! in-memory store for Postgres/SQLite without touching anything upstream.

pub mod memory;

use async_trait::async_trait;
use serde_json::Value;

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
}

/// A remote follower we deliver to. We cache the resolved `inbox` so we don't
/// re-fetch the actor document on every delivery.
#[derive(Debug, Clone)]
pub struct Follower {
    pub actor_id: String,
    pub inbox: String,
}

#[async_trait]
pub trait ActorRepository: Send + Sync {
    async fn get(&self, username: &str) -> Option<LocalActor>;
    async fn insert(&self, actor: LocalActor);
    async fn count(&self) -> usize;
}

#[async_trait]
pub trait FollowerRepository: Send + Sync {
    async fn add(&self, username: &str, follower: Follower);
    async fn remove(&self, username: &str, follower_actor_id: &str);
    async fn list(&self, username: &str) -> Vec<Follower>;
}

/// Stores activities we publish (outbox) and ones we accept (inbox history).
#[async_trait]
pub trait ActivityRepository: Send + Sync {
    async fn append_outbox(&self, username: &str, activity: Value);
    async fn outbox(&self, username: &str) -> Vec<Value>;
    async fn record_inbox(&self, username: &str, activity: Value);
}
