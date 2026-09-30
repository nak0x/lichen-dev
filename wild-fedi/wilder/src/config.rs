use std::net::SocketAddr;

/// Runtime configuration, sourced from the environment with dev-friendly
/// defaults.
///
/// `domain` is the single most important value: in the fediverse *identity is
/// the domain* (README §6), so every URL we mint — actor ids, inboxes, key ids —
/// is derived from it. It must be exactly the host (and port) that peers reach
/// us on, or signatures and id-matching will not line up.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub scheme: String, // "http" for local dev, "https" in production
    pub domain: String, // authority, e.g. "school-a.org" or "127.0.0.1:8080"
    pub bind_addr: SocketAddr,
    pub worker_count: usize,
    pub queue_capacity: usize,
    pub software_name: String,
    pub software_version: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let scheme = std::env::var("WILDER_SCHEME").unwrap_or_else(|_| "http".to_string());
        let domain =
            std::env::var("WILDER_DOMAIN").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        let bind_addr = std::env::var("WILDER_BIND")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
            .parse()?;
        Ok(Self {
            scheme,
            domain,
            bind_addr,
            worker_count: parse_env("WILDER_WORKERS", 4)?,
            queue_capacity: parse_env("WILDER_QUEUE_CAPACITY", 1024)?,
            software_name: "wilder".to_string(),
            software_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    pub fn base_url(&self) -> String {
        format!("{}://{}", self.scheme, self.domain)
    }
    pub fn actor_url(&self, name: &str) -> String {
        format!("{}/users/{}", self.base_url(), name)
    }
    pub fn inbox_url(&self, name: &str) -> String {
        format!("{}/inbox", self.actor_url(name))
    }
    pub fn outbox_url(&self, name: &str) -> String {
        format!("{}/outbox", self.actor_url(name))
    }
    pub fn followers_url(&self, name: &str) -> String {
        format!("{}/followers", self.actor_url(name))
    }
    pub fn shared_inbox_url(&self) -> String {
        format!("{}/inbox", self.base_url())
    }
    /// The `keyId` we publish and sign with: `<actor>#main-key`. The fragment is
    /// how a peer selects which of an actor's keys signed a request.
    pub fn key_id(&self, name: &str) -> String {
        format!("{}#main-key", self.actor_url(name))
    }
    pub fn activity_url(&self, id: &str) -> String {
        format!("{}/activities/{}", self.base_url(), id)
    }
    pub fn object_url(&self, id: &str) -> String {
        format!("{}/objects/{}", self.base_url(), id)
    }
}

fn parse_env<T>(key: &str, default: T) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match std::env::var(key) {
        Ok(v) => v
            .parse()
            .map_err(|e: T::Err| anyhow::anyhow!("invalid {key}: {e}")),
        Err(_) => Ok(default),
    }
}
