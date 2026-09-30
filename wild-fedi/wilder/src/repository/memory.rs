use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::RwLock;

use super::{ActivityRepository, ActorRepository, Follower, FollowerRepository, LocalActor};

/// One in-memory store implementing all three repository traits. Fine for a
/// prototype and tests; everything is lost on restart. Making it durable means
/// implementing the same traits over a database — nothing upstream changes.
///
/// We use `tokio::sync::RwLock` (not `std`) so a future handler that needs to
/// `.await` while holding a guard won't deadlock the runtime.
#[derive(Default)]
pub struct InMemoryStore {
    actors: RwLock<HashMap<String, LocalActor>>,
    followers: RwLock<HashMap<String, Vec<Follower>>>,
    outboxes: RwLock<HashMap<String, Vec<Value>>>,
    inboxes: RwLock<HashMap<String, Vec<Value>>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ActorRepository for InMemoryStore {
    async fn get(&self, username: &str) -> Option<LocalActor> {
        self.actors.read().await.get(username).cloned()
    }
    async fn insert(&self, actor: LocalActor) {
        self.actors
            .write()
            .await
            .insert(actor.username.clone(), actor);
    }
    async fn count(&self) -> usize {
        self.actors.read().await.len()
    }
}

#[async_trait]
impl FollowerRepository for InMemoryStore {
    async fn add(&self, username: &str, follower: Follower) {
        let mut guard = self.followers.write().await;
        let list = guard.entry(username.to_string()).or_default();
        // Idempotency: a re-sent Follow must not create a duplicate follower.
        if !list.iter().any(|f| f.actor_id == follower.actor_id) {
            list.push(follower);
        }
    }
    async fn remove(&self, username: &str, follower_actor_id: &str) {
        if let Some(list) = self.followers.write().await.get_mut(username) {
            list.retain(|f| f.actor_id != follower_actor_id);
        }
    }
    async fn list(&self, username: &str) -> Vec<Follower> {
        self.followers
            .read()
            .await
            .get(username)
            .cloned()
            .unwrap_or_default()
    }
}

#[async_trait]
impl ActivityRepository for InMemoryStore {
    async fn append_outbox(&self, username: &str, activity: Value) {
        self.outboxes
            .write()
            .await
            .entry(username.to_string())
            .or_default()
            .push(activity);
    }
    async fn outbox(&self, username: &str) -> Vec<Value> {
        self.outboxes
            .read()
            .await
            .get(username)
            .cloned()
            .unwrap_or_default()
    }
    async fn record_inbox(&self, username: &str, activity: Value) {
        self.inboxes
            .write()
            .await
            .entry(username.to_string())
            .or_default()
            .push(activity);
    }
}
