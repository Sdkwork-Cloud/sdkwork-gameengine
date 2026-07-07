//! SQLx-backed game matchmaking repository.

mod kind;
#[cfg(any(test, feature = "test-support"))]
mod memory;
mod sqlx;

pub use kind::GameMatchmakingRepositoryKind;
#[cfg(any(test, feature = "test-support"))]
pub use memory::InMemoryGameMatchmakingRepository;
pub use sqlx::SqlxGameMatchmakingRepository;
