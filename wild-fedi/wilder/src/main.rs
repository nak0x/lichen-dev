//! wilder — a bare, queue-backed ActivityPub server for the LICHEN federated
//! network (see ../README.md, Path B).
//!
//! This file is the composition root: the *only* place that knows the concrete
//! implementations. It wires trait objects together and starts the two runtimes
//! that make up the server — the HTTP listener and the background worker pool.

mod cache;
mod config;
mod crypto;
mod domain;
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

use crate::cache::SessionStore;
use crate::config::AppConfig;
use crate::crypto::KeyPair;
use crate::federation::FederationClient;
use crate::processor::AppJobProcessor;
use crate::queue::redis::build_pool;
use crate::queue::{JobQueue, RedisQueue, RedisWorkerPool};
use crate::repository::postgres::PgStore;
use crate::repository::{
    AccountRepository, ActivityRepository, ActorRepository, FollowerRepository, LocalActor,
};
use crate::service::{AccountService, ActorService, AuthService, InboxService, PublishService};
use crate::state::AppState;
use crate::vocab::{Activity, ActorType, Object};

/// Name of the Redis Stream holding federation jobs, and its consumer group.
const JOB_STREAM: &str = "wilder:jobs";
const JOB_GROUP: &str = "wilder-workers";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load a local .env in development (no-op when absent, e.g. in the container).
    dotenvy::dotenv().ok();

    // Structured logs; RUST_LOG overrides the default.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,wilder=debug")),
        )
        .init();

    let config = Arc::new(AppConfig::from_env()?);
    tracing::info!(domain = %config.domain, "starting wilder");

    // --- storage: Postgres source of truth behind the three repository traits ---
    // One concrete `PgStore` implements all of them; DIP keeps callers unaware.
    let store = Arc::new(
        PgStore::connect(&config.database_url)
            .await
            .context("connecting to Postgres")?,
    );
    store.migrate().await.context("running migrations")?;
    let actors: Arc<dyn ActorRepository> = store.clone();
    let followers: Arc<dyn FollowerRepository> = store.clone();
    let activities: Arc<dyn ActivityRepository> = store.clone();
    let accounts: Arc<dyn AccountRepository> = store.clone();

    // --- work queue: durable Redis Stream; build the producer side first ---
    let redis = build_pool(&config.redis_url, config.worker_count + 16)
        .context("building Redis pool")?;
    let queue: Arc<dyn JobQueue> =
        Arc::new(RedisQueue::new(redis.clone(), JOB_STREAM, config.queue_capacity));

    // --- identity: Redis sessions + argon2 auth + account provisioning ---
    let sessions = Arc::new(SessionStore::new(redis.clone()));
    let auth = Arc::new(AuthService::new(Arc::clone(&accounts), Arc::clone(&sessions)));
    let account_service = Arc::new(AccountService::new(
        Arc::clone(&config),
        Arc::clone(&actors),
        Arc::clone(&accounts),
    ));

    // Seed a demo actor on first boot (Postgres now persists across restarts, so
    // this is idempotent — see `seed_demo_actor`). Publishes through the same
    // PublishService a real endpoint would use.
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

    // --- worker pool: consumes the Redis Stream via the processor adapter ---
    let processor = Arc::new(AppJobProcessor::new(
        Arc::clone(&config),
        Arc::clone(&inbox_service),
        Arc::clone(&federation),
        Arc::clone(&actors),
    ));
    let _workers = RedisWorkerPool::spawn(
        config.worker_count,
        redis.clone(),
        JOB_STREAM,
        JOB_GROUP,
        processor,
    )
    .await?;

    // --- HTTP server ---
    let state = AppState {
        config: Arc::clone(&config),
        actors: Arc::clone(&actors),
        actor_service,
        federation,
        queue,
        auth,
        account_service,
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
    // Idempotent: with a durable store, only seed when the actor is absent, so a
    // restart neither duplicates the sample post nor rotates the actor's key.
    if actors.get(username).await?.is_some() {
        tracing::debug!(actor = username, "demo actor already present; skipping seed");
        return Ok(());
    }
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
        .await?;

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
            "name": "Rivière, en aval du pont",
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
        .await?;

    tracing::info!(actor = %actor_id, "seeded demo actor");
    Ok(())
}
