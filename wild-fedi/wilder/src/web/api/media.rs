use axum::Json;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::HeaderMap;
use axum::http::header::CONTENT_TYPE;
use axum::routing::post;
use axum::Router;
use serde::Serialize;

use crate::error::AppError;
use crate::state::AppState;
use crate::web::extract::CurrentUser;

/// Accept images a little above the service's own 8 MiB cap, leaving slack for
/// multipart/encoding overhead; the service still enforces the real limit.
const UPLOAD_LIMIT: usize = 9 * 1024 * 1024;

#[derive(Serialize)]
pub struct MediaUploadView {
    pub id: uuid::Uuid,
    pub url: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/media", post(upload))
        .layer(DefaultBodyLimit::max(UPLOAD_LIMIT))
}

/// Upload an image as the raw request body (its `Content-Type` is the image
/// type). Returns the new media id and the URL it is served at.
async fn upload(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<MediaUploadView>, AppError> {
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let media = st
        .media_service
        .upload(&user, &content_type, body.to_vec())
        .await?;
    Ok(Json(MediaUploadView {
        url: st.media_service.url_for(media.id),
        id: media.id,
    }))
}
