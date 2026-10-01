use async_trait::async_trait;
use sqlx::Row;

use super::PgStore;
use crate::repository::{Follower, FollowerRepository};

#[async_trait]
impl FollowerRepository for PgStore {
    async fn add(&self, username: &str, follower: Follower) -> anyhow::Result<()> {
        // The (username, actor_id) primary key gives idempotency for free; a
        // re-sent Follow just refreshes the cached inbox.
        sqlx::query(
            "INSERT INTO followers (username, actor_id, inbox) VALUES ($1, $2, $3) \
             ON CONFLICT (username, actor_id) DO UPDATE SET inbox = EXCLUDED.inbox",
        )
        .bind(username)
        .bind(&follower.actor_id)
        .bind(&follower.inbox)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn remove(&self, username: &str, follower_actor_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM followers WHERE username = $1 AND actor_id = $2")
            .bind(username)
            .bind(follower_actor_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    async fn list(&self, username: &str) -> anyhow::Result<Vec<Follower>> {
        let rows = sqlx::query(
            "SELECT actor_id, inbox FROM followers WHERE username = $1 ORDER BY created_at",
        )
        .bind(username)
        .fetch_all(self.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Follower {
                actor_id: r.get("actor_id"),
                inbox: r.get("inbox"),
            })
            .collect())
    }
}
