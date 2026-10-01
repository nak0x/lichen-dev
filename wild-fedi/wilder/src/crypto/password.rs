//! Password hashing with Argon2id — the modern memory-hard default. Both the
//! auth service (verify on login) and the account service (hash on creation)
//! share this, so the parameters live in exactly one place.
//!
//! argon2 0.6 generates the salt internally (its `getrandom` default feature),
//! so hashing is a one-liner and no salt plumbing is needed here.

use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};

/// Hash a plaintext password into a PHC string (algorithm, params, salt and
/// digest encoded together), suitable for storing in `accounts.password_hash`.
pub fn hash(password: &str) -> anyhow::Result<String> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes())
        .map_err(|e| anyhow::anyhow!("argon2 hash: {e}"))?
        .to_string())
}

/// Verify a plaintext password against a stored PHC hash. Returns `false` for a
/// mismatch *or* an unparseable hash — callers treat both as "wrong credentials"
/// and must not distinguish them to the client (no oracle).
pub fn verify(password: &str, phc_hash: &str) -> bool {
    match PasswordHash::new(phc_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}
