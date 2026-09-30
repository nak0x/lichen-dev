use anyhow::Context;
use rsa::pkcs8::{
    DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding,
};
use rsa::{RsaPrivateKey, RsaPublicKey};

/// 2048 bits: the de-facto floor across the fediverse and what Mastodon emits.
const KEY_BITS: usize = 2048;

/// An RSA keypair backing one actor.
///
/// A fediverse signature is only as trustworthy as the private key's secrecy, so
/// in production the private PEM belongs in a secret manager, not next to the
/// actor row. This prototype keeps it in the actor record for simplicity — the
/// abstraction boundary (the repository) is where you'd change that.
#[derive(Clone)]
pub struct KeyPair {
    private: RsaPrivateKey,
    public: RsaPublicKey,
}

impl KeyPair {
    /// Generate a fresh 2048-bit RSA keypair. This is CPU-heavy (tens of ms), so
    /// we do it once when an actor is created — never on a request path.
    pub fn generate() -> anyhow::Result<Self> {
        let mut rng = rand::thread_rng();
        let private = RsaPrivateKey::new(&mut rng, KEY_BITS).context("generating RSA key")?;
        let public = RsaPublicKey::from(&private);
        Ok(Self { private, public })
    }

    /// Reconstruct a keypair from a stored PKCS#8 private-key PEM.
    pub fn from_private_pem(pem: &str) -> anyhow::Result<Self> {
        let private = RsaPrivateKey::from_pkcs8_pem(pem).context("parsing private key PEM")?;
        let public = RsaPublicKey::from(&private);
        Ok(Self { private, public })
    }

    /// Parse a remote actor's SPKI public-key PEM (`-----BEGIN PUBLIC KEY-----`),
    /// which is the form ActivityPub actors publish in `publicKeyPem`.
    pub fn public_from_pem(pem: &str) -> anyhow::Result<RsaPublicKey> {
        RsaPublicKey::from_public_key_pem(pem).context("parsing public key PEM")
    }

    pub fn private_key(&self) -> &RsaPrivateKey {
        &self.private
    }

    /// Export the private key as PKCS#8 PEM for storage.
    pub fn private_pem(&self) -> anyhow::Result<String> {
        Ok(self.private.to_pkcs8_pem(LineEnding::LF)?.to_string())
    }

    /// Export the public key as SPKI PEM — the exact string we put in an actor's
    /// `publicKeyPem` field.
    pub fn public_pem(&self) -> anyhow::Result<String> {
        Ok(self.public.to_public_key_pem(LineEnding::LF)?)
    }
}
