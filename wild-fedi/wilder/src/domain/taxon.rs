use uuid::Uuid;

/// One node of the determination key. A non-leaf poses `question` and its
/// children are the answers; a leaf with `is_species` names a living thing.
#[derive(Debug, Clone)]
pub struct TaxonNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub rank: Option<String>,
    pub name: String,
    pub vernacular: Option<String>,
    pub is_species: bool,
    pub question: Option<String>,
    pub position: i32,
}
