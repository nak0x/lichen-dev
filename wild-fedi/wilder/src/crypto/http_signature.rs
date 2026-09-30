use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use rsa::{Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};

/// Errors from building or checking an HTTP Signature.
#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("malformed Signature header: {0}")]
    Malformed(String),
    #[error("missing signed header: {0}")]
    MissingHeader(String),
    #[error("signature verification failed")]
    Invalid,
    #[error("crypto error: {0}")]
    Crypto(String),
}

/// Compute the `Digest` header value for a body: `SHA-256=<base64(sha256(body))>`.
///
/// Signing this header (and checking it against the actual bytes on receipt) is
/// what binds a signature to a *specific* payload. Without it, a captured, valid
/// signature could be replayed over a swapped body.
pub fn digest_header(body: &[u8]) -> String {
    let hash = Sha256::digest(body);
    format!("SHA-256={}", STANDARD.encode(hash))
}

/// Build the canonical signing string from ordered `(name, value)` pairs, per the
/// "Signing HTTP Messages" draft everyone in the fediverse implements. Header
/// names are lower-cased; the pseudo-header `(request-target)` carries
/// `"<method> <path>"` (e.g. `post /users/alice/inbox`).
fn signing_string(headers: &[(String, String)]) -> String {
    headers
        .iter()
        .map(|(name, value)| format!("{}: {}", name.to_lowercase(), value))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Signs outbound requests with an actor's private key.
pub struct Signer<'a> {
    pub key_id: &'a str,
    pub private_key: &'a RsaPrivateKey,
}

impl Signer<'_> {
    /// Produce the value for the `Signature` header over the given ordered
    /// headers. The caller chooses which headers to sign; for delivery we sign
    /// `(request-target) host date digest`, the set every major server accepts.
    ///
    /// The order of `headers` is significant and must match the order the
    /// receiver reconstructs, which is why `headers=".."` is echoed verbatim.
    pub fn sign(&self, headers: &[(String, String)]) -> Result<String, SignatureError> {
        let signed_names = headers
            .iter()
            .map(|(name, _)| name.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");

        let string_to_sign = signing_string(headers);
        // Pkcs1v15Sign::new::<Sha256> prepends the SHA-256 DigestInfo prefix, so
        // we feed it the raw 32-byte digest, not the message. This is the
        // "rsa-sha256" scheme named in the Signature header below.
        let hashed = Sha256::digest(string_to_sign.as_bytes());
        let signature = self
            .private_key
            .sign(Pkcs1v15Sign::new::<Sha256>(), &hashed)
            .map_err(|e| SignatureError::Crypto(e.to_string()))?;

        Ok(format!(
            r#"keyId="{}",algorithm="rsa-sha256",headers="{}",signature="{}""#,
            self.key_id,
            signed_names,
            STANDARD.encode(signature),
        ))
    }
}

/// A parsed inbound `Signature` header.
#[derive(Debug, Clone)]
pub struct SignatureHeader {
    pub key_id: String,
    pub algorithm: Option<String>,
    pub headers: Vec<String>, // ordered, lower-cased list of signed header names
    pub signature: String,    // base64
}

impl SignatureHeader {
    /// Parse `keyId="..",algorithm="..",headers="a b c",signature=".."`.
    ///
    /// Splitting on `,` is safe here: `keyId` is a URL, `headers` is
    /// space-separated, and `signature` is base64 — none contain a comma. We
    /// only split each pair on the *first* `=` so base64 padding survives.
    pub fn parse(raw: &str) -> Result<Self, SignatureError> {
        let mut key_id = None;
        let mut algorithm = None;
        let mut headers: Option<Vec<String>> = None;
        let mut signature = None;

        for part in raw.split(',') {
            let part = part.trim();
            let Some((k, v)) = part.split_once('=') else {
                continue;
            };
            let value = v.trim().trim_matches('"');
            match k.trim() {
                "keyId" => key_id = Some(value.to_string()),
                "algorithm" => algorithm = Some(value.to_string()),
                "headers" => {
                    headers = Some(value.split_whitespace().map(str::to_lowercase).collect())
                }
                "signature" => signature = Some(value.to_string()),
                _ => {} // ignore unknown params such as `created` / `expires`
            }
        }

        Ok(Self {
            key_id: key_id.ok_or_else(|| SignatureError::Malformed("missing keyId".into()))?,
            algorithm,
            // An absent `headers` param means just `date`, per the draft.
            headers: headers.unwrap_or_else(|| vec!["date".to_string()]),
            signature: signature
                .ok_or_else(|| SignatureError::Malformed("missing signature".into()))?,
        })
    }
}

/// Verify a parsed signature against a received request.
///
/// `lookup` returns the actual value of a (lower-cased) header from the request;
/// the pseudo `(request-target)` is synthesised from `method` and `path`. We
/// rebuild the signing string in the exact order the sender declared, so any
/// tampering with a signed header — or the method/path — fails the check.
pub fn verify(
    public_key: &RsaPublicKey,
    parsed: &SignatureHeader,
    method: &str,
    path: &str,
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<(), SignatureError> {
    // We implement rsa-sha256 (also named `hs2019` when the key is RSA). Reject
    // any other declared algorithm up front, rather than letting it fail later
    // as a confusing "bad signature".
    if let Some(alg) = &parsed.algorithm {
        let alg = alg.to_ascii_lowercase();
        if alg != "rsa-sha256" && alg != "hs2019" {
            return Err(SignatureError::Crypto(format!("unsupported algorithm: {alg}")));
        }
    }

    let mut reconstructed = Vec::with_capacity(parsed.headers.len());
    for name in &parsed.headers {
        let value = if name == "(request-target)" {
            format!("{} {}", method.to_lowercase(), path)
        } else {
            lookup(name).ok_or_else(|| SignatureError::MissingHeader(name.clone()))?
        };
        reconstructed.push((name.clone(), value));
    }

    let string_to_sign = signing_string(&reconstructed);
    let hashed = Sha256::digest(string_to_sign.as_bytes());
    let signature = STANDARD
        .decode(parsed.signature.as_bytes())
        .map_err(|e| SignatureError::Malformed(format!("signature not base64: {e}")))?;

    public_key
        .verify(Pkcs1v15Sign::new::<Sha256>(), &hashed, &signature)
        .map_err(|_| SignatureError::Invalid)
}
