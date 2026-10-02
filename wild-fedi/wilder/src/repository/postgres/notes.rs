use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::postgres::PgRow;
use uuid::Uuid;

use super::PgStore;
use crate::domain::{Note, NoteKind};
use crate::repository::NoteRepository;

fn note_from_row(r: &PgRow) -> anyhow::Result<Note> {
    let kind_str: String = r.get("kind");
    Ok(Note {
        id: r.get("id"),
        owner_username: r.get("owner_username"),
        kind: NoteKind::from_db(&kind_str)
            .ok_or_else(|| anyhow::anyhow!("bad note kind: {kind_str}"))?,
        title: r.get("title"),
        content: r.get("content"),
        created_at: r.get::<DateTime<Utc>, _>("created_at"),
        updated_at: r.get::<DateTime<Utc>, _>("updated_at"),
    })
}

#[async_trait]
impl NoteRepository for PgStore {
    async fn list(&self, owner: &str, kind: NoteKind) -> anyhow::Result<Vec<Note>> {
        let rows = sqlx::query(
            "SELECT id, owner_username, kind, title, content, created_at, updated_at \
             FROM notes WHERE owner_username = $1 AND kind = $2 ORDER BY updated_at DESC",
        )
        .bind(owner)
        .bind(kind.as_db())
        .fetch_all(self.pool())
        .await?;
        rows.iter().map(note_from_row).collect()
    }

    async fn by_id(&self, id: Uuid) -> anyhow::Result<Option<Note>> {
        let row = sqlx::query(
            "SELECT id, owner_username, kind, title, content, created_at, updated_at \
             FROM notes WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;
        row.as_ref().map(note_from_row).transpose()
    }

    async fn insert(&self, note: &Note) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO notes (id, owner_username, kind, title, content, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(note.id)
        .bind(&note.owner_username)
        .bind(note.kind.as_db())
        .bind(&note.title)
        .bind(&note.content)
        .bind(note.created_at)
        .bind(note.updated_at)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn update(&self, id: Uuid, title: Option<&str>, content: &str) -> anyhow::Result<()> {
        sqlx::query("UPDATE notes SET title = $2, content = $3, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(title)
            .bind(content)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM notes WHERE id = $1")
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(())
    }
}
