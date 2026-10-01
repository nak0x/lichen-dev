use axum::Json;
use axum::extract::State;
use axum::routing::{get, post};
use axum::Router;

use super::{AccountView, NewAccountInput};
use crate::domain::NewAccount;
use crate::error::AppError;
use crate::state::AppState;
use crate::web::extract::SchoolUser;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/accounts/students", post(create_student))
        .route("/accounts", get(list_accounts))
}

/// A school provisions a student account. `SchoolUser` enforces the role before
/// the handler body runs; `create_student` re-checks as defence in depth.
async fn create_student(
    SchoolUser(school): SchoolUser,
    State(st): State<AppState>,
    Json(body): Json<NewAccountInput>,
) -> Result<Json<AccountView>, AppError> {
    let account = st
        .account_service
        .create_student(
            &school,
            NewAccount {
                username: body.username,
                password: body.password,
                display_name: body.display_name,
                email: body.email,
            },
        )
        .await?;
    Ok(Json(account.into()))
}

/// The roster of accounts this school has created.
async fn list_accounts(
    SchoolUser(school): SchoolUser,
    State(st): State<AppState>,
) -> Result<Json<Vec<AccountView>>, AppError> {
    let roster = st.account_service.roster(school.id).await?;
    Ok(Json(roster.into_iter().map(Into::into).collect()))
}
