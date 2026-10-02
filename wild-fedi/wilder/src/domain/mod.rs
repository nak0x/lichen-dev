//! The application domain: entities and the rules that govern them, independent
//! of how they are stored (repositories) or exposed (web). Keeping invariants
//! here — not scattered across SQL and handlers — is what lets the rest of the
//! system stay thin and testable.

pub mod account;
pub mod group;
pub mod measurement;
pub mod media;
pub mod note;
pub mod post;
pub mod taxon;

pub use account::{Account, NewAccount, Role};
pub use group::{Group, Member, MembershipRole, NewGroup};
pub use measurement::{Measurement, NewMeasurement};
pub use media::MediaRef;
pub use note::{NewNote, Note, NoteKind};
pub use post::{NewPost, Post};
pub use taxon::TaxonNode;
