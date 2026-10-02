use std::sync::Arc;

use crate::config::AppConfig;
use crate::federation::FederationClient;
use crate::queue::JobQueue;
use crate::repository::ActorRepository;
use crate::service::{
    AccountService, ActorService, AuthService, DeterminationService, FeedService, GroupService,
    MeasurementService, MediaService, NoteService, PostService,
};

/// The dependency container handed to every axum handler.
///
/// Cloning it is cheap — every field is an `Arc`. Handlers depend on the trait
/// objects and services here, not concrete types, so the wiring in `main` is the
/// only place that knows the implementations.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub actors: Arc<dyn ActorRepository>,
    pub actor_service: Arc<ActorService>,
    pub federation: Arc<FederationClient>,
    pub queue: Arc<dyn JobQueue>,
    // --- identity (Phase 1) ---
    pub auth: Arc<AuthService>,
    pub account_service: Arc<AccountService>,
    // --- groups, posts, feeds (Phase 2) ---
    pub group_service: Arc<GroupService>,
    pub post_service: Arc<PostService>,
    pub feed_service: Arc<FeedService>,
    // --- media (Phase 3) ---
    pub media_service: Arc<MediaService>,
    // --- measurements (Phase 4) ---
    pub measurement_service: Arc<MeasurementService>,
    // --- personal docs & determination key (Phase 5) ---
    pub note_service: Arc<NoteService>,
    pub determination_service: Arc<DeterminationService>,
}
