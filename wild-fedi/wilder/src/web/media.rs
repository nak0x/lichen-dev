use axum::extract::{Path, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

/// `GET /media/:id` — serve media bytes *through* wilder (the browser and remote
/// peers never touch the object store directly). Public, since media is embedded
/// in public posts and must resolve for any fetcher; cached, since the id→bytes
/// mapping is immutable.
pub async fn serve(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let object = st.media_service.fetch(id).await?;
    Ok((
        [
            (CONTENT_TYPE, object.content_type),
            (CACHE_CONTROL, "public, max-age=31536000, immutable".to_string()),
        ],
        object.bytes,
    )
        .into_response())
}
