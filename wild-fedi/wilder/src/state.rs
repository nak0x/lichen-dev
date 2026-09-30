use std::sync::Arc;

use crate::config::AppConfig;
use crate::federation::FederationClient;
use crate::queue::JobQueue;
use crate::repository::ActorRepository;
use crate::service::ActorService;

/// The dependency container handed to every axum handler.
///
/// Cloning it is cheap — every field is an `Arc`. Handlers depend on the trait
/// objects here (`dyn ActorRepository`, `dyn JobQueue`), not concrete types, so
/// the wiring in `main` is the only place that knows the implementations.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub actors: Arc<dyn ActorRepository>,
    pub actor_service: Arc<ActorService>,
    pub federation: Arc<FederationClient>,
    pub queue: Arc<dyn JobQueue>,
}
