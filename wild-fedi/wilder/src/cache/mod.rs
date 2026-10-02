//! Redis-backed, high-write / rebuildable state: anything whose loss is a
//! cache-miss, not data loss. Sessions live here (they expire anyway); later
//! phases add feeds, the captured index, remote-actor caching and rate limits.

pub mod feed;
pub mod rate_limit;
pub mod session;

pub use feed::LastVisitStore;
pub use rate_limit::RateLimiter;
pub use session::SessionStore;
