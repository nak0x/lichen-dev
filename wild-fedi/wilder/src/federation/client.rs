use std::time::Duration;

use anyhow::{Context, anyhow};
use reqwest::header::{ACCEPT, CONTENT_TYPE, DATE, HOST, USER_AGENT};
use rsa::RsaPublicKey;
use serde_json::Value;
use url::Url;

use crate::crypto::{KeyPair, Signer, http_signature};
use crate::repository::LocalActor;
use crate::vocab::actor::RemoteActor;

/// The content type for ActivityPub payloads. We send this and prefer it on
/// fetches; many servers also accept `application/ld+json; profile="..."`.
const ACTIVITY_JSON: &str = "application/activity+json";

pub struct FederationClient {
    http: reqwest::Client,
    user_agent: String,
}

impl FederationClient {
    pub fn new(user_agent: String) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            // Never let one slow peer hang a worker indefinitely.
            .timeout(Duration::from_secs(15))
            .build()
            .context("building HTTP client")?;
        Ok(Self { http, user_agent })
    }

    /// Fetch and parse a remote actor document.
    pub async fn fetch_actor(&self, actor_id: &str) -> anyhow::Result<RemoteActor> {
        let body = self
            .http
            .get(actor_id)
            .header(ACCEPT, ACTIVITY_JSON)
            .header(USER_AGENT, &self.user_agent)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        serde_json::from_str(&body).with_context(|| format!("parsing actor {actor_id}"))
    }

    /// Resolve the RSA public key referenced by a `keyId`.
    ///
    /// A `keyId` is the actor URL plus a `#fragment`; we strip the fragment,
    /// fetch the actor, and read `publicKey.publicKeyPem`. Production servers
    /// cache this aggressively — refetching a key on every inbound POST would be
    /// both slow and a nice amplification vector.
    pub async fn fetch_public_key(&self, key_id: &str) -> anyhow::Result<RsaPublicKey> {
        let actor_url = key_id.split('#').next().unwrap_or(key_id);
        let actor = self.fetch_actor(actor_url).await?;
        let pem = actor
            .public_key
            .ok_or_else(|| anyhow!("actor {actor_url} has no publicKey"))?
            .public_key_pem;
        KeyPair::public_from_pem(&pem)
    }

    /// Sign and POST an activity to a single remote inbox.
    ///
    /// The signature covers `(request-target) host date digest`. The `Digest`
    /// header binds the signature to *these exact bytes*, so a captured
    /// signature cannot be replayed against a different body.
    pub async fn deliver(
        &self,
        sender: &LocalActor,
        inbox_url: &str,
        activity: &Value,
        key_id: &str,
    ) -> anyhow::Result<()> {
        let body = serde_json::to_vec(activity)?;
        let url = Url::parse(inbox_url).with_context(|| format!("bad inbox url {inbox_url}"))?;

        // The `host` we sign must equal the Host header actually sent. Include
        // the port if present (matters for local dev on :8080).
        let host = url
            .host_str()
            .ok_or_else(|| anyhow!("inbox url has no host"))?
            .to_string();
        let host = match url.port() {
            Some(port) => format!("{host}:{port}"),
            None => host,
        };
        // request-target is the path (plus query, if any) exactly as requested.
        let path = match url.query() {
            Some(q) => format!("{}?{}", url.path(), q),
            None => url.path().to_string(),
        };

        let date = httpdate::fmt_http_date(std::time::SystemTime::now());
        let digest = http_signature::digest_header(&body);

        let keypair = KeyPair::from_private_pem(&sender.private_key_pem)?;
        let signer = Signer {
            key_id,
            private_key: keypair.private_key(),
        };
        // This header order must match what the receiver reconstructs; the
        // `headers="..."` field of the Signature header encodes exactly this.
        let signature = signer.sign(&[
            ("(request-target)".to_string(), format!("post {path}")),
            ("host".to_string(), host.clone()),
            ("date".to_string(), date.clone()),
            ("digest".to_string(), digest.clone()),
        ])?;

        let response = self
            .http
            .post(inbox_url)
            .header(HOST, host)
            .header(DATE, date)
            .header("Digest", digest)
            .header(CONTENT_TYPE, ACTIVITY_JSON)
            .header("Signature", signature)
            .header(USER_AGENT, &self.user_agent)
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("delivery to {inbox_url} failed: {status} {text}"));
        }
        Ok(())
    }
}
