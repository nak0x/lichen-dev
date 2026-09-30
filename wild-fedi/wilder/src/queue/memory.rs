use async_trait::async_trait;

use super::{Job, JobQueue};

/// An MPMC channel-backed queue.
///
/// `async-channel` (unlike tokio's single-consumer MPSC) lets many worker tasks
/// share one receiver, which is exactly the consumer pool we want. The queue is
/// **bounded**, so a flood of inbound traffic applies backpressure (`enqueue`
/// awaits) instead of growing memory without limit.
#[derive(Clone)]
pub struct InMemoryQueue {
    sender: async_channel::Sender<Job>,
}

impl InMemoryQueue {
    /// Create a bounded queue and hand back the receiver for the worker pool to
    /// clone across its workers.
    pub fn bounded(capacity: usize) -> (Self, async_channel::Receiver<Job>) {
        let (sender, receiver) = async_channel::bounded(capacity.max(1));
        (Self { sender }, receiver)
    }
}

#[async_trait]
impl JobQueue for InMemoryQueue {
    async fn enqueue(&self, job: Job) {
        // The channel is closed only if every receiver was dropped, which only
        // happens during shutdown — so we log rather than panic.
        if let Err(err) = self.sender.send(job).await {
            tracing::error!(?err, "failed to enqueue job: queue closed");
        }
    }
}
