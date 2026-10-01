use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::crypto::{KeyPair, password};
use crate::domain::{Account, NewAccount, Role};
use crate::error::AppError;
use crate::repository::{AccountRepository, ActorRepository, LocalActor};

/// Provisioning of accounts and their ActivityPub actors. Every account owns a
/// local actor (with a fresh keypair), created atomically-ish here so identity
/// and credentials always come into being together.
pub struct AccountService {
    config: Arc<AppConfig>,
    actors: Arc<dyn ActorRepository>,
    accounts: Arc<dyn AccountRepository>,
}

impl AccountService {
    pub fn new(
        config: Arc<AppConfig>,
        actors: Arc<dyn ActorRepository>,
        accounts: Arc<dyn AccountRepository>,
    ) -> Self {
        Self {
            config,
            actors,
            accounts,
        }
    }

    /// True when the instance has no accounts yet — the only moment a school may
    /// self-register (first-run bootstrap). Afterwards the instance is closed.
    pub async fn bootstrap_available(&self) -> anyhow::Result<bool> {
        Ok(self.accounts.count().await? == 0)
    }

    /// Create the very first account: a `School` that administers the instance.
    /// Refuses once any account exists, so the instance cannot be hijacked later.
    pub async fn bootstrap_school(&self, input: NewAccount) -> Result<Account, AppError> {
        if !self
            .bootstrap_available()
            .await
            .map_err(AppError::Internal)?
        {
            return Err(AppError::BadRequest(
                "instance already initialised".to_string(),
            ));
        }
        self.provision(Role::School, input, None).await
    }

    /// A school provisions a student account. Authorization (caller is a school)
    /// is enforced by the web guard, but we re-check here so the invariant holds
    /// regardless of entry point — defence in depth.
    pub async fn create_student(
        &self,
        creator: &Account,
        input: NewAccount,
    ) -> Result<Account, AppError> {
        if !creator.role.can_administer_instance() {
            return Err(AppError::Forbidden(
                "only a school may create accounts".to_string(),
            ));
        }
        self.provision(Role::Student, input, Some(creator.id)).await
    }

    /// A school provisions a capture station (a `Service` actor). Its posts feed
    /// the "captured" view. Same authorization as student creation.
    pub async fn create_station(
        &self,
        creator: &Account,
        input: NewAccount,
    ) -> Result<Account, AppError> {
        if !creator.role.can_administer_instance() {
            return Err(AppError::Forbidden(
                "only a school may create accounts".to_string(),
            ));
        }
        self.provision(Role::Service, input, Some(creator.id)).await
    }

    /// Shared creation path: validate, mint actor + keypair, hash password, persist.
    async fn provision(
        &self,
        role: Role,
        input: NewAccount,
        created_by: Option<Uuid>,
    ) -> Result<Account, AppError> {
        let username = normalise_username(&input.username)?;
        if input.password.len() < 8 {
            return Err(AppError::BadRequest(
                "password must be at least 8 characters".to_string(),
            ));
        }
        if self
            .accounts
            .by_username(&username)
            .await
            .map_err(AppError::Internal)?
            .is_some()
        {
            return Err(AppError::Conflict(format!("username '{username}' is taken")));
        }

        // Mint the actor first; the account references it by username.
        let keypair = KeyPair::generate().map_err(AppError::Internal)?;
        self.actors
            .insert(LocalActor {
                username: username.clone(),
                kind: role.actor_kind(),
                display_name: input.display_name.clone(),
                summary: None,
                private_key_pem: keypair.private_pem().map_err(AppError::Internal)?,
                public_key_pem: keypair.public_pem().map_err(AppError::Internal)?,
            })
            .await
            .map_err(AppError::Internal)?;

        let account = Account {
            id: Uuid::new_v4(),
            username,
            role,
            display_name: input.display_name,
            email: input.email,
            password_hash: password::hash(&input.password).map_err(AppError::Internal)?,
            created_by,
            created_at: Utc::now(),
        };
        self.accounts
            .insert(&account)
            .await
            .map_err(AppError::Internal)?;

        tracing::info!(username = %account.username, role = ?role, "provisioned account");
        let _ = &self.config; // reserved for future URL/profile defaults
        Ok(account)
    }

    pub async fn roster(&self, school_id: Uuid) -> anyhow::Result<Vec<Account>> {
        self.accounts.list_created_by(school_id).await
    }
}

/// Usernames become part of actor URLs and WebFinger handles, so constrain them
/// to a safe, predictable shape rather than trusting arbitrary input. Shared with
/// group creation (group slugs are usernames too).
pub(crate) fn normalise_username(raw: &str) -> Result<String, AppError> {
    let name = raw.trim().to_lowercase();
    let valid = !name.is_empty()
        && name.len() <= 32
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if valid {
        Ok(name)
    } else {
        Err(AppError::BadRequest(
            "username must be 1–32 chars of letters, digits, '-' or '_'".to_string(),
        ))
    }
}
