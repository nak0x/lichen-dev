//! Postgres-backed implementations of the repository traits. One `PgStore` holds
//! the connection pool and implements every trait (split across sibling files by
//! aggregate), mirroring how `InMemoryStore` implements them all — the
//! composition root picks which one to wire in.

mod accounts;
mod activities;
mod actors;
mod followers;
mod groups;
mod media;
mod posts;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::vocab::ActorType;

#[derive(Clone)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    /// Open a pooled connection. `max_connections` is deliberately modest: this
    /// is a small-instance server (README's Raspberry-Pi ethos), not a fleet.
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    /// Apply pending migrations. `migrate!` embeds the `./migrations` directory at
    /// compile time, so this needs no live database to *build* — only to *run*.
    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Exposed so later application-domain repositories can share the one pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Map our `ActorType` enum to the compact text stored in the `actors.kind`
/// column. Kept here (not in `vocab`) because it is a persistence concern.
pub(crate) fn kind_to_db(kind: ActorType) -> &'static str {
    match kind {
        ActorType::Person => "Person",
        ActorType::Group => "Group",
        ActorType::Organization => "Organization",
        ActorType::Service => "Service",
        ActorType::Application => "Application",
    }
}

pub(crate) fn kind_from_db(s: &str) -> ActorType {
    match s {
        "Group" => ActorType::Group,
        "Organization" => ActorType::Organization,
        "Service" => ActorType::Service,
        "Application" => ActorType::Application,
        _ => ActorType::Person, // default; unknown strings degrade to Person
    }
}
