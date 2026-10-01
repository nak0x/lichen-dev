use axum::Json;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::Router;
use serde::Deserialize;
use uuid::Uuid;

use super::{GroupView, MemberView};
use crate::domain::NewGroup;
use crate::error::AppError;
use crate::state::AppState;
use crate::web::extract::CurrentUser;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/groups", get(list).post(create))
        .route("/groups/mine", get(mine))
        .route("/groups/{username}", get(detail))
        .route("/groups/{username}/members", get(members))
        .route("/groups/{username}/members/{account_id}", delete(remove_member))
        .route("/groups/{username}/join", post(join))
        .route("/groups/{username}/leave", post(leave))
}

#[derive(Deserialize)]
struct NewGroupInput {
    username: String,
    name: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    subject: Option<String>,
}

/// Any authenticated account may start a group (classes and students alike).
async fn create(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Json(body): Json<NewGroupInput>,
) -> Result<Json<GroupView>, AppError> {
    let group = st
        .group_service
        .create(
            &user,
            NewGroup {
                username: body.username,
                name: body.name,
                summary: body.summary,
                subject: body.subject,
            },
        )
        .await?;
    // The creator is the admin.
    Ok(Json(GroupView::from(
        group,
        Some(crate::domain::MembershipRole::Admin),
    )))
}

/// List every group. Public: discovery does not require a session.
async fn list(State(st): State<AppState>) -> Result<Json<Vec<GroupView>>, AppError> {
    let groups = st.group_service.all().await?;
    Ok(Json(
        groups
            .into_iter()
            .map(|g| GroupView::from(g, None))
            .collect(),
    ))
}

/// Groups the current user belongs to.
async fn mine(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
) -> Result<Json<Vec<GroupView>>, AppError> {
    let groups = st.group_service.for_account(user.id).await?;
    Ok(Json(
        groups
            .into_iter()
            .map(|g| GroupView::from(g, None))
            .collect(),
    ))
}

/// A group's public document, annotated with the caller's membership if signed in.
async fn detail(
    State(st): State<AppState>,
    user: Option<CurrentUser>,
    Path(username): Path<String>,
) -> Result<Json<GroupView>, AppError> {
    let group = st.group_service.get(&username).await?;
    let role = match &user {
        Some(CurrentUser(acc)) => st.group_service.require_member(&username, acc.id).await.ok(),
        None => None,
    };
    Ok(Json(GroupView::from(group, role)))
}

async fn members(
    State(st): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Vec<MemberView>>, AppError> {
    let members = st.group_service.members(&username).await?;
    Ok(Json(members.into_iter().map(Into::into).collect()))
}

async fn join(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<GroupView>, AppError> {
    st.group_service.join(&user, &username).await?;
    let group = st.group_service.get(&username).await?;
    let role = st.group_service.require_member(&username, user.id).await.ok();
    Ok(Json(GroupView::from(group, role)))
}

async fn leave(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Path(username): Path<String>,
) -> Result<axum::http::StatusCode, AppError> {
    st.group_service.leave(&user, &username).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// Admin removes a member from the group.
async fn remove_member(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Path((username, account_id)): Path<(String, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    st.group_service
        .remove_member(&user, &username, account_id)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
