use axum::Json;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::domain::NewMeasurement;
use crate::error::AppError;
use crate::service::measurement_service::Series;
use crate::state::AppState;
use crate::web::extract::CurrentUser;

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/groups/{username}/measurements",
        get(series).post(record),
    )
}

#[derive(Deserialize)]
struct NewMeasurementInput {
    metric: String,
    value: f64,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    recorded_at: Option<DateTime<Utc>>,
}

/// Record a reading (members only — includes capture-station accounts).
async fn record(
    CurrentUser(user): CurrentUser,
    State(st): State<AppState>,
    Path(username): Path<String>,
    Json(body): Json<NewMeasurementInput>,
) -> Result<axum::http::StatusCode, AppError> {
    st.measurement_service
        .record(
            &user,
            &username,
            NewMeasurement {
                metric: body.metric,
                value: body.value,
                unit: body.unit,
                recorded_at: body.recorded_at,
            },
        )
        .await?;
    Ok(axum::http::StatusCode::CREATED)
}

/// The group's measurements as per-metric series. Empty ⇒ the UI shows no chart.
async fn series(
    State(st): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Vec<Series>>, AppError> {
    Ok(Json(st.measurement_service.series(&username).await?))
}
