//! Blob storage port. Media *bytes* live here (MinIO/S3); their *metadata*
//! lives in Postgres. The trait keeps the rest of the app ignorant of S3 — a
//! local-filesystem or other backend could implement it without any caller
//! change (DIP), mirroring the repository traits.

pub mod fs;
pub mod s3;

use async_trait::async_trait;

pub use fs::FilesystemMedia;
pub use s3::S3Media;

/// An object retrieved from storage: its bytes and the content type to serve.
pub struct StoredObject {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[async_trait]
pub trait MediaStore: Send + Sync {
    /// Store bytes under `key` with the given content type (overwrites).
    async fn put(&self, key: &str, content_type: &str, bytes: Vec<u8>) -> anyhow::Result<()>;
    /// Fetch an object's bytes and content type.
    async fn get(&self, key: &str) -> anyhow::Result<StoredObject>;
}
