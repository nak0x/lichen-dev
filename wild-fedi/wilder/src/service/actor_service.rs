use std::sync::Arc;

use serde_json::{Value, json};

use crate::config::AppConfig;
use crate::repository::{ActivityRepository, ActorRepository, FollowerRepository, LocalActor};
use crate::vocab::{self, Actor, Endpoints, OrderedCollection, PublicKey};

/// Assembles the read-side ActivityPub documents (actor, outbox, followers,
/// webfinger, nodeinfo) from stored state and config.
///
/// Pure translation: no signing, no outbound I/O, only repository reads. That
/// makes it trivial to unit-test with a stub repository.
pub struct ActorService {
    config: Arc<AppConfig>,
    actors: Arc<dyn ActorRepository>,
    followers: Arc<dyn FollowerRepository>,
    activities: Arc<dyn ActivityRepository>,
}

impl ActorService {
    pub fn new(
        config: Arc<AppConfig>,
        actors: Arc<dyn ActorRepository>,
        followers: Arc<dyn FollowerRepository>,
        activities: Arc<dyn ActivityRepository>,
    ) -> Self {
        Self {
            config,
            actors,
            followers,
            activities,
        }
    }

    pub async fn find(&self, username: &str) -> Option<LocalActor> {
        self.actors.get(username).await
    }

    /// Build the actor document for `GET /users/:name`.
    pub fn actor_document(&self, actor: &LocalActor) -> Actor {
        let id = self.config.actor_url(&actor.username);
        Actor {
            context: vocab::actor_context(),
            id: id.clone(),
            kind: actor.kind,
            preferred_username: actor.username.clone(),
            name: actor.display_name.clone(),
            summary: actor.summary.clone(),
            inbox: self.config.inbox_url(&actor.username),
            outbox: self.config.outbox_url(&actor.username),
            followers: self.config.followers_url(&actor.username),
            following: None,
            public_key: PublicKey {
                id: self.config.key_id(&actor.username),
                owner: id,
                public_key_pem: actor.public_key_pem.clone(),
            },
            endpoints: Some(Endpoints {
                shared_inbox: self.config.shared_inbox_url(),
            }),
        }
    }

    pub async fn outbox_collection(&self, username: &str) -> OrderedCollection {
        let items = self.activities.outbox(username).await;
        OrderedCollection::new(
            vocab::activity_context(),
            self.config.outbox_url(username),
            items,
        )
    }

    pub async fn followers_collection(&self, username: &str) -> OrderedCollection {
        // We expose follower IRIs, not the actors' full documents — that's the
        // conventional (and privacy-preserving) shape for this collection.
        let items = self
            .followers
            .list(username)
            .await
            .into_iter()
            .map(|f| Value::String(f.actor_id))
            .collect();
        OrderedCollection::new(
            vocab::activity_context(),
            self.config.followers_url(username),
            items,
        )
    }

    /// Resolve a WebFinger `acct:` resource to a JRD document (README §2).
    /// Returns `None` (→ 404) for accounts we don't host or the wrong domain.
    pub async fn webfinger(&self, resource: &str) -> Option<Value> {
        // Expect `acct:username@domain`; the domain must be ours.
        let acct = resource.strip_prefix("acct:").unwrap_or(resource);
        let (username, domain) = acct.split_once('@')?;
        if domain != self.config.domain {
            return None;
        }
        let actor = self.actors.get(username).await?;
        Some(json!({
            "subject": format!("acct:{}@{}", actor.username, self.config.domain),
            "links": [{
                "rel": "self",
                "type": "application/activity+json",
                "href": self.config.actor_url(&actor.username),
            }],
        }))
    }

    /// The NodeInfo 2.1 document other servers read to discover what we run.
    pub async fn nodeinfo(&self) -> Value {
        json!({
            "version": "2.1",
            "software": {
                "name": self.config.software_name,
                "version": self.config.software_version,
            },
            "protocols": ["activitypub"],
            "services": { "inbound": [], "outbound": [] },
            "openRegistrations": false,
            "usage": { "users": { "total": self.actors.count().await } },
            "metadata": {
                "nodeName": "wilder",
                "purpose": "LICHEN federated nature observations",
            },
        })
    }
}
