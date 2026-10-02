use axum::Json;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use uuid::Uuid;

use crate::error::AppError;
use crate::service::determination_service::{NodeView, Step};
use crate::state::AppState;

// The taxonomy is public reference data, so these reads need no session.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/determination", get(roots))
        .route("/determination/{id}", get(step))
}

/// The starting choices of the key.
async fn roots(State(st): State<AppState>) -> Result<Json<Vec<NodeView>>, AppError> {
    Ok(Json(st.determination_service.roots().await?))
}

/// One step: the chosen node, its breadcrumb, and the next choices.
async fn step(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Step>, AppError> {
    Ok(Json(st.determination_service.step(id).await?))
}
