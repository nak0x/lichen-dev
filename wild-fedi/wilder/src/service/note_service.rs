use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{Account, NewNote, Note, NoteKind};
use crate::error::AppError;
use crate::repository::NoteRepository;

/// Private personal documents. Every operation is scoped to the caller: a note
/// is only ever readable or writable by its owner, and nothing here federates.
pub struct NoteService {
    notes: Arc<dyn NoteRepository>,
}

impl NoteService {
    pub fn new(notes: Arc<dyn NoteRepository>) -> Self {
        Self { notes }
    }

    pub async fn list(&self, owner: &Account, kind: NoteKind) -> Result<Vec<Note>, AppError> {
        Ok(self.notes.list(&owner.username, kind).await?)
    }

    pub async fn create(&self, owner: &Account, input: NewNote) -> Result<Note, AppError> {
        if input.content.trim().is_empty() {
            return Err(AppError::BadRequest("content is required".to_string()));
        }
        let now = Utc::now();
        let note = Note {
            id: Uuid::new_v4(),
            owner_username: owner.username.clone(),
            kind: input.kind,
            title: input.title,
            content: input.content,
            created_at: now,
            updated_at: now,
        };
        self.notes.insert(&note).await?;
        Ok(note)
    }

    pub async fn update(
        &self,
        owner: &Account,
        id: Uuid,
        title: Option<String>,
        content: String,
    ) -> Result<(), AppError> {
        self.owned(owner, id).await?;
        self.notes.update(id, title.as_deref(), &content).await?;
        Ok(())
    }

    pub async fn delete(&self, owner: &Account, id: Uuid) -> Result<(), AppError> {
        self.owned(owner, id).await?;
        self.notes.delete(id).await?;
        Ok(())
    }

    /// Fetch a note and assert the caller owns it. A note belonging to someone
    /// else is reported as *not found* — we never reveal its existence.
    async fn owned(&self, owner: &Account, id: Uuid) -> Result<Note, AppError> {
        let note = self.notes.by_id(id).await?.ok_or(AppError::NotFound)?;
        if note.owner_username != owner.username {
            return Err(AppError::NotFound);
        }
        Ok(note)
    }
}
