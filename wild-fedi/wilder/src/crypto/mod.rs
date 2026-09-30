//! Cryptography for federation: per-actor RSA keys and the HTTP Signatures that
//! authenticate every server-to-server request (README §2, *HTTP Signatures*).
//!
//! We implement signing/verification directly on top of RustCrypto primitives
//! rather than pulling a signature framework. It keeps us off framework/reqwest
//! version coupling, and — since signing is *the* trust anchor of the fediverse
//! — it is worth having the exact canonicalisation in plain sight.

pub mod http_signature;
pub mod keypair;

pub use http_signature::{SignatureHeader, Signer, verify};
pub use keypair::KeyPair;
