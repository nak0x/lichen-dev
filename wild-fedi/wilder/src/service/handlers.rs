use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::federation::FederationClient;
use crate::queue::{Job, JobQueue};
use crate::repository::{ActivityRepository, Follower, FollowerRepository};
use crate::vocab::{self, Activity, IncomingActivity};

/// Everything a handler needs to act, passed per-activity. It holds `Arc`s (not
/// borrows) so a handler's future is `'static` — required for the boxed,
/// dynamically-dispatched futures below.
#[derive(Clone)]
pub struct HandlerContext {
    pub config: Arc<AppConfig>,
    pub followers: Arc<dyn FollowerRepository>,
    pub activities: Arc<dyn ActivityRepository>,
    pub federation: Arc<FederationClient>,
    pub queue: Arc<dyn JobQueue>,
    pub recipient: String, // local actor whose inbox received the activity
    pub sender_id: String, // verified authoring actor IRI
}

/// Strategy interface: one implementation per activity verb. Adding a verb means
/// adding a handler and registering it — the dispatcher and the inbox service
/// stay untouched. That's the Open/Closed principle in practice.
#[async_trait]
pub trait ActivityHandler: Send + Sync {
    fn verb(&self) -> &'static str;
    /// `raw` is the untouched activity JSON; `activity` is the parsed view. Some
    /// handlers (e.g. Follow) need to echo the original back verbatim.
    async fn handle(
        &self,
        ctx: &HandlerContext,
        activity: &IncomingActivity,
        raw: &Value,
    ) -> anyhow::Result<()>;
}

/// Routes an activity to the handler registered for its `type`.
pub struct ActivityDispatcher {
    handlers: HashMap<&'static str, Box<dyn ActivityHandler>>,
}

impl ActivityDispatcher {
    pub fn with_defaults() -> Self {
        let mut dispatcher = Self {
            handlers: HashMap::new(),
        };
        dispatcher.register(Box::new(FollowHandler));
        dispatcher.register(Box::new(UndoHandler));
        dispatcher.register(Box::new(AcceptHandler));
        // Verbs we currently only record. Each is a seam to grow real behaviour
        // on — e.g. `Create` is where LICHEN would parse a Note into a
        // structured observation and index it.
        for verb in ["Create", "Update", "Delete", "Like", "Announce"] {
            dispatcher.register(Box::new(RecordingHandler { verb }));
        }
        dispatcher
    }

    pub fn register(&mut self, handler: Box<dyn ActivityHandler>) {
        self.handlers.insert(handler.verb(), handler);
    }

    pub async fn dispatch(
        &self,
        ctx: &HandlerContext,
        activity: &IncomingActivity,
        raw: &Value,
    ) -> anyhow::Result<()> {
        match self.handlers.get(activity.kind.as_str()) {
            Some(handler) => handler.handle(ctx, activity, raw).await,
            None => {
                // Unknown verbs are ignored, not errored: the fediverse is broad
                // and we must not fail loudly on activities we don't implement.
                tracing::info!(verb = %activity.kind, "no handler for activity verb; ignoring");
                Ok(())
            }
        }
    }
}

/// `Follow`: register the sender as a follower and acknowledge with `Accept`.
struct FollowHandler;

#[async_trait]
impl ActivityHandler for FollowHandler {
    fn verb(&self) -> &'static str {
        "Follow"
    }

    async fn handle(
        &self,
        ctx: &HandlerContext,
        _activity: &IncomingActivity,
        raw: &Value,
    ) -> anyhow::Result<()> {
        // Resolve the follower by fetching their actor document.
        let follower = ctx.federation.fetch_actor(&ctx.sender_id).await?;
        // A directed Accept always goes to the follower's *personal* inbox.
        let personal_inbox = follower.inbox.clone();
        // But for future fan-out of our own posts we store their sharedInbox
        // when advertised: one POST there reaches all their users on that
        // server, instead of one POST per follower.
        let delivery_inbox = follower
            .endpoints
            .and_then(|e| e.shared_inbox)
            .unwrap_or_else(|| personal_inbox.clone());
        ctx.followers
            .add(
                &ctx.recipient,
                Follower {
                    actor_id: ctx.sender_id.clone(),
                    inbox: delivery_inbox,
                },
            )
            .await;

        // Acknowledge with an Accept that echoes the original Follow as its
        // object (the shape Mastodon and friends expect), and enqueue delivery
        // so the outbound POST is a separate, independently-retryable job.
        let accept = Activity {
            context: vocab::activity_context(),
            kind: "Accept".to_string(),
            id: ctx.config.activity_url(&Uuid::new_v4().to_string()),
            actor: ctx.config.actor_url(&ctx.recipient),
            to: vec![ctx.sender_id.clone()],
            cc: vec![],
            published: None,
            object: raw.clone(),
        };
        ctx.queue
            .enqueue(Job::Deliver {
                sender: ctx.recipient.clone(),
                inbox_url: personal_inbox,
                activity: serde_json::to_value(&accept)?,
            })
            .await;
        tracing::info!(follower = %ctx.sender_id, actor = %ctx.recipient, "accepted Follow");
        Ok(())
    }
}

/// `Undo`: reverse a prior activity. We act on `Undo(Follow)` (unfollow); other
/// undos are logged and ignored.
struct UndoHandler;

#[async_trait]
impl ActivityHandler for UndoHandler {
    fn verb(&self) -> &'static str {
        "Undo"
    }

    async fn handle(
        &self,
        ctx: &HandlerContext,
        activity: &IncomingActivity,
        _raw: &Value,
    ) -> anyhow::Result<()> {
        let inner_type = activity
            .object
            .as_ref()
            .and_then(|o| o.get("type"))
            .and_then(Value::as_str);
        if inner_type == Some("Follow") {
            ctx.followers.remove(&ctx.recipient, &ctx.sender_id).await;
            tracing::info!(follower = %ctx.sender_id, "removed follower via Undo(Follow)");
        } else {
            tracing::info!(?inner_type, "Undo of unhandled activity; ignoring");
        }
        Ok(())
    }
}

/// `Accept`: a remote server accepted a Follow *we* sent. A fuller server would
/// flip that pending relationship to confirmed; we log it.
struct AcceptHandler;

#[async_trait]
impl ActivityHandler for AcceptHandler {
    fn verb(&self) -> &'static str {
        "Accept"
    }

    async fn handle(
        &self,
        ctx: &HandlerContext,
        _activity: &IncomingActivity,
        _raw: &Value,
    ) -> anyhow::Result<()> {
        tracing::info!(by = %ctx.sender_id, "a Follow we sent was accepted");
        Ok(())
    }
}

/// Records the activity in the recipient's inbox history without further action.
/// Shared by the verbs we accept but don't yet act on, so we don't duplicate the
/// same trivial body five times.
struct RecordingHandler {
    verb: &'static str,
}

#[async_trait]
impl ActivityHandler for RecordingHandler {
    fn verb(&self) -> &'static str {
        self.verb
    }

    async fn handle(
        &self,
        ctx: &HandlerContext,
        _activity: &IncomingActivity,
        raw: &Value,
    ) -> anyhow::Result<()> {
        ctx.activities
            .record_inbox(&ctx.recipient, raw.clone())
            .await;
        tracing::info!(verb = self.verb, recipient = %ctx.recipient, "recorded activity");
        Ok(())
    }
}
