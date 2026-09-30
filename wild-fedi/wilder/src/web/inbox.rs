use axum::body::Bytes;
use axum::extract::{OriginalUri, Path, State};
use axum::http::{HeaderMap, Method, StatusCode};
use serde_json::Value;

use crate::crypto::{SignatureHeader, http_signature, verify};
use crate::error::AppError;
use crate::queue::Job;
use crate::state::AppState;
use crate::vocab::IncomingActivity;

/// `POST /users/:name/inbox` (README §4, contract 3).
///
/// The trust boundary of the whole server. We do the **minimum synchronous
/// work** — verify the HTTP signature and body digest — then hand the trusted
/// payload to the queue and return `202`. Everything heavier (fetching the
/// follower actor, delivering an Accept, storage writes) runs in a worker, so a
/// sender is never kept waiting and a burst cannot exhaust the runtime.
pub async fn inbox(
    State(state): State<AppState>,
    Path(name): Path<String>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    // Cheap existence check before we spend effort verifying a signature.
    if state.actors.get(&name).await.is_none() {
        return Err(AppError::NotFound);
    }
    ingest(&state, &method, uri.path(), &headers, &body, Some(name)).await
}

/// `POST /inbox` — the shared inbox. Senders batch-deliver here; recipients are
/// derived from the activity's addressing.
pub async fn shared_inbox(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    ingest(&state, &method, uri.path(), &headers, &body, None).await
}

/// Shared inbox pipeline for both the personal and shared endpoints.
async fn ingest(
    state: &AppState,
    method: &Method,
    path: &str,
    headers: &HeaderMap,
    body: &Bytes,
    explicit_recipient: Option<String>,
) -> Result<StatusCode, AppError> {
    // 1. Parse the Signature header.
    let sig_header = headers
        .get("signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("missing Signature header".into()))?;
    let parsed = SignatureHeader::parse(sig_header)
        .map_err(|e| AppError::Unauthorized(e.to_string()))?;

    // 2. A POST signature MUST cover the body digest, otherwise the body is
    //    unauthenticated (a valid signature could be replayed over new bytes).
    //    Enforce it, then check the digest against what we actually received.
    if !parsed.headers.iter().any(|h| h == "digest") {
        return Err(AppError::Unauthorized(
            "signature does not cover the digest header".into(),
        ));
    }
    let sent_digest = header_value(headers, "digest")
        .ok_or_else(|| AppError::Unauthorized("missing Digest header".into()))?;
    if !digest_matches(&sent_digest, &http_signature::digest_header(body)) {
        return Err(AppError::Unauthorized("digest mismatch".into()));
    }

    // 3. Resolve the signer's public key (keyId → actor → publicKeyPem) and
    //    verify the signature over the exact headers the sender declared.
    let public_key = state
        .federation
        .fetch_public_key(&parsed.key_id)
        .await
        .map_err(|e| AppError::Unauthorized(format!("cannot resolve signer key: {e}")))?;
    verify(&public_key, &parsed, method.as_str(), path, |name| {
        header_value(headers, name)
    })
    .map_err(|e| AppError::Unauthorized(e.to_string()))?;

    // 4. Parse the activity. A signature from key K proves the request came from
    //    K's owner; we still require that owner to match the activity's `actor`,
    //    so a valid signer cannot post *as someone else*.
    let value: Value = serde_json::from_slice(body)
        .map_err(|e| AppError::BadRequest(format!("invalid activity JSON: {e}")))?;
    let activity: IncomingActivity = serde_json::from_value(value.clone())
        .map_err(|e| AppError::BadRequest(format!("unrecognised activity: {e}")))?;
    let sender_id = activity
        .actor_id()
        .ok_or_else(|| AppError::BadRequest("activity has no actor".into()))?;
    let key_owner = parsed.key_id.split('#').next().unwrap_or(&parsed.key_id);
    if sender_id != key_owner {
        return Err(AppError::Unauthorized(
            "signer does not match activity actor".into(),
        ));
    }

    // 5. Fan out one processing job per local recipient, then 202. The empty
    //    case (a shared-inbox delivery addressed only to collections we can't
    //    expand) is logged, not errored — the sender did nothing wrong.
    let recipients = match explicit_recipient {
        Some(name) => vec![name],
        None => resolve_local_recipients(state, &activity).await,
    };
    if recipients.is_empty() {
        tracing::debug!(sender = %sender_id, "inbox delivery matched no local recipients");
    }
    for recipient in recipients {
        state
            .queue
            .enqueue(Job::ProcessInbox {
                recipient,
                sender_id: sender_id.clone(),
                activity: value.clone(),
            })
            .await;
    }

    // 202 Accepted: taken for asynchronous processing, outcome not yet known —
    // the spec-correct response for a queued inbox.
    Ok(StatusCode::ACCEPTED)
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

/// Compare digest headers. Both look like `SHA-256=<b64>`; some servers send a
/// comma-separated list of algorithms. Match the algorithm token
/// case-insensitively and the encoded value exactly.
fn digest_matches(sent: &str, expected: &str) -> bool {
    let Some((exp_algo, exp_val)) = expected.split_once('=') else {
        return false;
    };
    sent.split(',').any(|part| match part.trim().split_once('=') {
        Some((algo, val)) => algo.eq_ignore_ascii_case(exp_algo) && val == exp_val,
        None => false,
    })
}

/// For the shared inbox, work out which *local* actors an activity is for by
/// scanning its addressing for URLs under our own `/users/` path.
///
/// Limitation (noted deliberately): we resolve only directly-addressed actor
/// IRIs, not collection addresses like a `followers` URL. Expanding those would
/// mean knowing which local users follow the sender — a fuller server's job.
async fn resolve_local_recipients(state: &AppState, activity: &IncomingActivity) -> Vec<String> {
    let base = state.config.actor_url(""); // ".../users/"
    let mut names = Vec::new();
    for field in ["to", "cc", "bto", "bcc", "audience"] {
        if let Some(value) = activity.extra.get(field) {
            collect_addressed(value, &base, &mut names);
        }
    }

    let mut resolved = Vec::new();
    for name in names {
        if !resolved.contains(&name) && state.actors.get(&name).await.is_some() {
            resolved.push(name);
        }
    }
    resolved
}

/// Collect the trailing `{name}` of any addressing value that is one of our
/// actor URLs (`{base}/users/{name}`), recursing into arrays.
fn collect_addressed(value: &Value, base: &str, out: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            if let Some(rest) = s.strip_prefix(base) {
                // Exactly a username — not a sub-path like `alice/followers`.
                if !rest.is_empty() && !rest.contains('/') {
                    out.push(rest.to_string());
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_addressed(item, base, out);
            }
        }
        _ => {}
    }
}
