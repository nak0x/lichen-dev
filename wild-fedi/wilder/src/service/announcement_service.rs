use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::domain::{Account, Announcement, AnnouncementKind, NewAnnouncement};
use crate::error::AppError;
use crate::repository::AnnouncementRepository;
use crate::service::PublishService;
use crate::vocab::{self, Activity};

/// Publishing a school's events and news. Each becomes a `Create` wrapping an AS
/// `Event` or `Article`, persisted for listing and federated through the shared
/// `PublishService` (outbox + delivery to the school's followers).
pub struct AnnouncementService {
    config: Arc<AppConfig>,
    announcements: Arc<dyn AnnouncementRepository>,
    publisher: Arc<PublishService>,
}

impl AnnouncementService {
    pub fn new(
        config: Arc<AppConfig>,
        announcements: Arc<dyn AnnouncementRepository>,
        publisher: Arc<PublishService>,
    ) -> Self {
        Self {
            config,
            announcements,
            publisher,
        }
    }

    /// Publish an event or news item. Only a school may do this (the web guard
    /// enforces it; we re-check here as defence in depth).
    pub async fn publish(
        &self,
        school: &Account,
        input: NewAnnouncement,
    ) -> Result<Announcement, AppError> {
        if !school.role.can_administer_instance() {
            return Err(AppError::Forbidden(
                "only a school may publish events or news".to_string(),
            ));
        }
        if input.title.trim().is_empty() || input.content.trim().is_empty() {
            return Err(AppError::BadRequest("title and content are required".to_string()));
        }

        let now = Utc::now();
        let published = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let actor_url = self.config.actor_url(&school.username);
        let followers = self.config.followers_url(&school.username);
        let object_id = self.config.object_url(&Uuid::new_v4().to_string());

        // Build the AS object: Event (with time/place) or Article.
        let mut object = serde_json::json!({
            "type": input.kind.as_object_type(),
            "id": object_id,
            "attributedTo": actor_url,
            "name": input.title,
            "content": input.content,
            "published": published,
            "to": [vocab::PUBLIC],
            "cc": [followers],
        });
        if input.kind == AnnouncementKind::Event {
            if let Some(start) = input.start_time {
                object["startTime"] =
                    serde_json::json!(start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
            }
            if let Some(loc) = &input.location {
                object["location"] = serde_json::json!({ "type": "Place", "name": loc });
            }
        }

        let activity = Activity {
            context: vocab::activity_context(),
            kind: "Create".to_string(),
            id: self.config.activity_url(&Uuid::new_v4().to_string()),
            actor: actor_url,
            to: vec![vocab::PUBLIC.to_string()],
            cc: vec![followers],
            published: Some(published),
            object: object.clone(),
        };
        let activity_value = serde_json::to_value(&activity)?;

        let announcement = Announcement {
            id: Uuid::new_v4(),
            kind: input.kind,
            author_username: school.username.clone(),
            title: input.title,
            content: input.content,
            start_time: input.start_time,
            location: input.location,
            object_id,
            activity: activity_value.clone(),
            published: now,
        };
        self.announcements.insert(&announcement).await?;
        // Outbox + federation to the school's followers.
        self.publisher
            .publish(&school.username, activity_value)
            .await?;

        tracing::info!(kind = ?input.kind, by = %school.username, "published announcement");
        Ok(announcement)
    }

    pub async fn list(&self, kind: AnnouncementKind) -> Result<Vec<Announcement>, AppError> {
        Ok(self.announcements.list(kind, 100).await?)
    }

    pub async fn count(&self, kind: AnnouncementKind) -> Result<usize, AppError> {
        Ok(self.announcements.count(kind).await?)
    }
}
