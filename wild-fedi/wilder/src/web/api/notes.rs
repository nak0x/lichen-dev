use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::{get, patch};
use axum::Router;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{NewNote, Note, NoteKind};
use crate::error::AppError;
use crate::state::AppState;
use crate::web::extract::CurrentUser;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/notes", get(list).post(create))
        .route("/notes/{id}", patch(update).delete(remove))
}

#[derive(Serialize)]
struct NoteView {
    id: Uuid,
    kind: NoteKind,
    title: Option<String>,
    content: String,
    updated_at: String,
}

impl From<Note> for NoteView {
    fn from(n: Note) -> Self {
        Self {
            id: n.id,
            kind: n.kind,
            title: n.title,
            content: n.content,
            updated_at: n.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Deserialize)]
struct KindQuery {
    #[serde(default)]
    kind: Option<String>,
}

fn parse_kind(s: Option<&str>) -> Result<NoteKind, AppError> {
    match s {
        Some("research") => Ok(NoteKind::Research),
        Some("note") | None => Ok(NoteKind::Note),
        Some(other) => Err(AppError::BadRequest(format!("unknown kind: {other}"))),
    }
}

async fn list(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Query(q): Query<KindQuery>,
) -> Result<Json<Vec<NoteView>>, AppError> {
    let kind = parse_kind(q.kind.as_deref())?;
    let notes = st.note_service.list(&user, kind).await?;
    Ok(Json(notes.into_iter().map(Into::into).collect()))
}

#[derive(Deserialize)]
struct NewNoteInput {
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    title: Option<String>,
    content: String,
}

async fn create(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Json(body): Json<NewNoteInput>,
) -> Result<Json<NoteView>, AppError> {
    let kind = parse_kind(body.kind.as_deref())?;
    let note = st
        .note_service
        .create(
            &user,
            NewNote {
                kind,
                title: body.title,
                content: body.content,
            },
        )
        .await?;
    Ok(Json(note.into()))
}

#[derive(Deserialize)]
struct UpdateNoteInput {
    #[serde(default)]
    title: Option<String>,
    content: String,
}

async fn update(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateNoteInput>,
) -> Result<axum::http::StatusCode, AppError> {
    st.note_service
        .update(&user, id, body.title, body.content)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn remove(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    st.note_service.delete(&user, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
