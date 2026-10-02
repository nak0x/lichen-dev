use axum::Json;
use axum::extract::State;
use axum::routing::get;
use axum::Router;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::domain::{Announcement, AnnouncementKind, NewAnnouncement};
use crate::error::AppError;
use crate::state::AppState;
use crate::web::extract::SchoolUser;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/events", get(list_events).post(publish_event))
        .route("/news", get(list_news).post(publish_news))
        .route("/instance/stats", get(stats))
}

#[derive(Serialize)]
struct AnnouncementView {
    id: Uuid,
    kind: AnnouncementKind,
    author: String,
    title: String,
    content: String,
    start_time: Option<String>,
    location: Option<String>,
    published: String,
    activity: Value,
}

impl From<Announcement> for AnnouncementView {
    fn from(a: Announcement) -> Self {
        Self {
            id: a.id,
            kind: a.kind,
            author: a.author_username,
            title: a.title,
            content: a.content,
            start_time: a.start_time.map(|t| t.to_rfc3339()),
            location: a.location,
            published: a.published.to_rfc3339(),
            activity: a.activity,
        }
    }
}

#[derive(Deserialize)]
struct NewAnnouncementInput {
    title: String,
    content: String,
    #[serde(default)]
    start_time: Option<DateTime<Utc>>,
    #[serde(default)]
    location: Option<String>,
}

async fn publish(
    st: &AppState,
    school: &crate::domain::Account,
    kind: AnnouncementKind,
    body: NewAnnouncementInput,
) -> Result<Json<AnnouncementView>, AppError> {
    let a = st
        .announcement_service
        .publish(
            school,
            NewAnnouncement {
                kind,
                title: body.title,
                content: body.content,
                start_time: body.start_time,
                location: body.location,
            },
        )
        .await?;
    Ok(Json(a.into()))
}

async fn publish_event(
    SchoolUser(school): SchoolUser,
    State(st): State<AppState>,
    Json(body): Json<NewAnnouncementInput>,
) -> Result<Json<AnnouncementView>, AppError> {
    publish(&st, &school, AnnouncementKind::Event, body).await
}

async fn publish_news(
    SchoolUser(school): SchoolUser,
    State(st): State<AppState>,
    Json(body): Json<NewAnnouncementInput>,
) -> Result<Json<AnnouncementView>, AppError> {
    publish(&st, &school, AnnouncementKind::News, body).await
}

async fn list_events(State(st): State<AppState>) -> Result<Json<Vec<AnnouncementView>>, AppError> {
    let items = st.announcement_service.list(AnnouncementKind::Event).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

async fn list_news(State(st): State<AppState>) -> Result<Json<Vec<AnnouncementView>>, AppError> {
    let items = st.announcement_service.list(AnnouncementKind::News).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

/// Instance figures for the school dashboard.
async fn stats(
    SchoolUser(school): SchoolUser,
    State(st): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let students = st.account_service.roster(school.id).await?.len();
    let groups = st.group_service.all().await?.len();
    let events = st.announcement_service.count(AnnouncementKind::Event).await?;
    let news = st.announcement_service.count(AnnouncementKind::News).await?;
    Ok(Json(json!({
        "students": students,
        "groups": groups,
        "events": events,
        "news": news,
    })))
}
