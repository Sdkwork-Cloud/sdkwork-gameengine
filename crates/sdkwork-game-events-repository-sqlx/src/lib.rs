//! SQLx-backed game engine events and audit repository.

mod kind;
#[cfg(any(test, feature = "test-support"))]
mod memory;
mod sqlx;

pub use kind::GameEventsRepositoryKind;
#[cfg(any(test, feature = "test-support"))]
pub use memory::InMemoryGameEventsRepository;
pub use sqlx::SqlxGameEventsRepository;
