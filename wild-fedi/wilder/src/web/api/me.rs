use axum::Json;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::HeaderMap;
use axum::http::header::CONTENT_TYPE;
use axum::routing::{patch, post};
use axum::Router;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;
use crate::web::extract::CurrentUser;

const UPLOAD_LIMIT: usize = 9 * 1024 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me/profile", patch(update_profile))
        .route(
            "/me/avatar",
            post(set_avatar).layer(DefaultBodyLimit::max(UPLOAD_LIMIT)),
        )
}

#[derive(Deserialize)]
struct ProfileInput {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    summary: Option<String>,
}

/// Update the caller's display name and/or bio. The actor's own repository holds
/// these (the actor document reflects them immediately).
async fn update_profile(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Json(body): Json<ProfileInput>,
) -> Result<axum::http::StatusCode, AppError> {
    st.actors
        .update_bio(
            &user.username,
            body.display_name.as_deref(),
            body.summary.as_deref(),
        )
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
struct AvatarView {
    url: String,
}

/// Set the caller's avatar from a raw image body; returns its URL.
async fn set_avatar(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<AvatarView>, AppError> {
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let url = st
        .media_service
        .set_avatar(&user, &content_type, body.to_vec())
        .await?;
    Ok(Json(AvatarView { url }))
}
