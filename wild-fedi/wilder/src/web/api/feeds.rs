use axum::Json;
use axum::extract::State;
use axum::routing::get;
use axum::Router;
use serde::Serialize;

use super::PostView;
use crate::error::AppError;
use crate::state::AppState;
use crate::web::extract::CurrentUser;

pub fn routes() -> Router<AppState> {
    Router::new().route("/feed", get(personal))
}

#[derive(Serialize)]
struct FeedResponse {
    /// The instant this batch covers from — "du <since> à maintenant".
    since: String,
    posts: Vec<PostView>,
}

/// The caller's since-last-visit feed. Reading it advances their visit mark, so
/// a second call returns only what is newer — bounded, never infinite.
async fn personal(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
) -> Result<Json<FeedResponse>, AppError> {
    let batch = st.feed_service.since_last_visit(&user).await?;
    Ok(Json(FeedResponse {
        since: batch.since.to_rfc3339(),
        posts: batch.posts.into_iter().map(Into::into).collect(),
    }))
}
