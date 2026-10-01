//! Per-user "last visit" timestamps in Redis. This is the one piece of feed
//! state we keep hot: it is written on every visit (high churn) and losing it
//! merely shows a slightly longer feed — pure cache, never a source of truth.
//!
//! The feed itself is computed at read time from Postgres (posts in the user's
//! groups since this timestamp). Fan-out-on-write into Redis lists would only
//! matter at a scale this network will not reach; read-time keeps it simple and
//! always-consistent.

use chrono::{DateTime, Utc};
use deadpool_redis::Pool;
use redis::AsyncCommands;

fn key(username: &str) -> String {
    format!("lastvisit:{username}")
}

#[derive(Clone)]
pub struct LastVisitStore {
    pool: Pool,
}

impl LastVisitStore {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// The instant the user last loaded their feed, if we have it. Stored in
    /// milliseconds: second-granularity would let a post published in the same
    /// second as the visit leak into the next batch.
    pub async fn get(&self, username: &str) -> anyhow::Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.get().await?;
        let millis: Option<i64> = conn.get(key(username)).await?;
        Ok(millis.and_then(DateTime::from_timestamp_millis))
    }

    /// Record a new visit time (milliseconds since the epoch).
    pub async fn set(&self, username: &str, when: DateTime<Utc>) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;
        let _: () = conn.set(key(username), when.timestamp_millis()).await?;
        Ok(())
    }
}
