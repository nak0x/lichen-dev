use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use super::PgStore;
use crate::domain::Measurement;
use crate::repository::MeasurementRepository;

#[async_trait]
impl MeasurementRepository for PgStore {
    async fn insert(&self, m: &Measurement) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO measurements \
                (id, group_username, metric, value, unit, recorded_at, recorded_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(m.id)
        .bind(&m.group_username)
        .bind(&m.metric)
        .bind(m.value)
        .bind(&m.unit)
        .bind(m.recorded_at)
        .bind(m.recorded_by)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    async fn for_group(&self, group_username: &str) -> anyhow::Result<Vec<Measurement>> {
        let rows = sqlx::query(
            "SELECT id, group_username, metric, value, unit, recorded_at, recorded_by \
             FROM measurements WHERE group_username = $1 ORDER BY metric, recorded_at",
        )
        .bind(group_username)
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .iter()
            .map(|r| Measurement {
                id: r.get("id"),
                group_username: r.get("group_username"),
                metric: r.get("metric"),
                value: r.get("value"),
                unit: r.get("unit"),
                recorded_at: r.get::<DateTime<Utc>, _>("recorded_at"),
                recorded_by: r.get("recorded_by"),
            })
            .collect())
    }
}
