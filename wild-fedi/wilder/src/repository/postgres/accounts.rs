use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::postgres::PgRow;
use uuid::Uuid;

use super::PgStore;
use crate::domain::{Account, Role};
use crate::repository::AccountRepository;

/// Map a row to an `Account`. An unknown role string would be a data-integrity
/// bug, so we fail loudly rather than silently coercing.
fn account_from_row(r: &PgRow) -> anyhow::Result<Account> {
    let role_str: String = r.get("role");
    let role = Role::from_db(&role_str)
        .ok_or_else(|| anyhow::anyhow!("unknown role in db: {role_str}"))?;
    Ok(Account {
        id: r.get("id"),
        username: r.get("username"),
        role,
        display_name: r.get("display_name"),
        email: r.get("email"),
        password_hash: r.get("password_hash"),
        created_by: r.get("created_by"),
        created_at: r.get::<DateTime<Utc>, _>("created_at"),
    })
}

// sqlx 0.9 only accepts `&'static str` queries (an anti-injection guard), so
// each query is a full literal rather than a `format!` over a shared column list.

#[async_trait]
impl AccountRepository for PgStore {
    async fn by_username(&self, username: &str) -> anyhow::Result<Option<Account>> {
        let row = sqlx::query(
            "SELECT id, username, role, display_name, email, password_hash, created_by, created_at \
             FROM accounts WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(self.pool())
        .await?;
        row.as_ref().map(account_from_row).transpose()
    }

    async fn by_id(&self, id: Uuid) -> anyhow::Result<Option<Account>> {
        let row = sqlx::query(
            "SELECT id, username, role, display_name, email, password_hash, created_by, created_at \
             FROM accounts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;
        row.as_ref().map(account_from_row).transpose()
    }

    async fn insert(&self, account: &Account) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO accounts \
                (id, username, role, display_name, email, password_hash, created_by, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(account.id)
        .bind(&account.username)
        .bind(account.role.as_db())
        .bind(&account.display_name)
        .bind(&account.email)
        .bind(&account.password_hash)
        .bind(account.created_by)
        .bind(account.created_at)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn count(&self) -> anyhow::Result<usize> {
        let row = sqlx::query("SELECT count(*) AS n FROM accounts")
            .fetch_one(self.pool())
            .await?;
        Ok(row.get::<i64, _>("n") as usize)
    }

    async fn list_created_by(&self, school_id: Uuid) -> anyhow::Result<Vec<Account>> {
        let rows = sqlx::query(
            "SELECT id, username, role, display_name, email, password_hash, created_by, created_at \
             FROM accounts WHERE created_by = $1 ORDER BY created_at",
        )
        .bind(school_id)
        .fetch_all(self.pool())
        .await?;
        rows.iter().map(account_from_row).collect()
    }
}
