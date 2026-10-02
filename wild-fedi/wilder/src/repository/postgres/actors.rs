use async_trait::async_trait;
use sqlx::Row;

use super::{PgStore, kind_from_db, kind_to_db};
use crate::repository::{ActorRepository, LocalActor};

#[async_trait]
impl ActorRepository for PgStore {
    async fn get(&self, username: &str) -> anyhow::Result<Option<LocalActor>> {
        let row = sqlx::query(
            "SELECT username, kind, display_name, summary, private_key_pem, public_key_pem, \
                icon_url FROM actors WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.map(|r| LocalActor {
            username: r.get("username"),
            kind: kind_from_db(r.get::<String, _>("kind").as_str()),
            display_name: r.get("display_name"),
            summary: r.get("summary"),
            private_key_pem: r.get("private_key_pem"),
            public_key_pem: r.get("public_key_pem"),
            icon_url: r.get("icon_url"),
        }))
    }

    async fn insert(&self, actor: LocalActor) -> anyhow::Result<()> {
        // Upsert so a re-run of the seed (or a key rotation) is idempotent, matching
        // the in-memory store's map-insert-overwrites semantics.
        sqlx::query(
            "INSERT INTO actors \
                (username, kind, display_name, summary, private_key_pem, public_key_pem) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (username) DO UPDATE SET \
                kind = EXCLUDED.kind, \
                display_name = EXCLUDED.display_name, \
                summary = EXCLUDED.summary, \
                private_key_pem = EXCLUDED.private_key_pem, \
                public_key_pem = EXCLUDED.public_key_pem",
        )
        .bind(&actor.username)
        .bind(kind_to_db(actor.kind))
        .bind(&actor.display_name)
        .bind(&actor.summary)
        .bind(&actor.private_key_pem)
        .bind(&actor.public_key_pem)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn count(&self) -> anyhow::Result<usize> {
        let row = sqlx::query("SELECT count(*) AS n FROM actors")
            .fetch_one(self.pool())
            .await?;
        let n: i64 = row.get("n");
        Ok(n as usize)
    }

    async fn set_icon(&self, username: &str, icon_url: &str) -> anyhow::Result<()> {
        sqlx::query("UPDATE actors SET icon_url = $2 WHERE username = $1")
            .bind(username)
            .bind(icon_url)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    async fn update_bio(
        &self,
        username: &str,
        display_name: Option<&str>,
        summary: Option<&str>,
    ) -> anyhow::Result<()> {
        // COALESCE keeps the existing value when a field is not supplied.
        sqlx::query(
            "UPDATE actors SET display_name = COALESCE($2, display_name), \
                summary = COALESCE($3, summary) WHERE username = $1",
        )
        .bind(username)
        .bind(display_name)
        .bind(summary)
        .execute(self.pool())
        .await?;
        Ok(())
    }
}
