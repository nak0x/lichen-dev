use std::sync::Arc;

use anyhow::{Context, anyhow};
use async_trait::async_trait;

use crate::config::AppConfig;
use crate::federation::FederationClient;
use crate::queue::{Job, JobProcessor};
use crate::repository::ActorRepository;
use crate::service::InboxService;

/// Wires each `Job` variant to the service that executes it. This is the adapter
/// between "what work exists" (the queue) and "how work is done" (the services),
/// which is why the worker pool can stay ignorant of both.
pub struct AppJobProcessor {
    config: Arc<AppConfig>,
    inbox: Arc<InboxService>,
    federation: Arc<FederationClient>,
    actors: Arc<dyn ActorRepository>,
}

impl AppJobProcessor {
    pub fn new(
        config: Arc<AppConfig>,
        inbox: Arc<InboxService>,
        federation: Arc<FederationClient>,
        actors: Arc<dyn ActorRepository>,
    ) -> Self {
        Self {
            config,
            inbox,
            federation,
            actors,
        }
    }
}

#[async_trait]
impl JobProcessor for AppJobProcessor {
    async fn process(&self, job: Job) -> anyhow::Result<()> {
        match job {
            Job::ProcessInbox {
                recipient,
                sender_id,
                activity,
            } => self.inbox.process(&recipient, &sender_id, activity).await,

            Job::Deliver {
                sender,
                inbox_url,
                activity,
            } => {
                // Look up the signing actor at delivery time — its key must be
                // current, and the actor may have been created after enqueue.
                let local = self
                    .actors
                    .get(&sender)
                    .await
                    .ok_or_else(|| anyhow!("delivery sender {sender} no longer exists"))?;
                let key_id = self.config.key_id(&sender);
                self.federation
                    .deliver(&local, &inbox_url, &activity, &key_id)
                    .await
                    .with_context(|| format!("delivering to {inbox_url}"))
            }
        }
    }
}
