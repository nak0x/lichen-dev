use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgRow;
use sqlx::types::Json;

use super::PgStore;
use crate::domain::{Announcement, AnnouncementKind};
use crate::repository::AnnouncementRepository;

fn from_row(r: &PgRow) -> anyhow::Result<Announcement> {
    let kind_str: String = r.get("kind");
    Ok(Announcement {
        id: r.get("id"),
        kind: AnnouncementKind::from_db(&kind_str)
            .ok_or_else(|| anyhow::anyhow!("bad announcement kind: {kind_str}"))?,
        author_username: r.get("author_username"),
        title: r.get("title"),
        content: r.get("content"),
        start_time: r.get::<Option<DateTime<Utc>>, _>("start_time"),
        location: r.get("location"),
        object_id: r.get("object_id"),
        activity: r.get::<Json<Value>, _>("activity").0,
        published: r.get::<DateTime<Utc>, _>("published"),
    })
}

#[async_trait]
impl AnnouncementRepository for PgStore {
    async fn insert(&self, a: &Announcement) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO announcements \
                (id, kind, author_username, title, content, start_time, location, object_id, \
                 activity, published) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(a.id)
        .bind(a.kind.as_db())
        .bind(&a.author_username)
        .bind(&a.title)
        .bind(&a.content)
        .bind(a.start_time)
        .bind(&a.location)
        .bind(&a.object_id)
        .bind(Json(&a.activity))
        .bind(a.published)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn list(&self, kind: AnnouncementKind, limit: i64) -> anyhow::Result<Vec<Announcement>> {
        let rows = sqlx::query(
            "SELECT id, kind, author_username, title, content, start_time, location, object_id, \
                activity, published \
             FROM announcements WHERE kind = $1 ORDER BY published DESC LIMIT $2",
        )
        .bind(kind.as_db())
        .bind(limit)
        .fetch_all(self.pool())
        .await?;
        rows.iter().map(from_row).collect()
    }

    async fn count(&self, kind: AnnouncementKind) -> anyhow::Result<usize> {
        let row = sqlx::query("SELECT count(*) AS n FROM announcements WHERE kind = $1")
            .bind(kind.as_db())
            .fetch_one(self.pool())
            .await?;
        Ok(row.get::<i64, _>("n") as usize)
    }
}
