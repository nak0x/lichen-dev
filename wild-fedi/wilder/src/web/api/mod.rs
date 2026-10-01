//! The client-facing REST API (`/api/v1/*`), distinct from the server-to-server
//! ActivityPub surface. It is what `wilder-web` talks to: JSON in, JSON out,
//! authenticated by the session cookie. Federation still happens over the AP
//! endpoints; this is the human/app side.

mod accounts;
mod auth;

use axum::Router;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Account, Role};
use crate::state::AppState;

/// Aggregate every `/api/v1` sub-router. Mounted by `web::router`.
pub fn routes() -> Router<AppState> {
    Router::new().merge(auth::routes()).merge(accounts::routes())
}

/// Public projection of an account — everything the client may see, and nothing
/// it may not (the password hash never crosses this boundary).
#[derive(Serialize)]
pub struct AccountView {
    pub id: Uuid,
    pub username: String,
    pub role: Role,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub created_at: String,
}

impl From<Account> for AccountView {
    fn from(a: Account) -> Self {
        Self {
            id: a.id,
            username: a.username,
            role: a.role,
            display_name: a.display_name,
            email: a.email,
            created_at: a.created_at.to_rfc3339(),
        }
    }
}

/// Input for creating an account (school bootstrap or student provisioning).
#[derive(Deserialize)]
pub struct NewAccountInput {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}
