//! Typed models for the ActivityStreams 2.0 vocabulary and the ActivityPub
//! extensions we emit and accept.
//!
//! We model the wire format with explicit `serde` structs rather than depending
//! on the `activitystreams` crate. That crate is only published as a perpetual
//! `0.7.0-alpha` and does not model an actor's `publicKey` as a first-class
//! field (it needs hand-rolled extension structs via a second alpha crate,
//! `activitystreams-ext`). Modelling the vocabulary directly keeps us on stable
//! dependencies, matches how production servers (Mastodon, GoToSocial) do it, and
//! gives us precise control over the JSON-LD `@context` — which is what actually
//! determines interoperability on the wire.

pub mod activity;
pub mod actor;
pub mod collection;
pub mod object;

pub use activity::{Activity, IncomingActivity};
pub use actor::{Actor, ActorType, Endpoints, PublicKey};
pub use collection::OrderedCollection;
pub use object::Object;

use serde_json::{Value, json};

/// The magic "everyone" address. Anything addressed here is world-readable.
pub const PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

/// The base ActivityStreams context every document must carry.
pub const AS_CONTEXT: &str = "https://www.w3.org/ns/activitystreams";
/// The security vocabulary that defines `publicKey` / `publicKeyPem`.
pub const SECURITY_CONTEXT: &str = "https://w3id.org/security/v1";

/// The `@context` for actor documents — needs the security vocab for the key.
pub fn actor_context() -> Value {
    json!([AS_CONTEXT, SECURITY_CONTEXT])
}

/// The `@context` for plain activities and objects.
pub fn activity_context() -> Value {
    json!(AS_CONTEXT)
}

/// Extract an object's `id` whether it arrived as a bare IRI string
/// (`"https://.../actor"`) or an embedded object (`{ "id": "https://..." }`).
/// AS2 allows either form almost anywhere, so every consumer must handle both.
pub fn extract_id(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Object(map) => map.get("id").and_then(Value::as_str).map(str::to_owned),
        _ => None,
    }
}
