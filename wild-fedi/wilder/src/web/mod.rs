//! The HTTP edge: the seven ActivityPub contracts (README §4, Path B) mapped to
//! axum handlers, plus the response helpers that set the content types peers
//! content-negotiate on.

mod actor;
mod inbox;
mod wellknown;

use axum::Router;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Build the router. Note axum 0.8 path params use `{name}`, not the old `:name`.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/users/{name}", get(actor::actor))
        .route("/users/{name}/inbox", post(inbox::inbox))
        .route("/users/{name}/outbox", get(actor::outbox))
        .route("/users/{name}/followers", get(actor::followers))
        .route("/inbox", post(inbox::shared_inbox)) // sharedInbox for fan-out
        .route("/.well-known/webfinger", get(wellknown::webfinger))
        .route("/.well-known/nodeinfo", get(wellknown::nodeinfo_discovery))
        .route("/nodeinfo/2.1", get(wellknown::nodeinfo))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Serialise `T` as JSON with the ActivityPub content type. Using the right
/// content type is not cosmetic: servers content-negotiate, and many will refuse
/// to treat plain `application/json` as ActivityPub.
pub struct ActivityJson<T>(pub T);

impl<T: Serialize> IntoResponse for ActivityJson<T> {
    fn into_response(self) -> Response {
        match serde_json::to_vec(&self.0) {
            Ok(body) => (
                [(CONTENT_TYPE, "application/activity+json")],
                body,
            )
                .into_response(),
            Err(e) => crate::error::AppError::Internal(e.into()).into_response(),
        }
    }
}
