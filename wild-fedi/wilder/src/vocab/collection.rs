use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An `OrderedCollection` (used for the outbox and followers collections).
///
/// A production server paginates these with `OrderedCollectionPage`; a bare
/// server inlines the items and notes the omission. `totalItems` must stay
/// accurate regardless, for clients that read only the count.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderedCollection {
    #[serde(rename = "@context")]
    pub context: Value,
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub total_items: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ordered_items: Vec<Value>,
}

impl OrderedCollection {
    pub fn new(context: Value, id: String, items: Vec<Value>) -> Self {
        Self {
            context,
            id,
            kind: "OrderedCollection".to_string(),
            total_items: items.len(),
            ordered_items: items,
        }
    }
}
