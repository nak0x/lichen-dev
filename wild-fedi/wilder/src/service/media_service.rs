use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::domain::{Account, MediaRef};
use crate::error::AppError;
use crate::media::{MediaStore, StoredObject};
use crate::repository::{ActorRepository, MediaRepository};

/// Largest media we accept (8 MiB) — field photos, not raw video.
const MAX_BYTES: usize = 8 * 1024 * 1024;

/// Uploading, serving and attaching media. Bytes go to the `MediaStore`, a
/// metadata row to Postgres, and everything is served back *through* wilder so
/// the browser never reaches the object store directly.
pub struct MediaService {
    config: Arc<AppConfig>,
    store: Arc<dyn MediaStore>,
    media: Arc<dyn MediaRepository>,
    actors: Arc<dyn ActorRepository>,
}

impl MediaService {
    pub fn new(
        config: Arc<AppConfig>,
        store: Arc<dyn MediaStore>,
        media: Arc<dyn MediaRepository>,
        actors: Arc<dyn ActorRepository>,
    ) -> Self {
        Self {
            config,
            store,
            media,
            actors,
        }
    }

    /// Store an uploaded image and return its metadata. Rejects non-images and
    /// oversized payloads before touching storage.
    pub async fn upload(
        &self,
        owner: &Account,
        content_type: &str,
        bytes: Vec<u8>,
    ) -> Result<MediaRef, AppError> {
        if !content_type.starts_with("image/") {
            return Err(AppError::BadRequest("only images are accepted".to_string()));
        }
        if bytes.is_empty() || bytes.len() > MAX_BYTES {
            return Err(AppError::BadRequest(format!(
                "image must be 1 byte–{} MiB",
                MAX_BYTES / (1024 * 1024)
            )));
        }

        let id = Uuid::new_v4();
        // Key namespaced by owner for easy inspection; id guarantees uniqueness.
        let key = format!("{}/{}", owner.username, id);
        self.store.put(&key, content_type, bytes.clone()).await?;

        let media = MediaRef {
            id,
            owner_username: owner.username.clone(),
            object_key: key,
            content_type: content_type.to_string(),
            byte_size: bytes.len() as i64,
            created_at: Utc::now(),
        };
        self.media.insert(&media).await?;
        Ok(media)
    }

    /// Fetch bytes to serve for a public media id.
    pub async fn fetch(&self, id: Uuid) -> Result<StoredObject, AppError> {
        let media = self.media.by_id(id).await?.ok_or(AppError::NotFound)?;
        Ok(self.store.get(&media.object_key).await?)
    }

    /// The public URL a media id is served at.
    pub fn url_for(&self, id: Uuid) -> String {
        self.config.media_url(&id.to_string())
    }

    /// Set the caller's avatar from an uploaded image.
    pub async fn set_avatar(
        &self,
        owner: &Account,
        content_type: &str,
        bytes: Vec<u8>,
    ) -> Result<String, AppError> {
        let media = self.upload(owner, content_type, bytes).await?;
        let url = self.url_for(media.id);
        self.actors.set_icon(&owner.username, &url).await?;
        Ok(url)
    }
}
