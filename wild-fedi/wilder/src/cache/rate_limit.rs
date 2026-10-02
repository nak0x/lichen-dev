//! A fixed-window rate limiter in Redis: `INCR` a per-key counter that expires
//! after the window. Good enough to blunt credential brute-forcing without the
//! complexity of a sliding log. Counters are pure cache — losing them only
//! resets a window early.

use deadpool_redis::Pool;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct RateLimiter {
    pool: Pool,
}

impl RateLimiter {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Record one hit against `key` and report whether it is still within `max`
    /// hits per `window_secs`. On any Redis error we *allow* the request —
    /// a limiter outage must not lock everyone out (fail-open).
    pub async fn allow(&self, key: &str, max: u32, window_secs: u64) -> bool {
        match self.try_allow(key, max, window_secs).await {
            Ok(allowed) => allowed,
            Err(err) => {
                tracing::warn!(?err, "rate limiter unavailable; allowing");
                true
            }
        }
    }

    async fn try_allow(&self, key: &str, max: u32, window_secs: u64) -> anyhow::Result<bool> {
        let mut conn = self.pool.get().await?;
        let k = format!("rl:{key}");
        let count: u64 = conn.incr(&k, 1).await?;
        if count == 1 {
            // First hit in this window — set the expiry that defines the window.
            let _: bool = conn.expire(&k, window_secs as i64).await?;
        }
        Ok(count <= max as u64)
    }
}
