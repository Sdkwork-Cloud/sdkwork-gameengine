//! SQLx-backed game session repository.

mod kind;
#[cfg(any(test, feature = "test-support"))]
mod memory;
mod sqlx;

pub use kind::GameSessionRepositoryKind;
#[cfg(any(test, feature = "test-support"))]
pub use memory::InMemoryGameSessionRepository;
pub use sqlx::SqlxGameSessionRepository;
