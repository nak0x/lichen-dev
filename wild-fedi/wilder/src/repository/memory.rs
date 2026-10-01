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

// The in-memory store cannot fail, so every method simply wraps its result in
// `Ok` to satisfy the now-fallible trait signatures.

#[async_trait]
impl ActorRepository for InMemoryStore {
    async fn get(&self, username: &str) -> anyhow::Result<Option<LocalActor>> {
        Ok(self.actors.read().await.get(username).cloned())
    }
    async fn insert(&self, actor: LocalActor) -> anyhow::Result<()> {
        self.actors
            .write()
            .await
            .insert(actor.username.clone(), actor);
        Ok(())
    }
    async fn count(&self) -> anyhow::Result<usize> {
        Ok(self.actors.read().await.len())
    }
}

#[async_trait]
impl FollowerRepository for InMemoryStore {
    async fn add(&self, username: &str, follower: Follower) -> anyhow::Result<()> {
        let mut guard = self.followers.write().await;
        let list = guard.entry(username.to_string()).or_default();
        // Idempotency: a re-sent Follow must not create a duplicate follower.
        if !list.iter().any(|f| f.actor_id == follower.actor_id) {
            list.push(follower);
        }
        Ok(())
    }
    async fn remove(&self, username: &str, follower_actor_id: &str) -> anyhow::Result<()> {
        if let Some(list) = self.followers.write().await.get_mut(username) {
            list.retain(|f| f.actor_id != follower_actor_id);
        }
        Ok(())
    }
    async fn list(&self, username: &str) -> anyhow::Result<Vec<Follower>> {
        Ok(self
            .followers
            .read()
            .await
            .get(username)
            .cloned()
            .unwrap_or_default())
    }
}

#[async_trait]
impl ActivityRepository for InMemoryStore {
    async fn append_outbox(&self, username: &str, activity: Value) -> anyhow::Result<()> {
        self.outboxes
            .write()
            .await
            .entry(username.to_string())
            .or_default()
            .push(activity);
        Ok(())
    }
    async fn outbox(&self, username: &str) -> anyhow::Result<Vec<Value>> {
        Ok(self
            .outboxes
            .read()
            .await
            .get(username)
            .cloned()
            .unwrap_or_default())
    }
    async fn record_inbox(&self, username: &str, activity: Value) -> anyhow::Result<()> {
        self.inboxes
            .write()
            .await
            .entry(username.to_string())
            .or_default()
            .push(activity);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocab::ActorType;
    use serde_json::json;

    fn sample_actor(username: &str) -> LocalActor {
        LocalActor {
            username: username.to_string(),
            kind: ActorType::Person,
            display_name: Some("Alice".into()),
            summary: None,
            private_key_pem: "priv".into(),
            public_key_pem: "pub".into(),
        }
    }

    #[tokio::test]
    async fn actor_roundtrip_and_count() {
        let store = InMemoryStore::new();
        assert_eq!(store.count().await.unwrap(), 0);

        store.insert(sample_actor("alice")).await.unwrap();
        let got = store.get("alice").await.unwrap().expect("actor present");
        assert_eq!(got.username, "alice");
        assert_eq!(store.count().await.unwrap(), 1);
        assert!(store.get("bob").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn followers_add_is_idempotent_and_removable() {
        let store = InMemoryStore::new();
        let follower = Follower {
            actor_id: "https://remote/u".into(),
            inbox: "https://remote/u/inbox".into(),
        };
        store.add("alice", follower.clone()).await.unwrap();
        store.add("alice", follower.clone()).await.unwrap(); // duplicate ignored
        assert_eq!(store.list("alice").await.unwrap().len(), 1);

        store.remove("alice", "https://remote/u").await.unwrap();
        assert!(store.list("alice").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn outbox_appends_and_reads_back() {
        let store = InMemoryStore::new();
        store.append_outbox("alice", json!({"n": 1})).await.unwrap();
        store.append_outbox("alice", json!({"n": 2})).await.unwrap();
        assert_eq!(store.outbox("alice").await.unwrap().len(), 2);
        // A different actor's outbox is independent.
        assert!(store.outbox("bob").await.unwrap().is_empty());
    }
}
