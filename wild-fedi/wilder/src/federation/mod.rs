//! Talking to *other* servers: resolving remote actors and delivering signed
//! activities. Confining all outbound network I/O here keeps signing, timeouts,
//! and error handling in one testable place.

pub mod client;

pub use client::FederationClient;
