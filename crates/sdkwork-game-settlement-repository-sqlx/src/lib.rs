//! SQLx-backed game settlement repository.

mod kind;
#[cfg(any(test, feature = "test-support"))]
mod memory;
mod sqlx;

pub use kind::GameSettlementRepositoryKind;
#[cfg(any(test, feature = "test-support"))]
pub use memory::InMemoryGameSettlementRepository;
pub use sqlx::SqlxGameSettlementRepository;
