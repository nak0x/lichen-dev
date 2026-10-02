//! The application domain: entities and the rules that govern them, independent
//! of how they are stored (repositories) or exposed (web). Keeping invariants
//! here — not scattered across SQL and handlers — is what lets the rest of the
//! system stay thin and testable.

pub mod account;
pub mod group;
pub mod media;
pub mod post;

pub use account::{Account, NewAccount, Role};
pub use group::{Group, Member, MembershipRole, NewGroup};
pub use media::MediaRef;
pub use post::{NewPost, Post};
