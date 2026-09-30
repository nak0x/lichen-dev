use axum::extract::{Path, State};

use crate::error::AppError;
use crate::state::AppState;
use crate::vocab::{Actor, OrderedCollection};
use crate::web::ActivityJson;

/// `GET /users/:name` — the actor document (README §4, contract 1).
pub async fn actor(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<ActivityJson<Actor>, AppError> {
    let actor = state
        .actor_service
        .find(&name)
        .await
        .ok_or(AppError::NotFound)?;
    Ok(ActivityJson(state.actor_service.actor_document(&actor)))
}

/// `GET /users/:name/outbox` — the actor's published activities (contract 4).
pub async fn outbox(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<ActivityJson<OrderedCollection>, AppError> {
    // 404 unknown actors so we don't advertise empty collections for names we
    // don't host.
    state
        .actor_service
        .find(&name)
        .await
        .ok_or(AppError::NotFound)?;
    Ok(ActivityJson(
        state.actor_service.outbox_collection(&name).await,
    ))
}

/// `GET /users/:name/followers` — the followers collection (contract 5).
pub async fn followers(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<ActivityJson<OrderedCollection>, AppError> {
    state
        .actor_service
        .find(&name)
        .await
        .ok_or(AppError::NotFound)?;
    Ok(ActivityJson(
        state.actor_service.followers_collection(&name).await,
    ))
}
