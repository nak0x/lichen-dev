use std::sync::{Arc, OnceLock};

use crate::cache::SessionStore;
use crate::crypto::password;
use crate::domain::Account;
use crate::repository::AccountRepository;

/// A real argon2 hash of a fixed string, computed once. Verifying against it when
/// the username is unknown spends argon2 time comparable to a real check, so a
/// caller cannot distinguish "no such user" from "wrong password" by timing.
fn dummy_hash() -> &'static str {
    static HASH: OnceLock<String> = OnceLock::new();
    HASH.get_or_init(|| password::hash("wilder-timing-dummy").unwrap_or_default())
}

/// Authentication: verifying credentials and minting/resolving sessions. It owns
/// neither account creation (that is `AccountService`) nor authorization (that is
/// the web role guard) — only "who is this request, if anyone".
pub struct AuthService {
    accounts: Arc<dyn AccountRepository>,
    sessions: Arc<SessionStore>,
}

impl AuthService {
    pub fn new(accounts: Arc<dyn AccountRepository>, sessions: Arc<SessionStore>) -> Self {
        Self { accounts, sessions }
    }

    /// Verify a username/password and, on success, issue a session token.
    /// Returns `Ok(None)` for *any* bad-credential case — unknown user or wrong
    /// password are indistinguishable to the caller (no account-enumeration
    /// oracle). We still run a verify on a dummy hash when the user is unknown to
    /// keep the timing roughly constant.
    pub async fn login(
        &self,
        username: &str,
        plaintext: &str,
    ) -> anyhow::Result<Option<(Account, String)>> {
        let account = self.accounts.by_username(username).await?;
        let ok = match &account {
            Some(acc) => password::verify(plaintext, &acc.password_hash),
            None => {
                // Constant-ish time: verify against a throwaway hash, ignore result.
                let _ = password::verify(plaintext, dummy_hash());
                false
            }
        };
        match (ok, account) {
            (true, Some(acc)) => {
                let token = self.sessions.create(acc.id).await?;
                Ok(Some((acc, token)))
            }
            _ => Ok(None),
        }
    }

    /// Open a session for an account without a password check — used right after
    /// provisioning (e.g. school bootstrap) so the new account is logged in.
    pub async fn start_session(&self, account_id: uuid::Uuid) -> anyhow::Result<String> {
        self.sessions.create(account_id).await
    }

    pub async fn logout(&self, token: &str) -> anyhow::Result<()> {
        self.sessions.revoke(token).await
    }

    /// Resolve a session token to its account, or `None` if the session is
    /// unknown/expired or the account has since been removed.
    pub async fn account_for_token(&self, token: &str) -> anyhow::Result<Option<Account>> {
        match self.sessions.lookup(token).await? {
            Some(id) => self.accounts.by_id(id).await,
            None => Ok(None),
        }
    }
}
