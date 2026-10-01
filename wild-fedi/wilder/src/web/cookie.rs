//! Minimal cookie handling for the session token. Hand-rolled (no cookie crate)
//! because we only need one cookie and want its attributes in plain sight — the
//! same "own the security-critical bits" stance as the HTTP-signature code.

use axum::http::HeaderMap;

/// Name of the session cookie. Opaque token inside; see `cache::session`.
pub const SESSION_COOKIE: &str = "wilder_session";

/// Read a named cookie from the request's `Cookie` header, if present.
pub fn read(headers: &HeaderMap, name: &str) -> Option<String> {
    let raw = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    raw.split(';').find_map(|pair| {
        let (k, v) = pair.trim().split_once('=')?;
        (k == name).then(|| v.to_string())
    })
}

/// Build a `Set-Cookie` value that stores the session token.
///
/// `HttpOnly` keeps it away from JS (XSS can't read it); `SameSite=Lax` blocks
/// cross-site POSTs from riding the cookie; `Secure` is set only under HTTPS so
/// local http development still works. `Path=/` so every API route sees it.
pub fn set_session(token: &str, secure: bool, max_age_secs: i64) -> String {
    let mut c = format!(
        "{SESSION_COOKIE}={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age_secs}"
    );
    if secure {
        c.push_str("; Secure");
    }
    c
}

/// Build a `Set-Cookie` value that clears the session (logout).
pub fn clear_session(secure: bool) -> String {
    let mut c =
        format!("{SESSION_COOKIE}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0");
    if secure {
        c.push_str("; Secure");
    }
    c
}
