use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Router, http::HeaderMap};
use serde::Deserialize;
use serde_json::json;

use super::{AccountView, NewAccountInput};
use crate::domain::NewAccount;
use crate::error::AppError;
use crate::state::AppState;
use crate::web::cookie::{self, SESSION_COOKIE};

/// Cookie lifetime, matching the Redis session TTL (one week).
const SESSION_MAX_AGE: i64 = 7 * 24 * 60 * 60;

#[derive(Deserialize)]
struct LoginInput {
    username: String,
    password: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/bootstrap", get(bootstrap_status).post(register_school))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
}

/// Whether first-run school registration is still open (no accounts yet).
async fn bootstrap_status(State(st): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let available = st.account_service.bootstrap_available().await?;
    Ok(Json(json!({ "available": available })))
}

/// Create the first school and immediately open its session.
async fn register_school(
    State(st): State<AppState>,
    Json(body): Json<NewAccountInput>,
) -> Result<impl IntoResponse, AppError> {
    let account = st
        .account_service
        .bootstrap_school(NewAccount {
            username: body.username,
            password: body.password,
            display_name: body.display_name,
            email: body.email,
        })
        .await?;
    let token = st.auth.start_session(account.id).await?;
    Ok(with_session_cookie(&st, &token, StatusCode::CREATED, account.into()))
}

async fn login(
    State(st): State<AppState>,
    Json(body): Json<LoginInput>,
) -> Result<impl IntoResponse, AppError> {
    // Throttle per-username to blunt credential brute-forcing: 10 tries / 5 min.
    if !st
        .rate_limiter
        .allow(&format!("login:{}", body.username), 10, 300)
        .await
    {
        return Err(AppError::TooManyRequests);
    }
    let (account, token) = st
        .auth
        .login(&body.username, &body.password)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".to_string()))?;
    Ok(with_session_cookie(&st, &token, StatusCode::OK, account.into()))
}

async fn logout(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    if let Some(token) = cookie::read(&headers, SESSION_COOKIE) {
        st.auth.logout(&token).await?;
    }
    let secure = st.config.scheme == "https";
    Ok((
        StatusCode::NO_CONTENT,
        [(SET_COOKIE, cookie::clear_session(secure))],
    ))
}

async fn me(user: crate::web::extract::CurrentUser) -> Json<AccountView> {
    Json(user.0.into())
}

/// Attach the session cookie to a JSON response with the given status. Returns an
/// owned `Response` so it borrows nothing from its arguments.
fn with_session_cookie(
    st: &AppState,
    token: &str,
    status: StatusCode,
    view: AccountView,
) -> Response {
    let secure = st.config.scheme == "https";
    let cookie = cookie::set_session(token, secure, SESSION_MAX_AGE);
    (status, [(SET_COOKIE, cookie)], Json(view)).into_response()
}
