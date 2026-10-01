//! A durable job queue backed by a **Redis Stream**, replacing the in-process
//! `async-channel` for production. Producers `XADD`; a pool of consumers in a
//! single consumer group `XREADGROUP` (blocking) and `XACK`. Because the stream
//! persists (Redis AOF), jobs survive a restart, and consumers can live in
//! another process — the properties an in-memory channel cannot give us.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use deadpool_redis::{Config, Pool, PoolConfig, Runtime};
use redis::AsyncCommands;
use redis::streams::{StreamMaxlen, StreamReadOptions, StreamReadReply};

use super::worker::process_with_retry;
use super::{Job, JobProcessor, JobQueue};

/// How long to wait between polls when the stream has no new jobs. We poll
/// rather than issue a long-blocking `XREADGROUP BLOCK`, because blocking
/// commands misbehave over the pooled *multiplexed* connections deadpool hands
/// out (the multiplexer expects prompt, ordered replies). Polling is robust and,
/// for low-volume federation delivery, a sub-second latency is irrelevant.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// How many pending entries to claim per read.
const BATCH: usize = 10;

/// Build a pooled Redis client. `max_size` must comfortably exceed the worker
/// count: each worker holds one connection parked in a blocking `XREADGROUP`,
/// and the rest of the app (sessions, feeds, caches) borrows from the same pool.
pub fn build_pool(url: &str, max_size: usize) -> anyhow::Result<Pool> {
    let mut cfg = Config::from_url(url);
    cfg.pool = Some(PoolConfig::new(max_size));
    cfg.create_pool(Some(Runtime::Tokio1))
        .context("creating redis pool")
}

/// Producer side: serialise a `Job` and append it to the stream.
pub struct RedisQueue {
    pool: Pool,
    stream: String,
    /// Approximate cap on stream length, so a backlog cannot grow without bound
    /// (XADD MAXLEN ~). Sized from `queue_capacity`.
    maxlen: usize,
}

impl RedisQueue {
    pub fn new(pool: Pool, stream: impl Into<String>, maxlen: usize) -> Self {
        Self {
            pool,
            stream: stream.into(),
            maxlen: maxlen.max(1),
        }
    }
}

#[async_trait]
impl JobQueue for RedisQueue {
    async fn enqueue(&self, job: Job) {
        // The trait is fire-and-forget (producers don't await completion), so we
        // log failures rather than propagate. A Redis outage here means the whole
        // instance is already degraded; Phase 7 may make this fallible + 503.
        let payload = match serde_json::to_string(&job) {
            Ok(p) => p,
            Err(err) => {
                tracing::error!(?err, "failed to serialise job");
                return;
            }
        };
        let result = async {
            let mut conn = self.pool.get().await?;
            let _: String = conn
                .xadd_maxlen(
                    &self.stream,
                    StreamMaxlen::Approx(self.maxlen),
                    "*",
                    &[("job", payload.as_str())],
                )
                .await?;
            anyhow::Ok(())
        }
        .await;
        if let Err(err) = result {
            tracing::error!(?err, "failed to enqueue job to redis");
        }
    }
}

/// Consumer side: a fixed pool of workers sharing one consumer group.
pub struct RedisWorkerPool;

impl RedisWorkerPool {
    pub async fn spawn(
        count: usize,
        pool: Pool,
        stream: impl Into<String>,
        group: impl Into<String>,
        processor: Arc<dyn JobProcessor>,
    ) -> anyhow::Result<Vec<tokio::task::JoinHandle<()>>> {
        let stream = stream.into();
        let group = group.into();

        // Create the group (and the stream, via MKSTREAM) if absent. A repeat
        // start returns BUSYGROUP, which is the "already exists" success case.
        {
            let mut conn = pool.get().await.context("redis connection for group setup")?;
            let created: redis::RedisResult<String> =
                conn.xgroup_create_mkstream(&stream, &group, "$").await;
            // BUSYGROUP just means the group already exists — the success case on
            // a restart. Anything else is a real setup failure.
            if let Err(err) = created
                && !err.to_string().contains("BUSYGROUP")
            {
                return Err(anyhow::anyhow!("creating consumer group: {err}"));
            }
        }

        let handles = (0..count.max(1))
            .map(|id| {
                let pool = pool.clone();
                let stream = stream.clone();
                let group = group.clone();
                let processor = Arc::clone(&processor);
                let consumer = format!("worker-{id}");
                tokio::spawn(async move {
                    consume_loop(id, pool, stream, group, consumer, processor).await
                })
            })
            .collect();
        tracing::info!(workers = count, %stream, "redis worker pool started");
        Ok(handles)
    }
}

async fn consume_loop(
    worker: usize,
    pool: Pool,
    stream: String,
    group: String,
    consumer: String,
    processor: Arc<dyn JobProcessor>,
) {
    // Non-blocking read: returns immediately with whatever is available.
    let opts = StreamReadOptions::default()
        .group(&group, &consumer)
        .count(BATCH);

    loop {
        let mut conn = match pool.get().await {
            Ok(conn) => conn,
            Err(err) => {
                tracing::error!(worker, ?err, "redis pool get failed; backing off");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        // `>` = "messages never delivered to any consumer in this group". When
        // the stream is empty the server replies nil, which the client surfaces
        // as an empty reply (Ok) or a Nil-typed error depending on version — both
        // simply mean "nothing to do", so we poll again after a short pause.
        let reply: redis::RedisResult<StreamReadReply> =
            conn.xread_options(&[&stream], &[">"], &opts).await;
        let entries = match reply {
            Ok(reply) => reply.keys.into_iter().flat_map(|k| k.ids).collect::<Vec<_>>(),
            // A nil reply (no new messages) surfaces as a type/parse error on some
            // client versions; both mean "nothing to do".
            Err(err)
                if matches!(
                    err.kind(),
                    redis::ErrorKind::UnexpectedReturnType | redis::ErrorKind::Parse
                ) =>
            {
                Vec::new()
            }
            Err(err) => {
                tracing::error!(worker, ?err, "xreadgroup failed; backing off");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        if entries.is_empty() {
            tokio::time::sleep(POLL_INTERVAL).await;
            continue;
        }

        for entry in entries {
            let id = entry.id.clone();
            match entry.get::<String>("job") {
                Some(payload) => match serde_json::from_str::<Job>(&payload) {
                    Ok(job) => process_with_retry(worker, &processor, job).await,
                    Err(err) => {
                        tracing::error!(worker, %id, ?err, "undeserialisable job; dropping")
                    }
                },
                None => tracing::warn!(worker, %id, "stream entry missing 'job' field"),
            }
            // ACK unconditionally: retries are already exhausted inside
            // `process_with_retry`, and a poison entry must not stay pending
            // forever. A dead-letter stream would be the next refinement.
            let _: redis::RedisResult<i64> = conn.xack(&stream, &group, &[&id]).await;
        }
    }
}
