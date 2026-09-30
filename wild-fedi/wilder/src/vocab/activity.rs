use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// An activity we *emit* (`Create`, `Accept`, `Follow`, `Announce`, ...).
/// `object` is a `Value` because it may be an embedded object (a `Create`
/// wrapping a `Note`) or a bare IRI (a `Follow` targeting an actor URL).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    #[serde(rename = "@context")]
    pub context: Value,
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    pub object: Value,
}

/// An activity we *receive* at an inbox. Deliberately lenient: we pin down only
/// the fields we route on and keep everything else in `extra`, because the
/// fediverse is full of implementations that add, omit, or reshape fields.
/// "Be conservative in what you send, be liberal in what you accept."
#[derive(Debug, Clone, Deserialize)]
pub struct IncomingActivity {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub actor: Option<Value>,
    #[serde(default)]
    pub object: Option<Value>,
    // Everything not named above (to, cc, published, ...) lands here untouched.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl IncomingActivity {
    /// The IRI of the actor that authored this activity, whether `actor` arrived
    /// as a string or an embedded object.
    pub fn actor_id(&self) -> Option<String> {
        self.actor.as_ref().and_then(super::extract_id)
    }
}
