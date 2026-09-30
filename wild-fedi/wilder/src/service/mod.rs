//! Application services: the use-case layer between the HTTP/queue edges and the
//! repositories. `ActorService` is read-side (build documents), `InboxService`
//! is write-side (apply inbound activities), and `handlers` holds the per-verb
//! Strategy implementations the inbox service dispatches to.

pub mod actor_service;
pub mod handlers;
pub mod inbox_service;
pub mod publish_service;

pub use actor_service::ActorService;
pub use inbox_service::InboxService;
pub use publish_service::PublishService;
