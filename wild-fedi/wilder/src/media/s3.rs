use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;

use super::{MediaStore, StoredObject};
use crate::config::S3Config;

/// S3-compatible media store, used against MinIO. Path-style addressing is
/// required (MinIO does not do virtual-host buckets by default).
pub struct S3Media {
    client: Client,
    bucket: String,
}

impl S3Media {
    pub fn new(cfg: &S3Config) -> Self {
        let creds = Credentials::new(&cfg.access_key, &cfg.secret_key, None, None, "static");
        let conf = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(cfg.region.clone()))
            .endpoint_url(&cfg.endpoint)
            .credentials_provider(creds)
            .force_path_style(true)
            .build();
        Self {
            client: Client::from_conf(conf),
            bucket: cfg.bucket.clone(),
        }
    }

    /// Ensure the bucket exists, retrying while MinIO finishes starting (compose
    /// gates us on "started", not "healthy", so the first calls may race boot).
    pub async fn ensure_bucket(&self) -> anyhow::Result<()> {
        let mut last_err = None;
        for attempt in 1..=15 {
            match self.try_ensure_bucket().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    tracing::debug!(attempt, error = %e, "waiting for object store");
                    last_err = Some(e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
        Err(last_err.unwrap()).context("object store never became ready")
    }

    async fn try_ensure_bucket(&self) -> anyhow::Result<()> {
        if self
            .client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .is_ok()
        {
            return Ok(());
        }
        // Not present (or not reachable yet): attempt to create it. A concurrent
        // create or an "owned by you" response is success.
        match self.client.create_bucket().bucket(&self.bucket).send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = format!("{e:?}");
                if msg.contains("BucketAlreadyOwnedByYou") || msg.contains("BucketAlreadyExists") {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("create bucket: {e}"))
                }
            }
        }
    }
}

#[async_trait]
impl MediaStore for S3Media {
    async fn put(&self, key: &str, content_type: &str, bytes: Vec<u8>) -> anyhow::Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .context("s3 put_object")?;
        Ok(())
    }

    async fn get(&self, key: &str) -> anyhow::Result<StoredObject> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .context("s3 get_object")?;
        let content_type = resp
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = resp.body.collect().await.context("s3 read body")?.into_bytes();
        Ok(StoredObject {
            content_type,
            bytes: bytes.to_vec(),
        })
    }
}
