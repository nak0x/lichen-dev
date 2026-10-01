//! Server-side sessions in Redis. The cookie carries only an opaque, random
//! token; all authority lives in the `session:<token>` key here. That makes
//! logout and forced revocation trivial (delete the key) and leaks nothing if a
//! cookie is stolen without the matching Redis entry — the reason we chose
//! server sessions over stateless JWTs.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use deadpool_redis::Pool;
use rand::RngCore;
use redis::AsyncCommands;
use uuid::Uuid;

/// Default session lifetime: one week. Every lookup refreshes it (sliding
/// expiry), so an active user is not logged out mid-session.
const DEFAULT_TTL_SECS: u64 = 7 * 24 * 60 * 60;

/// 256 bits of entropy, URL-safe base64 — unguessable and cookie-safe.
fn new_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn key(token: &str) -> String {
    format!("session:{token}")
}

#[derive(Clone)]
pub struct SessionStore {
    pool: Pool,
    ttl_secs: u64,
}

impl SessionStore {
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            ttl_secs: DEFAULT_TTL_SECS,
        }
    }

    /// Issue a new session for an account and return its token.
    pub async fn create(&self, account_id: Uuid) -> anyhow::Result<String> {
        let token = new_token();
        let mut conn = self.pool.get().await?;
        let _: () = conn
            .set_ex(key(&token), account_id.to_string(), self.ttl_secs)
            .await?;
        Ok(token)
    }

    /// Resolve a token to its account id, refreshing the TTL on success so
    /// active sessions slide forward. Returns `None` for an unknown/expired token.
    pub async fn lookup(&self, token: &str) -> anyhow::Result<Option<Uuid>> {
        let mut conn = self.pool.get().await?;
        let stored: Option<String> = conn.get(key(token)).await?;
        match stored {
            Some(raw) => {
                let _: bool = conn.expire(key(token), self.ttl_secs as i64).await?;
                Ok(Some(Uuid::parse_str(&raw)?))
            }
            None => Ok(None),
        }
    }

    /// Drop a session (logout). Deleting a missing key is a no-op.
    pub async fn revoke(&self, token: &str) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;
        let _: () = conn.del(key(token)).await?;
        Ok(())
    }
}
