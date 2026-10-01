//! Request extractors that turn a session cookie into an authenticated account,
//! and a role-gated variant for school-only endpoints. Putting authentication in
//! an extractor means a handler that names `CurrentUser` *cannot* run
//! unauthenticated — the type system enforces the guard.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::domain::{Account, Role};
use crate::error::AppError;
use crate::state::AppState;
use crate::web::cookie::{self, SESSION_COOKIE};

/// The authenticated account behind a request. Rejects with 401 when there is no
/// valid session.
pub struct CurrentUser(pub Account);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = cookie::read(&parts.headers, SESSION_COOKIE)
            .ok_or_else(|| AppError::Unauthorized("no session".to_string()))?;
        let account = state
            .auth
            .account_for_token(&token)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid or expired session".to_string()))?;
        Ok(CurrentUser(account))
    }
}

/// A `CurrentUser` additionally proven to be a school. Handlers that administer
/// the instance name this type and get the 403 for free.
pub struct SchoolUser(pub Account);

impl FromRequestParts<AppState> for SchoolUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let CurrentUser(account) = CurrentUser::from_request_parts(parts, state).await?;
        if account.role == Role::School {
            Ok(SchoolUser(account))
        } else {
            Err(AppError::Forbidden(
                "this action requires a school account".to_string(),
            ))
        }
    }
}
