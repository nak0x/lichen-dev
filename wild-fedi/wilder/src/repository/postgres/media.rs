use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use super::PgStore;
use crate::domain::MediaRef;
use crate::repository::MediaRepository;

#[async_trait]
impl MediaRepository for PgStore {
    async fn insert(&self, media: &MediaRef) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO media (id, owner_username, object_key, content_type, byte_size, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(media.id)
        .bind(&media.owner_username)
        .bind(&media.object_key)
        .bind(&media.content_type)
        .bind(media.byte_size)
        .bind(media.created_at)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn by_id(&self, id: Uuid) -> anyhow::Result<Option<MediaRef>> {
        let row = sqlx::query(
            "SELECT id, owner_username, object_key, content_type, byte_size, created_at \
             FROM media WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(|r| MediaRef {
            id: r.get("id"),
            owner_username: r.get("owner_username"),
            object_key: r.get("object_key"),
            content_type: r.get("content_type"),
            byte_size: r.get("byte_size"),
            created_at: r.get::<DateTime<Utc>, _>("created_at"),
        }))
    }
}
