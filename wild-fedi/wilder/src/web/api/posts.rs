use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use uuid::Uuid;

use super::PostView;
use crate::domain::NewPost;
use crate::error::AppError;
use crate::state::AppState;
use crate::web::extract::CurrentUser;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/posts", post(create))
        .route("/groups/{username}/posts", get(group_posts))
        .route("/groups/{username}/captured", get(group_captured))
        .route("/groups/{username}/pictures", get(group_pictures))
        // Moderation (group admins): the review queue and its two decisions.
        .route("/groups/{username}/pending", get(group_pending))
        .route("/posts/{id}/approve", post(approve))
        .route("/posts/{id}/reject", post(reject))
}

#[derive(Deserialize)]
struct NewPostInput {
    #[serde(default)]
    group: Option<String>,
    content: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    media: Vec<uuid::Uuid>,
}

/// Compose a post. Members post into a group; anyone posts to their own
/// followers (group omitted). Membership is enforced in the service.
async fn create(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Json(body): Json<NewPostInput>,
) -> Result<Json<PostView>, AppError> {
    let post = st
        .post_service
        .create(
            &user,
            NewPost {
                group_username: body.group,
                content: body.content,
                tags: body.tags,
                media: body.media,
            },
        )
        .await?;
    Ok(Json(post.into()))
}

/// A group's picture space: every image attached to its posts.
async fn group_pictures(
    State(st): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    Ok(Json(st.post_service.pictures(&username).await?))
}

#[derive(Deserialize)]
struct PostFilter {
    #[serde(default)]
    tag: Option<String>,
}

/// A group's posts, newest first, optionally filtered to one tag.
async fn group_posts(
    State(st): State<AppState>,
    Path(username): Path<String>,
    Query(filter): Query<PostFilter>,
) -> Result<Json<Vec<PostView>>, AppError> {
    let posts = st
        .post_service
        .in_group(&username, filter.tag.as_deref())
        .await?;
    Ok(Json(posts.into_iter().map(Into::into).collect()))
}

/// A group's "captured" feed: only posts authored by `Service` (capture-station)
/// actors — the automated eyes on the outside world.
async fn group_captured(
    State(st): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Vec<PostView>>, AppError> {
    let posts = st.post_service.captured(Some(&username)).await?;
    Ok(Json(posts.into_iter().map(Into::into).collect()))
}

/// A group's review queue: guest submissions awaiting a decision. Admins only
/// (enforced in the service).
async fn group_pending(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Vec<PostView>>, AppError> {
    let posts = st.post_service.pending(&user, &username).await?;
    Ok(Json(posts.into_iter().map(Into::into).collect()))
}

/// Approve a held submission — it goes live and federates. Admins only.
async fn approve(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PostView>, AppError> {
    let post = st.post_service.approve(&user, id).await?;
    Ok(Json(post.into()))
}

/// Decline a held submission. Admins only.
async fn reject(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    st.post_service.reject(&user, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
