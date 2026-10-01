use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use sqlx::types::Json;

use super::PgStore;
use crate::repository::ActivityRepository;

/// Lift the `published` timestamp out of an activity (or its embedded object) so
/// rows can be ordered chronologically and queried by "since" for feeds.
fn published_of(activity: &Value) -> Option<DateTime<Utc>> {
    let raw = activity
        .get("published")
        .or_else(|| activity.get("object").and_then(|o| o.get("published")))
        .and_then(Value::as_str)?;
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

impl PgStore {
    async fn append(&self, username: &str, mailbox: &str, activity: Value) -> anyhow::Result<()> {
        let published = published_of(&activity);
        sqlx::query(
            "INSERT INTO activities (username, box, activity, published) VALUES ($1, $2, $3, $4)",
        )
        .bind(username)
        .bind(mailbox)
        .bind(Json(activity))
        .bind(published)
        .execute(self.pool())
        .await?;
        Ok(())
    }
}

#[async_trait]
impl ActivityRepository for PgStore {
    async fn append_outbox(&self, username: &str, activity: Value) -> anyhow::Result<()> {
        self.append(username, "outbox", activity).await
    }

    async fn outbox(&self, username: &str) -> anyhow::Result<Vec<Value>> {
        // Newest first: the conventional shape for an outbox OrderedCollection.
        let rows = sqlx::query(
            "SELECT activity FROM activities WHERE username = $1 AND box = 'outbox' ORDER BY id DESC",
        )
        .bind(username)
        .fetch_all(self.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| r.get::<Json<Value>, _>("activity").0)
            .collect())
    }

    async fn record_inbox(&self, username: &str, activity: Value) -> anyhow::Result<()> {
        self.append(username, "inbox", activity).await
    }
}
