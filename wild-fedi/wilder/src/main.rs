//! wilder — a bare, queue-backed ActivityPub server for the LICHEN federated
//! network (see ../README.md, Path B).
//!
//! This file is the composition root: the *only* place that knows the concrete
//! implementations. It wires trait objects together and starts the two runtimes
//! that make up the server — the HTTP listener and the background worker pool.

mod config;
mod crypto;
mod error;
mod federation;
mod processor;
mod queue;
mod repository;
mod service;
mod state;
mod vocab;
mod web;

use std::sync::Arc;

use anyhow::Context;
use tracing_subscriber::EnvFilter;

use crate::config::AppConfig;
use crate::crypto::KeyPair;
use crate::federation::FederationClient;
use crate::processor::AppJobProcessor;
use crate::queue::{InMemoryQueue, JobQueue, WorkerPool};
use crate::repository::memory::InMemoryStore;
use crate::repository::{ActivityRepository, ActorRepository, FollowerRepository, LocalActor};
use crate::service::{ActorService, InboxService, PublishService};
use crate::state::AppState;
use crate::vocab::{Activity, ActorType, Object};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Structured logs; RUST_LOG overrides the default.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,wilder=debug")),
        )
        .init();

    let config = Arc::new(AppConfig::from_env()?);
    tracing::info!(domain = %config.domain, "starting wilder");

    // --- storage: one concrete store behind three trait objects (DIP) ---
    let store = Arc::new(InMemoryStore::new());
    let actors: Arc<dyn ActorRepository> = store.clone();
    let followers: Arc<dyn FollowerRepository> = store.clone();
    let activities: Arc<dyn ActivityRepository> = store.clone();

    // --- work queue: create the enqueue side first; workers are spawned last ---
    let (queue_impl, receiver) = InMemoryQueue::bounded(config.queue_capacity);
    let queue: Arc<dyn JobQueue> = Arc::new(queue_impl);

    // Seed a demo actor so the server is inspectable out of the box (README §7).
    // We publish through the same PublishService a real endpoint would use.
    let publish_service = PublishService::new(
        Arc::clone(&activities),
        Arc::clone(&followers),
        Arc::clone(&queue),
    );
    seed_demo_actor(&config, actors.as_ref(), &publish_service).await?;

    // --- federation client: the sole owner of outbound network I/O ---
    let federation = Arc::new(FederationClient::new(format!(
        "wilder/{} (+{})",
        config.software_version,
        config.base_url()
    ))?);

    // --- services ---
    let inbox_service = Arc::new(InboxService::new(
        Arc::clone(&config),
        Arc::clone(&followers),
        Arc::clone(&activities),
        Arc::clone(&federation),
        Arc::clone(&queue),
    ));
    let actor_service = Arc::new(ActorService::new(
        Arc::clone(&config),
        Arc::clone(&actors),
        Arc::clone(&followers),
        Arc::clone(&activities),
    ));

    // --- worker pool: consumes the queue via the processor adapter ---
    let processor = Arc::new(AppJobProcessor::new(
        Arc::clone(&config),
        Arc::clone(&inbox_service),
        Arc::clone(&federation),
        Arc::clone(&actors),
    ));
    let _workers = WorkerPool::spawn(config.worker_count, receiver, processor);
    tracing::info!(workers = config.worker_count, "worker pool started");

    // --- HTTP server ---
    let state = AppState {
        config: Arc::clone(&config),
        actors: Arc::clone(&actors),
        actor_service,
        federation,
        queue,
    };
    let app = web::router(state);

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .with_context(|| format!("binding {}", config.bind_addr))?;
    tracing::info!(addr = %config.bind_addr, "listening");
    axum::serve(listener, app).await.context("server error")?;
    Ok(())
}

/// Create one `Group` actor ("beaver-class") with a fresh keypair and a sample
/// observation in its outbox, mirroring the README's LICHEN scenario. This is
/// the one bit of imperative setup a fresh instance needs to be worth curling.
async fn seed_demo_actor(
    config: &AppConfig,
    actors: &dyn ActorRepository,
    publisher: &PublishService,
) -> anyhow::Result<()> {
    let username = "beaver-class";
    let keypair = KeyPair::generate()?;
    actors
        .insert(LocalActor {
            username: username.to_string(),
            kind: ActorType::Group,
            display_name: Some("Beaver Class".to_string()),
            summary: Some("Observations naturalistes de la classe Castor.".to_string()),
            private_key_pem: keypair.private_pem()?,
            public_key_pem: keypair.public_pem()?,
        })
        .await;

    // A sample observation, shaped like README §5's structured Note. Built with
    // our typed vocabulary models to exercise the serialisation path.
    let actor_id = config.actor_url(username);
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let note = Object {
        kind: "Note".to_string(),
        id: config.object_url(&uuid::Uuid::new_v4().to_string()),
        attributed_to: Some(actor_id.clone()),
        content: Some(
            "Traces de rongement fraîches sur un saule au bord de la rivière.".to_string(),
        ),
        published: Some(now.clone()),
        to: vec![vocab::PUBLIC.to_string()],
        cc: vec![config.followers_url(username)],
        attachment: vec![],
        tag: vec![serde_json::json!({ "type": "Hashtag", "name": "#castor" })],
        location: Some(serde_json::json!({
            "type": "Place",
            "name": "Rivière, aval du pont",
            "latitude": 45.75,
            "longitude": 4.85,
        })),
    };
    let create = Activity {
        context: vocab::activity_context(),
        kind: "Create".to_string(),
        id: config.activity_url(&uuid::Uuid::new_v4().to_string()),
        actor: actor_id.clone(),
        to: vec![vocab::PUBLIC.to_string()],
        cc: vec![config.followers_url(username)],
        published: Some(now),
        object: serde_json::to_value(&note)?,
    };
    publisher
        .publish(username, serde_json::to_value(&create)?)
        .await;

    tracing::info!(actor = %actor_id, "seeded demo actor");
    Ok(())
}
