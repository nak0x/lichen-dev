//! Filesystem-backed media store: bytes under a directory, content type in a
//! sidecar file. The zero-dependency alternative to MinIO — handy for a
//! single-box / Raspberry-Pi deployment, and what the tests run against. The S3
//! store remains the default; this is chosen with `MEDIA_BACKEND=fs`.

use std::path::{Path, PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use tokio::fs;

use super::{MediaStore, StoredObject};

pub struct FilesystemMedia {
    root: PathBuf,
}

impl FilesystemMedia {
    pub async fn new(root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)
            .await
            .with_context(|| format!("creating media dir {}", root.display()))?;
        Ok(Self { root })
    }

    fn path(&self, key: &str) -> PathBuf {
        self.root.join(key)
    }
}

#[async_trait]
impl MediaStore for FilesystemMedia {
    async fn put(&self, key: &str, content_type: &str, bytes: Vec<u8>) -> anyhow::Result<()> {
        let path = self.path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&path, &bytes).await?;
        // Sidecar carries the content type so `get` can serve it back.
        fs::write(path.with_extension("ct"), content_type).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> anyhow::Result<StoredObject> {
        let path = self.path(key);
        let bytes = fs::read(&path)
            .await
            .with_context(|| format!("reading media {key}"))?;
        let content_type = fs::read_to_string(path.with_extension("ct"))
            .await
            .unwrap_or_else(|_| "application/octet-stream".to_string());
        Ok(StoredObject {
            content_type,
            bytes,
        })
    }
}
