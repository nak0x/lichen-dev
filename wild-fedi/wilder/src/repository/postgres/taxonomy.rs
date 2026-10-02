use async_trait::async_trait;
use sqlx::Row;
use sqlx::postgres::PgRow;
use uuid::Uuid;

use super::PgStore;
use crate::domain::TaxonNode;
use crate::repository::TaxonomyRepository;

fn node_from_row(r: &PgRow) -> TaxonNode {
    TaxonNode {
        id: r.get("id"),
        parent_id: r.get("parent_id"),
        rank: r.get("rank"),
        name: r.get("name"),
        vernacular: r.get("vernacular"),
        is_species: r.get("is_species"),
        question: r.get("question"),
        position: r.get("position"),
    }
}

#[async_trait]
impl TaxonomyRepository for PgStore {
    async fn roots(&self) -> anyhow::Result<Vec<TaxonNode>> {
        let rows = sqlx::query(
            "SELECT id, parent_id, rank, name, vernacular, is_species, question, position \
             FROM taxon_nodes WHERE parent_id IS NULL ORDER BY position, name",
        )
        .fetch_all(self.pool())
        .await?;
        Ok(rows.iter().map(node_from_row).collect())
    }

    async fn children(&self, parent_id: Uuid) -> anyhow::Result<Vec<TaxonNode>> {
        let rows = sqlx::query(
            "SELECT id, parent_id, rank, name, vernacular, is_species, question, position \
             FROM taxon_nodes WHERE parent_id = $1 ORDER BY position, name",
        )
        .bind(parent_id)
        .fetch_all(self.pool())
        .await?;
        Ok(rows.iter().map(node_from_row).collect())
    }

    async fn by_id(&self, id: Uuid) -> anyhow::Result<Option<TaxonNode>> {
        let row = sqlx::query(
            "SELECT id, parent_id, rank, name, vernacular, is_species, question, position \
             FROM taxon_nodes WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.as_ref().map(node_from_row))
    }

    async fn count(&self) -> anyhow::Result<usize> {
        let row = sqlx::query("SELECT count(*) AS n FROM taxon_nodes")
            .fetch_one(self.pool())
            .await?;
        Ok(row.get::<i64, _>("n") as usize)
    }

    async fn insert(&self, node: &TaxonNode) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO taxon_nodes \
                (id, parent_id, rank, name, vernacular, is_species, question, position) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(node.id)
        .bind(node.parent_id)
        .bind(&node.rank)
        .bind(&node.name)
        .bind(&node.vernacular)
        .bind(node.is_species)
        .bind(&node.question)
        .bind(node.position)
        .execute(self.pool())
        .await?;
        Ok(())
    }
}
