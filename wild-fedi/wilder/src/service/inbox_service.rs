use std::sync::Arc;

use serde_json::Value;

use crate::config::AppConfig;
use crate::federation::FederationClient;
use crate::queue::JobQueue;
use crate::repository::{ActivityRepository, FollowerRepository};
use crate::service::handlers::{ActivityDispatcher, HandlerContext};
use crate::vocab::IncomingActivity;

/// The write-side application service. It owns the dispatcher and turns a raw,
/// *already-verified* inbox payload into domain effects.
///
/// It is invoked only from workers, never directly from the HTTP handler — the
/// handler's job is to verify and enqueue, keeping request latency bounded and
/// off the network-bound work this triggers (fetching actors, delivering an
/// Accept, writing to storage).
pub struct InboxService {
    config: Arc<AppConfig>,
    followers: Arc<dyn FollowerRepository>,
    activities: Arc<dyn ActivityRepository>,
    federation: Arc<FederationClient>,
    queue: Arc<dyn JobQueue>,
    dispatcher: ActivityDispatcher,
}

impl InboxService {
    pub fn new(
        config: Arc<AppConfig>,
        followers: Arc<dyn FollowerRepository>,
        activities: Arc<dyn ActivityRepository>,
        federation: Arc<FederationClient>,
        queue: Arc<dyn JobQueue>,
    ) -> Self {
        Self {
            config,
            followers,
            activities,
            federation,
            queue,
            dispatcher: ActivityDispatcher::with_defaults(),
        }
    }

    pub async fn process(
        &self,
        recipient: &str,
        sender_id: &str,
        raw: Value,
    ) -> anyhow::Result<()> {
        let activity: IncomingActivity = serde_json::from_value(raw.clone())?;
        tracing::debug!(
            activity_id = ?activity.id,
            verb = %activity.kind,
            recipient = %recipient,
            "processing verified inbox activity"
        );
        let ctx = HandlerContext {
            config: Arc::clone(&self.config),
            followers: Arc::clone(&self.followers),
            activities: Arc::clone(&self.activities),
            federation: Arc::clone(&self.federation),
            queue: Arc::clone(&self.queue),
            recipient: recipient.to_string(),
            sender_id: sender_id.to_string(),
        };
        self.dispatcher.dispatch(&ctx, &activity, &raw).await
    }
}
