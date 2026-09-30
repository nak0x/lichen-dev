use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The ActivityPub actor types relevant to LICHEN (README §2a): a child
/// (`Person`), a class (`Group`), a school (`Organization`), a capture station
/// (`Service`), or a bot (`Application`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    Person,
    Group,
    Organization,
    Service,
    Application,
}

/// The `publicKey` object embedded in an actor document. Remote servers fetch
/// ours to verify our HTTP signatures; we fetch theirs to verify inbound ones.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub id: String,    // e.g. https://host/users/alice#main-key
    pub owner: String, // the actor id that owns this key
    pub public_key_pem: String,
}

/// The `endpoints` object. We only advertise `sharedInbox` (README §2d) so a
/// sender can batch-deliver a single POST for many of our local followers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    pub shared_inbox: String,
}

/// An actor document served from `GET /users/:name`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    #[serde(rename = "@context")]
    pub context: Value,
    pub id: String,
    #[serde(rename = "type")]
    pub kind: ActorType,
    pub preferred_username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub inbox: String,
    pub outbox: String,
    pub followers: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub following: Option<String>,
    pub public_key: PublicKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Endpoints>,
}

/// A *remote* actor we fetched. Deliberately lenient: we model only the fields
/// we actually consume (`inbox`, `endpoints`, `publicKey`) and let serde ignore
/// everything else, because we cannot assume other servers populate the same
/// optional fields we do.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteActor {
    pub inbox: String,
    #[serde(default)]
    pub endpoints: Option<RemoteEndpoints>,
    #[serde(default)]
    pub public_key: Option<PublicKey>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteEndpoints {
    #[serde(default)]
    pub shared_inbox: Option<String>,
}
