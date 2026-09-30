use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A published object — a `Note`, `Image`, `Video`, `Place`, etc. For LICHEN a
/// nature observation is typically a `Note` carrying `Image`/`Audio` attachments
/// and a `Place` location (README §5).
///
/// `attachment`, `tag`, and `location` stay as raw `Value`s on purpose: the AS2
/// vocabulary for those is open-ended, and forcing every possible shape into
/// Rust types here would be brittle without buying us anything the callers need.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributed_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachment: Vec<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tag: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Value>,
}
