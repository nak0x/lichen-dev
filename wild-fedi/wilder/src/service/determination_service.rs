use std::sync::Arc;

use serde::Serialize;
use uuid::Uuid;

use crate::domain::TaxonNode;
use crate::error::AppError;
use crate::repository::TaxonomyRepository;

/// Walks the determination key. The client starts at the roots, then repeatedly
/// steps into the chosen child, narrowing from the general to a species leaf.
pub struct DeterminationService {
    taxonomy: Arc<dyn TaxonomyRepository>,
}

/// A serialisable view of a taxon node for the API/UI.
#[derive(Serialize)]
pub struct NodeView {
    pub id: Uuid,
    pub name: String,
    pub rank: Option<String>,
    pub vernacular: Option<String>,
    pub is_species: bool,
    pub question: Option<String>,
}

impl From<&TaxonNode> for NodeView {
    fn from(n: &TaxonNode) -> Self {
        Self {
            id: n.id,
            name: n.name.clone(),
            rank: n.rank.clone(),
            vernacular: n.vernacular.clone(),
            is_species: n.is_species,
            question: n.question.clone(),
        }
    }
}

/// One step of the key: where we are, the trail that led here, and the choices
/// onward. Empty `children` on a `is_species` node means we have an answer.
#[derive(Serialize)]
pub struct Step {
    pub node: NodeView,
    pub ancestors: Vec<NodeView>,
    pub children: Vec<NodeView>,
}

impl DeterminationService {
    pub fn new(taxonomy: Arc<dyn TaxonomyRepository>) -> Self {
        Self { taxonomy }
    }

    /// The starting choices (top-level nodes).
    pub async fn roots(&self) -> Result<Vec<NodeView>, AppError> {
        Ok(self.taxonomy.roots().await?.iter().map(NodeView::from).collect())
    }

    /// Resolve one node into a full step (with breadcrumb and next choices).
    pub async fn step(&self, id: Uuid) -> Result<Step, AppError> {
        let node = self.taxonomy.by_id(id).await?.ok_or(AppError::NotFound)?;
        let children = self.taxonomy.children(id).await?;

        // Breadcrumb: climb parents to the root (bounded by tree depth).
        let mut ancestors = Vec::new();
        let mut current = node.parent_id;
        while let Some(pid) = current {
            if let Some(parent) = self.taxonomy.by_id(pid).await? {
                current = parent.parent_id;
                ancestors.push(NodeView::from(&parent));
            } else {
                break;
            }
        }
        ancestors.reverse();

        Ok(Step {
            node: NodeView::from(&node),
            ancestors,
            children: children.iter().map(NodeView::from).collect(),
        })
    }
}
