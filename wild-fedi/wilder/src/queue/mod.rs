//! The work queue that decouples *accepting* a request from *processing* it.
//!
//! The inbox endpoint does the minimum synchronous work — verify the signature —
//! then enqueues a `Job` and returns `202 Accepted`. A pool of workers drains
//! the queue. This is the standard ActivityPub shape: it bounds request latency,
//! smooths bursts, and stops one slow remote peer from blocking the HTTP path.

pub mod memory;
pub mod worker;

pub use memory::InMemoryQueue;
pub use worker::WorkerPool;

use async_trait::async_trait;
use serde_json::Value;

/// A unit of deferred work. Inbox jobs carry *already-verified* data, so workers
/// never re-check trust. Delivery is one job per recipient inbox, so a single
/// unreachable peer can't hold up the others.
#[derive(Debug, Clone)]
pub enum Job {
    /// Handle a verified inbound activity against a local actor's inbox.
    ProcessInbox {
        recipient: String, // local username whose inbox received it
        sender_id: String, // verified authoring actor IRI
        activity: Value,   // the raw activity JSON
    },
    /// Sign and POST an activity to one remote inbox.
    Deliver {
        sender: String,    // local username whose key signs the delivery
        inbox_url: String, // absolute remote inbox URL
        activity: Value,   // the activity JSON to POST
    },
}

/// The enqueue side. Producers (handlers, services) depend on this abstraction,
/// not on a concrete channel, so the transport (in-memory now, Redis/NATS later)
/// stays an implementation detail.
#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue(&self, job: Job);
}

/// The processing side: whatever actually executes a job. Separating this from
/// the worker loop keeps the loop's concerns (concurrency, retries, backoff)
/// independent of the business logic (how to handle an activity / deliver JSON).
#[async_trait]
pub trait JobProcessor: Send + Sync {
    async fn process(&self, job: Job) -> anyhow::Result<()>;
}
