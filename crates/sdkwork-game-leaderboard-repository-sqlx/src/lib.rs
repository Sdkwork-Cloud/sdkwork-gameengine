//! SQLx-backed game leaderboard repository.

mod kind;
#[cfg(any(test, feature = "test-support"))]
mod memory;
mod sqlx;

pub use kind::LeaderboardRepositoryKind;
#[cfg(any(test, feature = "test-support"))]
pub use memory::InMemoryLeaderboardRepository;
pub use sqlx::SqlxLeaderboardRepository;
