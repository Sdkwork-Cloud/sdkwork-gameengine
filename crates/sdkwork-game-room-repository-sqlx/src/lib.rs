//! SQLx-backed game room repository.

mod kind;
#[cfg(any(test, feature = "test-support"))]
mod memory;
mod sqlx;

pub use kind::GameRoomRepositoryKind;
#[cfg(any(test, feature = "test-support"))]
pub use memory::InMemoryGameRoomRepository;
pub use sqlx::SqlxGameRoomRepository;
