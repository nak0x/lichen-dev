use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::postgres::PgRow;
use uuid::Uuid;

use super::PgStore;
use crate::domain::{Group, Member, MembershipRole};
use crate::repository::GroupRepository;

fn group_from_row(r: &PgRow) -> Group {
    Group {
        username: r.get("username"),
        name: r.get("name"),
        summary: r.get("summary"),
        subject: r.get("subject"),
        created_by: r.get("created_by"),
        created_at: r.get::<DateTime<Utc>, _>("created_at"),
    }
}

#[async_trait]
impl GroupRepository for PgStore {
    async fn insert(&self, group: &Group) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO groups (username, name, summary, subject, created_by, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&group.username)
        .bind(&group.name)
        .bind(&group.summary)
        .bind(&group.subject)
        .bind(group.created_by)
        .bind(group.created_at)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn by_username(&self, username: &str) -> anyhow::Result<Option<Group>> {
        let row = sqlx::query(
            "SELECT username, name, summary, subject, created_by, created_at \
             FROM groups WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.as_ref().map(group_from_row))
    }

    async fn all(&self) -> anyhow::Result<Vec<Group>> {
        let rows = sqlx::query(
            "SELECT username, name, summary, subject, created_by, created_at \
             FROM groups ORDER BY created_at DESC",
        )
        .fetch_all(self.pool())
        .await?;
        Ok(rows.iter().map(group_from_row).collect())
    }

    async fn for_account(&self, account_id: Uuid) -> anyhow::Result<Vec<Group>> {
        let rows = sqlx::query(
            "SELECT g.username, g.name, g.summary, g.subject, g.created_by, g.created_at \
             FROM groups g JOIN memberships m ON m.group_username = g.username \
             WHERE m.account_id = $1 ORDER BY g.created_at DESC",
        )
        .bind(account_id)
        .fetch_all(self.pool())
        .await?;
        Ok(rows.iter().map(group_from_row).collect())
    }

    async fn add_member(
        &self,
        group_username: &str,
        account_id: Uuid,
        role: MembershipRole,
    ) -> anyhow::Result<()> {
        // Idempotent: re-joining updates the role rather than erroring.
        sqlx::query(
            "INSERT INTO memberships (group_username, account_id, role) VALUES ($1, $2, $3) \
             ON CONFLICT (group_username, account_id) DO UPDATE SET role = EXCLUDED.role",
        )
        .bind(group_username)
        .bind(account_id)
        .bind(role.as_db())
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn remove_member(&self, group_username: &str, account_id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM memberships WHERE group_username = $1 AND account_id = $2")
            .bind(group_username)
            .bind(account_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    async fn members(&self, group_username: &str) -> anyhow::Result<Vec<Member>> {
        let rows = sqlx::query(
            "SELECT a.id, a.username, a.display_name, m.role \
             FROM memberships m JOIN accounts a ON a.id = m.account_id \
             WHERE m.group_username = $1 ORDER BY m.role, a.username",
        )
        .bind(group_username)
        .fetch_all(self.pool())
        .await?;
        rows.iter()
            .map(|r| {
                let role_str: String = r.get("role");
                Ok(Member {
                    account_id: r.get("id"),
                    username: r.get("username"),
                    display_name: r.get("display_name"),
                    role: MembershipRole::from_db(&role_str)
                        .ok_or_else(|| anyhow::anyhow!("bad membership role: {role_str}"))?,
                })
            })
            .collect()
    }

    async fn role_of(
        &self,
        group_username: &str,
        account_id: Uuid,
    ) -> anyhow::Result<Option<MembershipRole>> {
        let row =
            sqlx::query("SELECT role FROM memberships WHERE group_username = $1 AND account_id = $2")
                .bind(group_username)
                .bind(account_id)
                .fetch_optional(self.pool())
                .await?;
        match row {
            Some(r) => {
                let role_str: String = r.get("role");
                Ok(MembershipRole::from_db(&role_str))
            }
            None => Ok(None),
        }
    }
}
