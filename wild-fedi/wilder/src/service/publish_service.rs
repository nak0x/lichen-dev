use std::sync::Arc;

use serde_json::Value;

use crate::queue::{Job, JobQueue};
use crate::repository::{ActivityRepository, FollowerRepository};

/// The publish side of the server (README §4, contract 6): record an activity in
/// an actor's outbox and deliver it to every follower.
///
/// Delivery is *enqueued* (one `Deliver` job per follower), so `publish` returns
/// immediately and the fan-out happens on the worker pool — the same async shape
/// as inbound processing. This is the seam a future client-to-server outbox POST
/// endpoint would call; for now the seed data publishes through it.
pub struct PublishService {
    activities: Arc<dyn ActivityRepository>,
    followers: Arc<dyn FollowerRepository>,
    queue: Arc<dyn JobQueue>,
}

impl PublishService {
    pub fn new(
        activities: Arc<dyn ActivityRepository>,
        followers: Arc<dyn FollowerRepository>,
        queue: Arc<dyn JobQueue>,
    ) -> Self {
        Self {
            activities,
            followers,
            queue,
        }
    }

    pub async fn publish(&self, username: &str, activity: Value) {
        self.activities
            .append_outbox(username, activity.clone())
            .await;
        for follower in self.followers.list(username).await {
            self.queue
                .enqueue(Job::Deliver {
                    sender: username.to_string(),
                    inbox_url: follower.inbox,
                    activity: activity.clone(),
                })
                .await;
        }
    }
}
