use std::sync::Arc;
use std::time::Duration;

use super::{Job, JobProcessor};

/// How many times a failing job is retried before it is dropped (and logged).
const MAX_ATTEMPTS: u32 = 3;

/// A fixed pool of worker tasks draining the queue concurrently.
pub struct WorkerPool;

impl WorkerPool {
    /// Spawn `count` workers, each pulling from the shared receiver. Returns the
    /// `JoinHandle`s so the caller can await or abort them on shutdown.
    pub fn spawn(
        count: usize,
        receiver: async_channel::Receiver<Job>,
        processor: Arc<dyn JobProcessor>,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        (0..count.max(1))
            .map(|id| {
                let receiver = receiver.clone();
                let processor = Arc::clone(&processor);
                tokio::spawn(async move { worker_loop(id, receiver, processor).await })
            })
            .collect()
    }
}

async fn worker_loop(
    id: usize,
    receiver: async_channel::Receiver<Job>,
    processor: Arc<dyn JobProcessor>,
) {
    tracing::debug!(worker = id, "worker started");
    // `recv` returns Err only once the channel is closed *and* drained — our
    // clean-shutdown signal.
    while let Ok(job) = receiver.recv().await {
        process_with_retry(id, &processor, job).await;
    }
    tracing::debug!(worker = id, "worker stopped");
}

/// Retry transient failures with linear backoff. A production system needs a
/// durable dead-letter queue and idempotency keys; here we cap attempts and log,
/// which is enough to keep one unreachable peer from wedging a worker forever.
async fn process_with_retry(worker: usize, processor: &Arc<dyn JobProcessor>, job: Job) {
    for attempt in 1..=MAX_ATTEMPTS {
        match processor.process(job.clone()).await {
            Ok(()) => return,
            Err(err) if attempt < MAX_ATTEMPTS => {
                tracing::warn!(worker, attempt, error = ?err, "job failed; retrying");
                tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
            }
            Err(err) => {
                tracing::error!(worker, error = ?err, "job permanently failed; dropping");
            }
        }
    }
}
