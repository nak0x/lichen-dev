use std::collections::HashMap;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::error::AppError;
use crate::state::AppState;

/// `GET /.well-known/webfinger?resource=acct:name@domain` (README §2, contract 2).
/// Turns a human handle into an actor URL so clients can discover us.
pub async fn webfinger(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    let resource = params
        .get("resource")
        .ok_or_else(|| AppError::BadRequest("missing resource parameter".into()))?;
    let jrd = state
        .actor_service
        .webfinger(resource)
        .await
        .ok_or(AppError::NotFound)?;
    // WebFinger has its own content type (RFC 7033): application/jrd+json.
    Ok((
        [(CONTENT_TYPE, "application/jrd+json")],
        serde_json::to_vec(&jrd).expect("jrd serialises"),
    )
        .into_response())
}

/// `GET /.well-known/nodeinfo` — the discovery document pointing at 2.1.
pub async fn nodeinfo_discovery(State(state): State<AppState>) -> Response {
    Json(json!({
        "links": [{
            "rel": "http://nodeinfo.diaspora.software/ns/schema/2.1",
            "href": format!("{}/nodeinfo/2.1", state.config.base_url()),
        }],
    }))
    .into_response()
}

/// `GET /nodeinfo/2.1` — the NodeInfo document itself (README §4, contract 7).
pub async fn nodeinfo(State(state): State<AppState>) -> Response {
    Json(state.actor_service.nodeinfo().await).into_response()
}
