use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgRow;
use sqlx::types::Json;

use super::PgStore;
use crate::domain::Post;
use crate::repository::PostRepository;

fn post_from_row(r: &PgRow) -> Post {
    Post {
        id: r.get("id"),
        author_username: r.get("author_username"),
        group_username: r.get("group_username"),
        content: r.get("content"),
        tags: r.get::<Vec<String>, _>("tags"),
        object_id: r.get("object_id"),
        activity: r.get::<Json<Value>, _>("activity").0,
        published: r.get::<DateTime<Utc>, _>("published"),
    }
}

#[async_trait]
impl PostRepository for PgStore {
    async fn insert(&self, post: &Post) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO posts \
                (id, author_username, group_username, content, tags, object_id, activity, published) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(post.id)
        .bind(&post.author_username)
        .bind(&post.group_username)
        .bind(&post.content)
        .bind(&post.tags)
        .bind(&post.object_id)
        .bind(Json(&post.activity))
        .bind(post.published)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn in_group(
        &self,
        group_username: &str,
        tag: Option<&str>,
        limit: i64,
    ) -> anyhow::Result<Vec<Post>> {
        // Two literal queries (sqlx 0.9 wants &'static str): with or without the
        // tag filter. `$2 = ANY(tags)` hits the GIN index.
        let rows = match tag {
            Some(t) => {
                sqlx::query(
                    "SELECT id, author_username, group_username, content, tags, object_id, \
                        activity, published FROM posts \
                     WHERE group_username = $1 AND $2 = ANY(tags) \
                     ORDER BY published DESC LIMIT $3",
                )
                .bind(group_username)
                .bind(t)
                .bind(limit)
                .fetch_all(self.pool())
                .await?
            }
            None => {
                sqlx::query(
                    "SELECT id, author_username, group_username, content, tags, object_id, \
                        activity, published FROM posts \
                     WHERE group_username = $1 ORDER BY published DESC LIMIT $2",
                )
                .bind(group_username)
                .bind(limit)
                .fetch_all(self.pool())
                .await?
            }
        };
        Ok(rows.iter().map(post_from_row).collect())
    }

    async fn in_groups_since(
        &self,
        group_usernames: &[String],
        since: DateTime<Utc>,
        limit: i64,
    ) -> anyhow::Result<Vec<Post>> {
        let rows = sqlx::query(
            "SELECT id, author_username, group_username, content, tags, object_id, \
                activity, published FROM posts \
             WHERE group_username = ANY($1) AND published > $2 \
             ORDER BY published DESC LIMIT $3",
        )
        .bind(group_usernames)
        .bind(since)
        .bind(limit)
        .fetch_all(self.pool())
        .await?;
        Ok(rows.iter().map(post_from_row).collect())
    }

    async fn captured(
        &self,
        group_username: Option<&str>,
        limit: i64,
    ) -> anyhow::Result<Vec<Post>> {
        let rows = match group_username {
            Some(g) => {
                sqlx::query(
                    "SELECT p.id, p.author_username, p.group_username, p.content, p.tags, \
                        p.object_id, p.activity, p.published \
                     FROM posts p JOIN actors a ON a.username = p.author_username \
                     WHERE a.kind = 'Service' AND p.group_username = $1 \
                     ORDER BY p.published DESC LIMIT $2",
                )
                .bind(g)
                .bind(limit)
                .fetch_all(self.pool())
                .await?
            }
            None => {
                sqlx::query(
                    "SELECT p.id, p.author_username, p.group_username, p.content, p.tags, \
                        p.object_id, p.activity, p.published \
                     FROM posts p JOIN actors a ON a.username = p.author_username \
                     WHERE a.kind = 'Service' ORDER BY p.published DESC LIMIT $1",
                )
                .bind(limit)
                .fetch_all(self.pool())
                .await?
            }
        };
        Ok(rows.iter().map(post_from_row).collect())
    }
}
