use std::sync::Arc;

use chrono::Utc;

use crate::config::AppConfig;
use crate::crypto::KeyPair;
use crate::domain::{Account, Group, Member, MembershipRole, NewGroup};
use crate::error::AppError;
use crate::repository::{ActorRepository, GroupRepository, LocalActor};
use crate::service::account_service::normalise_username;

/// Group lifecycle: creation (by a school *or* a student — both may start a
/// study circle), membership, and the admin predicate. A group is a `Group`
/// actor, so creating one mints an actor + keypair exactly like an account.
pub struct GroupService {
    config: Arc<AppConfig>,
    actors: Arc<dyn ActorRepository>,
    groups: Arc<dyn GroupRepository>,
}

impl GroupService {
    pub fn new(
        config: Arc<AppConfig>,
        actors: Arc<dyn ActorRepository>,
        groups: Arc<dyn GroupRepository>,
    ) -> Self {
        Self {
            config,
            actors,
            groups,
        }
    }

    /// Create a group; the creator becomes its admin. Any authenticated account
    /// may do this — the README has classes *and* students starting circles.
    pub async fn create(&self, creator: &Account, input: NewGroup) -> Result<Group, AppError> {
        let username = normalise_username(&input.username)?;
        if self.actors.get(&username).await?.is_some() {
            return Err(AppError::Conflict(format!("name '{username}' is taken")));
        }

        let keypair = KeyPair::generate()?;
        self.actors
            .insert(LocalActor {
                username: username.clone(),
                kind: crate::vocab::ActorType::Group,
                display_name: Some(input.name.clone()),
                summary: input.summary.clone().or_else(|| input.subject.clone()),
                private_key_pem: keypair.private_pem()?,
                public_key_pem: keypair.public_pem()?,
                icon_url: None,
            })
            .await?;

        let group = Group {
            username: username.clone(),
            name: input.name,
            summary: input.summary,
            subject: input.subject,
            created_by: Some(creator.id),
            created_at: Utc::now(),
        };
        self.groups.insert(&group).await?;
        self.groups
            .add_member(&username, creator.id, MembershipRole::Admin)
            .await?;

        tracing::info!(group = %username, by = %creator.username, "created group");
        let _ = &self.config;
        Ok(group)
    }

    pub async fn get(&self, username: &str) -> Result<Group, AppError> {
        self.groups
            .by_username(username)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn all(&self) -> Result<Vec<Group>, AppError> {
        Ok(self.groups.all().await?)
    }

    pub async fn for_account(&self, account_id: uuid::Uuid) -> Result<Vec<Group>, AppError> {
        Ok(self.groups.for_account(account_id).await?)
    }

    pub async fn members(&self, username: &str) -> Result<Vec<Member>, AppError> {
        // 404 for a non-existent group rather than an empty list.
        self.get(username).await?;
        Ok(self.groups.members(username).await?)
    }

    pub async fn join(&self, account: &Account, username: &str) -> Result<(), AppError> {
        self.get(username).await?;
        self.groups
            .add_member(username, account.id, MembershipRole::Member)
            .await?;
        Ok(())
    }

    pub async fn leave(&self, account: &Account, username: &str) -> Result<(), AppError> {
        self.groups.remove_member(username, account.id).await?;
        Ok(())
    }

    /// The caller's role in a group, or `Forbidden` if they are not a member —
    /// the guard posting into a group goes through.
    pub async fn require_member(
        &self,
        username: &str,
        account_id: uuid::Uuid,
    ) -> Result<MembershipRole, AppError> {
        self.groups
            .role_of(username, account_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("not a member of this group".to_string()))
    }

    /// An admin removes another member. The caller must be an admin; admins
    /// remove themselves by leaving, not here.
    pub async fn remove_member(
        &self,
        admin: &Account,
        username: &str,
        target: uuid::Uuid,
    ) -> Result<(), AppError> {
        if self.require_member(username, admin.id).await? != MembershipRole::Admin {
            return Err(AppError::Forbidden("admin only".to_string()));
        }
        if target == admin.id {
            return Err(AppError::BadRequest(
                "use leave to remove yourself".to_string(),
            ));
        }
        self.groups.remove_member(username, target).await?;
        Ok(())
    }
}
