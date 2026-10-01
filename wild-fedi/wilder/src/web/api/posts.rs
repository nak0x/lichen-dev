use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;

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
}

#[derive(Deserialize)]
struct NewPostInput {
    #[serde(default)]
    group: Option<String>,
    content: String,
    #[serde(default)]
    tags: Vec<String>,
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
            },
        )
        .await?;
    Ok(Json(post.into()))
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
